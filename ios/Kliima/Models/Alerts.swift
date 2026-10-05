import Foundation

/// Les alertes : ce que Kliima dit sans qu'on ouvre l'application.
///
/// Miroir de `rust/klima-core/src/alerts.rs`. Toute règle ajoutée d'un côté se
/// porte de l'autre, avec les mêmes cas de test.
///
/// Ce fichier ne notifie rien. Il répond à une seule question — « qu'y a-t-il à
/// dire, maintenant ? » — à partir de la prévision et de ce qu'on a déjà dit.
///
/// Ce qu'une ville a besoin de savoir avant que ça arrive : la pluie qui
/// approche, l'orage, le gel, la chaleur, le vent. Trois principes :
///
/// 1. **On ne dit que ce qui change quelque chose.** La pluie s'annonce quand
///    elle approche et qu'il fait encore sec.
/// 2. **On ne réveille personne.** Une alerte calculée la nuit attend le matin.
///    Si l'événement est passé entre-temps, elle est abandonnée.
/// 3. **On ne répète pas.** Un délai de garde par nature d'alerte.
enum AlertKind: String, CaseIterable, Codable, Sendable {
    case pluie, orage, gel, chaleur, vent
}

/// Les seuils des alertes, plus hauts que ceux des conseils.
enum AlertSeuils {
    /// Une pluie s'annonce si elle commence dans les deux heures.
    static let pluiePreavisHeures = 2.0
    /// Orage, gel, chaleur, vent : les douze heures qui viennent.
    static let horizonHeures = 12
    /// Codes WMO d'orage.
    static let orage: Set<Int> = [95, 96, 99]
    static let gel = 0.0
    static let chaleur = 33.0
    static let rafales = 70.0
}

struct Alert: Equatable, Sendable {
    let kind: AlertKind
    /// Moment de ce qui est annoncé — pas celui de l'envoi.
    let at: Date
    /// Clés de catalogue : le domaine ne fabrique pas de phrases.
    let titleKey: String
    let bodyKey: String
    let params: [String: Double]
}

/// Ce qu'on sait déjà avoir dit. `Codable` : il survit aux redémarrages. Un
/// état enregistré par une version précédente se relit : les clés en trop sont
/// ignorées.
struct AlertState: Codable, Equatable, Sendable {
    /// Dernier envoi par nature, indexé par `AlertKind.rawValue`.
    var lastSent: [String: Date] = [:]

    static let empty = AlertState()
}

struct AlertOptions: Sendable {
    var now: Date
    /// Heure locale à partir de laquelle on se tait, et heure de reprise.
    var quietFrom: Int = 22
    var quietTo: Int = 7
    /// Délai de garde entre deux alertes de même nature (heures).
    var cooldownHours: Double = 6
    /// Calendrier injectable : les tests n'héritent pas du fuseau de la machine.
    var calendar: Calendar = .current
}

enum Alerts {
    /// Vrai si l'heure locale tombe dans la plage de silence.
    static func isQuiet(hour: Int, from: Int, to: Int) -> Bool {
        from <= to ? (hour >= from && hour < to) : (hour >= from || hour < to)
    }

    /// Repousse un envoi à la fin du silence. Renvoie `nil` si l'événement
    /// annoncé sera passé d'ici là : une alerte en retard est pire que rien.
    static func deferPastQuietHours(
        send: Date,
        event: Date,
        options: AlertOptions
    ) -> Date? {
        let calendar = options.calendar
        let hour = calendar.component(.hour, from: send)
        guard isQuiet(hour: hour, from: options.quietFrom, to: options.quietTo) else { return send }

        let base = hour >= options.quietTo ? calendar.date(byAdding: .day, value: 1, to: send)! : send
        guard let resume = calendar.date(
            bySettingHour: options.quietTo, minute: 0, second: 0, of: base
        ) else { return nil }

        return resume <= event ? resume : nil
    }

    /// Ce qu'il y a à dire maintenant.
    ///
    /// `hours` commence à l'heure en cours. Une liste vide est le cas normal.
    static func evaluate(
        hours: [HourlySample],
        state: AlertState,
        options: AlertOptions
    ) -> [Alert] {
        var alerts: [Alert] = []
        let fenetre = Array(hours.prefix(AlertSeuils.horizonHeures))
        let libre = { (kind: AlertKind) in !held(kind, state, options) }

        // 1. La pluie qui approche, tant qu'il fait sec.
        if libre(.pluie), let premiere = fenetre.first, !Ville.pluvieuse(premiere) {
            let limite = options.now.addingTimeInterval(AlertSeuils.pluiePreavisHeures * 3600)
            if let pluie = fenetre.first(where: { Ville.pluvieuse($0) && $0.time <= limite }) {
                alerts.append(Alert(
                    kind: .pluie, at: pluie.time,
                    titleKey: "alert.pluie.title", bodyKey: "alert.pluie.body",
                    params: ["probability": pluie.precipitationProbability]
                ))
            }
        }

        // 2. L'orage.
        if libre(.orage), let orage = fenetre.first(where: { AlertSeuils.orage.contains($0.weatherCode) }) {
            alerts.append(Alert(
                kind: .orage, at: orage.time,
                titleKey: "alert.orage.title", bodyKey: "alert.orage.body", params: [:]
            ))
        }

        // 3. Le gel : trottoirs, pare-brise, plaques de verglas.
        if libre(.gel), let premiere = fenetre.first(where: { $0.temperature <= AlertSeuils.gel }) {
            alerts.append(Alert(
                kind: .gel, at: premiere.time,
                titleKey: "alert.gel.title", bodyKey: "alert.gel.body",
                params: ["temperature": fenetre.map(\.temperature).min() ?? premiere.temperature]
            ))
        }

        // 4. La forte chaleur, annoncée à son heure la plus chaude — la
        //    première, à égalité.
        if libre(.chaleur) {
            var plusChaude: HourlySample?
            for heure in fenetre where heure.temperature >= AlertSeuils.chaleur {
                if let actuelle = plusChaude, actuelle.temperature >= heure.temperature { continue }
                plusChaude = heure
            }
            if let heure = plusChaude {
                alerts.append(Alert(
                    kind: .chaleur, at: heure.time,
                    titleKey: "alert.chaleur.title", bodyKey: "alert.chaleur.body",
                    params: ["temperature": heure.temperature]
                ))
            }
        }

        // 5. Le vent qui arrache.
        if libre(.vent), let premiere = fenetre.first(where: { $0.windGusts >= AlertSeuils.rafales }) {
            alerts.append(Alert(
                kind: .vent, at: premiere.time,
                titleKey: "alert.vent.title", bodyKey: "alert.vent.body",
                params: ["gusts": fenetre.map(\.windGusts).max() ?? premiere.windGusts]
            ))
        }

        return alerts
    }

    /// Enregistre ce qui vient d'être dit, pour ne pas le redire.
    static func recordSent(_ state: AlertState, _ alerts: [Alert], at sentAt: Date) -> AlertState {
        var next = state
        for alert in alerts { next.lastSent[alert.kind.rawValue] = sentAt }
        return next
    }

    private static func held(_ kind: AlertKind, _ state: AlertState, _ options: AlertOptions) -> Bool {
        guard let last = state.lastSent[kind.rawValue] else { return false }
        return options.now.timeIntervalSince(last) < options.cooldownHours * 3600
    }
}
