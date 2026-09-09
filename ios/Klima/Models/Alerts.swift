import Foundation

/// Les alertes : ce que Klima dit sans qu'on ouvre l'application.
///
/// Miroir de `core/src/alerts.ts`. Toute règle ajoutée d'un côté se porte de
/// l'autre, avec les mêmes cas de test.
///
/// Ce fichier ne notifie rien. Il répond à une seule question — « qu'y a-t-il à
/// dire, maintenant ? » — à partir de la prévision et de ce qu'on a déjà dit.
/// L'acheminement (notification locale, APNs) est affaire de plateforme ; la
/// règle est affaire de domaine, et c'est ce qui permet de la tester sans
/// appareil.
///
/// Trois principes valent mieux que trente réglages :
///
/// 1. **Une alerte sans marge de manœuvre est du bruit.** On ne sort pas le
///    pulvérisateur en dix minutes : d'où un préavis minimum.
/// 2. **On ne réveille personne.** Une alerte calculée la nuit attend le matin.
///    Si l'événement est passé entre-temps, elle est abandonnée.
/// 3. **On ne répète pas.** Un délai de garde par nature d'alerte.
enum AlertKind: String, CaseIterable, Codable, Sendable {
    case fenetre, gel, sol, pluie
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

/// Ce qu'on sait déjà avoir dit. `Codable` : il survit aux redémarrages.
struct AlertState: Codable, Equatable, Sendable {
    /// Dernier envoi par nature, indexé par `AlertKind.rawValue`. On garde des
    /// clés de texte plutôt que l'énumération : c'est la forme que produit le
    /// côté TypeScript, et l'état reste lisible des deux bords.
    var lastSent: [String: Date] = [:]
    /// État du sol au dernier examen, pour repérer le moment où il devient portant.
    var lastSoilState: SoilState?

    static let empty = AlertState()
}

struct AlertOptions: Sendable {
    var now: Date
    /// Heure locale à partir de laquelle on se tait, et heure de reprise.
    var quietFrom: Int = 21
    var quietTo: Int = 6
    /// Délai de garde entre deux alertes de même nature (heures).
    var cooldownHours: Double = 6
    /// Préavis minimum avant une fenêtre de traitement (heures).
    var leadHours: Double = 2
    /// Préavis maximum : au-delà, il est trop tôt pour en parler.
    var horizonHours: Double = 18
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
    /// Une liste vide est le cas normal : la plupart des heures n'ont rien à
    /// annoncer, et c'est ce qui rend les alertes supportables.
    static func evaluate(
        summary: AgroSummary,
        hours: [HourlySample],
        state: AlertState,
        options: AlertOptions
    ) -> [Alert] {
        var alerts: [Alert] = []

        // 1. La fenêtre de traitement qui s'ouvre — la raison d'être des alertes.
        if let spray = summary.nextSpray, !held(.fenetre, state, options) {
            let inHours = spray.start.timeIntervalSince(options.now) / 3600
            if inHours >= options.leadHours && inHours <= options.horizonHours {
                alerts.append(Alert(
                    kind: .fenetre,
                    at: spray.start,
                    titleKey: "alert.fenetre.title",
                    bodyKey: "alert.fenetre.body",
                    params: ["score": Double(spray.score)]
                ))
            }
        }

        // 2. Le gel de la nuit. On le dit tant qu'il reste quelque chose à faire.
        if summary.frost.severity != .aucun, !held(.gel, state, options) {
            alerts.append(Alert(
                kind: .gel,
                at: hours.first(where: { $0.temperature <= 0 })?.time ?? options.now,
                titleKey: "alert.gel.title",
                bodyKey: "alert.gel.body",
                params: ["temperature": summary.frost.minTemperature]
            ))
        }

        // 3. Le sol devenu portant : c'est un changement d'état, pas un état.
        //    Sans le précédent, on se tait.
        if state.lastSoilState == .sature, summary.soil.state == .ressuye,
           !held(.sol, state, options) {
            alerts.append(Alert(
                kind: .sol,
                at: options.now,
                titleKey: "alert.sol.title",
                bodyKey: "alert.sol.body",
                params: ["moisture": summary.soil.moisture]
            ))
        }

        // 4. La pluie qui laverait un traitement fait à l'instant.
        if let spray = summary.nextSpray, !held(.pluie, state, options),
           let washout = rainAfter(hours: hours, from: spray.end, within: 6) {
            alerts.append(Alert(
                kind: .pluie,
                at: washout.time,
                titleKey: "alert.pluie.title",
                bodyKey: "alert.pluie.body",
                params: ["rain": washout.precipitation]
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

    /// Retient l'état du sol pour repérer le prochain ressuyage.
    static func recordSoil(_ state: AlertState, _ soil: SoilState) -> AlertState {
        var next = state
        next.lastSoilState = soil
        return next
    }

    private static func held(_ kind: AlertKind, _ state: AlertState, _ options: AlertOptions) -> Bool {
        guard let last = state.lastSent[kind.rawValue] else { return false }
        return options.now.timeIntervalSince(last) < options.cooldownHours * 3600
    }

    /// Première pluie significative dans les `within` heures suivant `from`.
    private static func rainAfter(
        hours: [HourlySample],
        from: Date,
        within: Double
    ) -> HourlySample? {
        let limit = from.addingTimeInterval(within * 3600)
        return hours.first {
            $0.time >= from && $0.time <= limit && $0.precipitation > AgroThresholds.sprayRainMax
        }
    }
}
