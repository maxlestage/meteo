import Foundation

/// Le guetteur : la demi-heure en cours et les deux heures qui viennent, au
/// quart d'heure près.
///
/// Miroir de `rust/klima-core/src/veille.rs`, avec les mêmes cas de test.
/// Toute règle ajoutée d'un côté se porte de l'autre.
///
/// Il lit la prévision au quart d'heure d'Open-Meteo (`minutely_15`), la
/// relit tous les quarts d'heure, et dit deux choses : la demi-heure en
/// cours — le quart entamé et le suivant —, puis les deux heures. Le domaine
/// rend des états, des heures et des quantités ; l'interface les dit.
enum VeilleSeuils {
    /// Un quart d'heure, en secondes.
    static let quart: TimeInterval = 900
    /// La demi-heure en cours : le quart entamé et le suivant.
    static let immediatQuarts = 2
    /// Deux heures : huit quarts.
    static let horizonQuarts = 8
    /// Un quart est mouillé à partir de 0,1 mm.
    static let pluieQuartMm = 0.1
    /// Intensités, en mm/h (un quart compte quatre fois) : les classes de
    /// l'Organisation météorologique mondiale.
    static let modereeMmH = 2.5
    static let forteMmH = 7.6
    /// Rafales que le guetteur signale (km/h) : le seuil du conseil de la ville.
    static let rafales = VilleSeuils.rafalesConseil
}

/// Un quart d'heure de prévision.
struct QuartSample: Equatable, Sendable {
    /// Début du quart.
    let time: Date
    /// Précipitations sur le quart (mm).
    let precipitation: Double
    let weatherCode: Int
    let temperature: Double
    let apparentTemperature: Double
    /// Rafales (km/h).
    let windGusts: Double
    let isDay: Bool
}

/// Ce qui tombe. L'ordre des cas est celui de la priorité : l'orage
/// l'emporte sur la neige, la neige sur la pluie.
enum VeilleNature: Int, Comparable, Sendable {
    case pluie, neige, orage

    static func < (a: VeilleNature, b: VeilleNature) -> Bool { a.rawValue < b.rawValue }
}

/// Avec quelle force.
enum VeilleIntensite: Int, Comparable, Sendable {
    case faible, moderee, forte

    var code: String {
        switch self {
        case .faible: return "faible"
        case .moderee: return "moderee"
        case .forte: return "forte"
        }
    }

    static func < (a: VeilleIntensite, b: VeilleIntensite) -> Bool { a.rawValue < b.rawValue }
}

/// Ce qui tombe, et avec quelle force : une seule clé, parce que l'accord se
/// fait dans la langue.
struct VeillePrecipitation: Equatable, Sendable {
    let nature: VeilleNature
    let intensite: VeilleIntensite

    var key: String {
        switch nature {
        case .orage: return "veille.kind.orage"
        case .pluie: return "veille.kind.pluie.\(intensite.code)"
        case .neige: return "veille.kind.neige.\(intensite.code)"
        }
    }

    var label: String { Localized.text(key) }
}

/// La demi-heure en cours.
enum VeilleImmediat: Equatable {
    /// Rien ne tombe d'ici une demi-heure.
    case sec
    /// Ça commence au quart suivant.
    case commence(debut: Date, precipitation: VeillePrecipitation)
    /// Ça tombe, et toute la demi-heure (`continue` côté Rust).
    case dure(precipitation: VeillePrecipitation)
    /// Ça tombe, et ça s'arrête au quart suivant.
    case cesse(fin: Date)

    var code: String {
        switch self {
        case .sec: return "sec"
        case .commence: return "commence"
        case .dure: return "continue"
        case .cesse: return "cesse"
        }
    }
}

/// Les deux heures.
enum VeilleSuite: Equatable {
    /// Rien d'ici la fin de la fenêtre.
    case sec
    /// Une averse qui n'a pas commencé ; `fin` absente si elle déborde.
    case episode(debut: Date, fin: Date?, precipitation: VeillePrecipitation, cumul: Double)
    /// Ça tombe de bout en bout.
    case persiste(precipitation: VeillePrecipitation, cumul: Double)
    /// Ça tombe maintenant, ça s'arrête à `fin`, et peut reprendre.
    case accalmie(fin: Date, reprise: Date?)

    var code: String {
        switch self {
        case .sec: return "sec"
        case .episode: return "episode"
        case .persiste: return "persiste"
        case .accalmie: return "accalmie"
        }
    }
}

/// Les plus fortes rafales de la fenêtre, et leur quart.
struct VeilleRafale: Equatable {
    let valeur: Double
    let quand: Date
}

/// Ce que le guetteur a vu.
struct VeilleLecture: Equatable {
    /// Les quarts regardés, à partir de celui qui est entamé.
    let quarts: [QuartSample]
    let immediat: VeilleImmediat
    let suite: VeilleSuite
    /// La fin de la fenêtre : le début du quart qui suit le dernier.
    let finFenetre: Date
    let rafales: VeilleRafale?
}

enum Veille {

    static func mouille(_ quart: QuartSample) -> Bool {
        quart.precipitation >= VeilleSeuils.pluieQuartMm
    }

    /// L'intensité d'un quart, d'après son débit horaire.
    static func intensite(_ quart: QuartSample) -> VeilleIntensite {
        let debit = quart.precipitation * 4
        if debit >= VeilleSeuils.forteMmH { return .forte }
        if debit >= VeilleSeuils.modereeMmH { return .moderee }
        return .faible
    }

    /// La nature de ce qui tombe, d'après le code météo du quart.
    static func nature(_ quart: QuartSample) -> VeilleNature {
        switch quart.weatherCode {
        case 95, 96, 99: return .orage
        case 71...77, 85, 86: return .neige
        default: return .pluie
        }
    }

    /// Ce qui tombe sur une suite de quarts : la nature la plus marquante et
    /// l'intensité la plus forte.
    static func precipitation(_ quarts: [QuartSample]) -> VeillePrecipitation {
        let mouilles = quarts.filter(mouille)
        return VeillePrecipitation(
            nature: mouilles.map(nature).max() ?? .pluie,
            intensite: mouilles.map(intensite).max() ?? .faible
        )
    }

    /// Le guetteur, à `maintenant`. `nil` si la série ne couvre pas au moins
    /// la demi-heure en cours.
    static func veille(_ serie: [QuartSample], maintenant: Date) -> VeilleLecture? {
        guard
            let debut = serie.firstIndex(where: { $0.time.addingTimeInterval(VeilleSeuils.quart) > maintenant }),
            serie[debut].time <= maintenant
        else { return nil }

        let quarts = Array(serie[debut...].prefix(VeilleSeuils.horizonQuarts))
        guard quarts.count >= VeilleSeuils.immediatQuarts else { return nil }

        let mouilles = quarts.map(mouille)
        let immediat: VeilleImmediat
        switch (mouilles[0], mouilles[1]) {
        case (false, false):
            immediat = .sec
        case (false, true):
            immediat = .commence(debut: quarts[1].time, precipitation: precipitation([quarts[1]]))
        case (true, true):
            immediat = .dure(precipitation: precipitation(Array(quarts[0..<2])))
        case (true, false):
            immediat = .cesse(fin: quarts[1].time)
        }

        let finFenetre = quarts[quarts.count - 1].time.addingTimeInterval(VeilleSeuils.quart)
        let suite: VeilleSuite
        if let premier = mouilles.firstIndex(of: true) {
            if premier == 0 {
                if let sec = mouilles.firstIndex(of: false) {
                    let reprise = mouilles[sec...].firstIndex(of: true).map { quarts[$0].time }
                    suite = .accalmie(fin: quarts[sec].time, reprise: reprise)
                } else {
                    suite = .persiste(precipitation: precipitation(quarts), cumul: cumul(quarts))
                }
            } else {
                let duree = mouilles[premier...].prefix(while: { $0 }).count
                let episode = Array(quarts[premier..<(premier + duree)])
                suite = .episode(
                    debut: quarts[premier].time,
                    fin: premier + duree < quarts.count ? quarts[premier + duree].time : nil,
                    precipitation: precipitation(episode),
                    cumul: cumul(episode)
                )
            }
        } else {
            suite = .sec
        }

        // À égalité, le premier quart : « vers 16 h 45 » plutôt que le dernier.
        var rafales: VeilleRafale?
        if let plus = quarts.max(by: { $0.windGusts < $1.windGusts }), plus.windGusts >= VeilleSeuils.rafales {
            rafales = VeilleRafale(valeur: plus.windGusts, quand: plus.time)
        }

        return VeilleLecture(
            quarts: quarts, immediat: immediat, suite: suite, finFenetre: finFenetre, rafales: rafales
        )
    }

    /// La série, corrigée de ce qu'un aéroport proche voit tomber (`Ciel`) :
    /// les quarts secs que le bulletin couvre (`CielObserve.tombeJusquA`) —
    /// le quart en cours toujours — prennent ce qu'on voit, à sa force. Au-delà,
    /// la prévision reprend la parole ; un ciel sec observé ne retire rien.
    static func observer(_ serie: [QuartSample], maintenant: Date, ciel: CielObserve?) -> [QuartSample] {
        guard let ciel, let tombe = ciel.tombe, let validite = ciel.tombeJusquA,
              tombe.code >= FusionSeuils.codeMouille
        else { return serie }
        // Ce qu'on voit couvre au moins la demi-heure en cours, sauf si les
        // prévisionnistes en annoncent la fin.
        var finAnnoncee = false
        if case .changement(passager: false, tombe: _, sec: true) = ciel.tendance { finAnnoncee = true }
        let quart = VeilleSeuils.quart
        let debutDuQuart = (maintenant.timeIntervalSince1970 / quart).rounded(.down) * quart
        let demiHeure = Date(timeIntervalSince1970: debutDuQuart + Double(VeilleSeuils.immediatQuarts) * quart)
        let jusquA = finAnnoncee ? validite : max(validite, demiHeure)
        let mm = Ciel.quartObserveMm(tombe.intensite)
        return serie.map { q in
            let fin = q.time.addingTimeInterval(VeilleSeuils.quart)
            let enCours = q.time <= maintenant && maintenant < fin
            let couvert = fin > maintenant && q.time < jusquA
            guard enCours || couvert, !mouille(q) else { return q }
            return QuartSample(
                time: q.time, precipitation: mm, weatherCode: tombe.code,
                temperature: q.temperature, apparentTemperature: q.apparentTemperature,
                windGusts: q.windGusts, isDay: q.isDay
            )
        }
    }

    /// La série, refaite avec le radar (`Radar`) : le premier quart prévu par
    /// le radar compte entièrement, le huitième pour un huitième, la prévision
    /// fait le reste. Un quart que le radar ne couvre pas garde sa prévision ;
    /// un quart qui devient mouillé prend un code de pluie, sauf si la
    /// prévision y mettait déjà neige ou orage. Miroir de `veille::radariser`.
    static func radariser(_ serie: [QuartSample], radar: [(debut: Date, debit: Double)]) -> [QuartSample] {
        guard let premier = radar.first?.debut else { return serie }
        return serie.map { q in
            guard let vu = radar.first(where: { $0.debut == q.time }) else { return q }
            let rang = max(0, (q.time.timeIntervalSince(premier) / VeilleSeuils.quart).rounded(.down))
            let poids = max(0, 1 - rang / Double(VeilleSeuils.horizonQuarts))
            let brut = poids * vu.debit / 4 + (1 - poids) * q.precipitation
            let precipitation = (brut * 100).rounded() / 100
            var code = q.weatherCode
            if precipitation >= VeilleSeuils.pluieQuartMm && code < FusionSeuils.codeMouille {
                code = Radar.codeDuDebit(precipitation * 4)
            }
            return QuartSample(
                time: q.time, precipitation: precipitation, weatherCode: code,
                temperature: q.temperature, apparentTemperature: q.apparentTemperature,
                windGusts: q.windGusts, isDay: q.isDay
            )
        }
    }

    /// Vrai quand l'aéroport voit tomber ce que la prévision du quart en cours
    /// ne voit pas : la fin que donneraient les modèles n'en est pas une.
    static func aveugle(_ serie: [QuartSample], maintenant: Date, ciel: CielObserve?) -> Bool {
        guard let tombe = ciel?.tombe, tombe.code >= FusionSeuils.codeMouille,
              let quart = serie.first(where: { $0.time <= maintenant && maintenant < $0.time.addingTimeInterval(VeilleSeuils.quart) })
        else { return false }
        return !mouille(quart)
    }

    /// Quand relire : une minute après le début du quart qui suit la lecture.
    static func prochaineLecture(_ luA: Date) -> Date {
        let quart = VeilleSeuils.quart
        let debut = (luA.timeIntervalSince1970 / quart).rounded(.down) * quart
        return Date(timeIntervalSince1970: debut + quart + 60)
    }

    /// Ce que versent des quarts, arrondi au dixième de millimètre.
    static func cumul(_ quarts: [QuartSample]) -> Double {
        (quarts.reduce(0) { $0 + $1.precipitation } * 10).rounded() / 10
    }
}
