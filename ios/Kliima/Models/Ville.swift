import Foundation

/// La ville : ce qu'on regarde avant de sortir.
///
/// Miroir de `rust/klima-core/src/ville.rs`, avec les mêmes cas de test. Toute
/// règle ajoutée d'un côté se porte de l'autre.
///
/// Trois questions, dans l'ordre où on se les pose sur le pas de la porte :
/// va-t-il pleuvoir, et quand ; que faut-il emporter ; le soleil tape-t-il.
/// Le domaine répond par des états et des motifs, jamais par des phrases.
///
/// Les séries commencent à l'heure en cours : la première heure est
/// « maintenant ».
enum VilleSeuils {
    /// Une heure est pluvieuse à partir de ce cumul (mm)…
    static let pluieMm = 0.1
    /// … ou à partir de ce risque (%).
    static let pluieProbabilite = 50.0
    /// On regarde la pluie et les conseils sur douze heures.
    static let horizonPluie = 12
    /// Ressenti sous lequel on conseille un manteau (°C).
    static let manteauRessenti = 10.0
    /// Indice UV à partir duquel on conseille des lunettes de soleil…
    static let uvLunettes = 3.0
    /// … et de la crème solaire.
    static let uvCreme = 6.0
    /// Température à partir de laquelle on conseille de boire (°C).
    static let chaleurConseil = 30.0
    /// Température sous laquelle routes et trottoirs peuvent geler (°C).
    static let gel = 0.0
    /// Rafales à partir desquelles un parapluie se retourne (km/h).
    static let rafalesConseil = 50.0
}

/// Ce que la pluie fera dans les heures qui viennent.
enum Pluie: Equatable {
    /// Rien de prévu sur `heures` heures.
    case aucune(heures: Int)
    /// Il pleut. `fin` : le début de la première heure sèche, absente si la
    /// pluie dure au-delà de l'horizon.
    case enCours(fin: Date?)
    /// La pluie arrive, avec le risque le plus fort de l'épisode et son cumul.
    case prevue(debut: Date, probabilite: Double, cumul: Double)

    var code: String {
        switch self {
        case .aucune: return "aucune"
        case .enCours: return "enCours"
        case .prevue: return "prevue"
        }
    }
}

/// Un conseil pour sortir. L'ordre des cas est celui de l'affichage.
enum Conseil: String, CaseIterable, Sendable {
    case parapluie, manteau, cremeSolaire, lunettes, hydratation, gel, vent

    var key: String { "advice.\(rawValue)" }
    var label: String { Localized.text(key) }

    /// Le symbole qui l'accompagne à l'écran.
    var symbolName: String {
        switch self {
        case .parapluie: return "umbrella.fill"
        case .manteau: return "thermometer.snowflake"
        case .cremeSolaire: return "sun.max.fill"
        case .lunettes: return "sunglasses.fill"
        case .hydratation: return "drop.fill"
        case .gel: return "snowflake"
        case .vent: return "wind"
        }
    }
}

/// L'échelle UV de l'Organisation mondiale de la santé.
enum NiveauUv: String, CaseIterable, Sendable {
    case faible, modere, eleve, tresEleve, extreme

    var key: String { "uv.\(rawValue)" }
    var label: String { Localized.text(key) }
}

enum Ville {

    /// Vrai si l'heure est pluvieuse : assez d'eau, ou assez de risque.
    static func pluvieuse(_ heure: HourlySample) -> Bool {
        heure.precipitation >= VilleSeuils.pluieMm
            || heure.precipitationProbability >= VilleSeuils.pluieProbabilite
    }

    /// La pluie des douze prochaines heures.
    static func prochainePluie(_ heures: [HourlySample]) -> Pluie {
        let fenetre = Array(heures.prefix(VilleSeuils.horizonPluie))
        guard let premiere = fenetre.first else { return .aucune(heures: 0) }

        if pluvieuse(premiere) {
            return .enCours(fin: fenetre.first { !pluvieuse($0) }?.time)
        }

        guard let debut = fenetre.firstIndex(where: pluvieuse) else {
            return .aucune(heures: fenetre.count)
        }
        let episode = fenetre[debut...].prefix(while: pluvieuse)
        return .prevue(
            debut: fenetre[debut].time,
            probabilite: episode.map(\.precipitationProbability).max() ?? 0,
            cumul: arrondi(episode.map(\.precipitation).reduce(0, +), 1)
        )
    }

    /// Ce qu'il faut emporter pour les douze prochaines heures. Une liste vide
    /// est une bonne nouvelle, pas un oubli.
    static func conseils(_ heures: [HourlySample]) -> [Conseil] {
        let fenetre = Array(heures.prefix(VilleSeuils.horizonPluie))
        guard !fenetre.isEmpty else { return [] }

        var liste: [Conseil] = []
        if fenetre.contains(where: pluvieuse) { liste.append(.parapluie) }
        if (fenetre.map(\.apparentTemperature).min() ?? .infinity) <= VilleSeuils.manteauRessenti {
            liste.append(.manteau)
        }
        let uv = fenetre.map(\.uvIndex).max() ?? 0
        if uv >= VilleSeuils.uvCreme {
            liste.append(.cremeSolaire)
        } else if uv >= VilleSeuils.uvLunettes {
            liste.append(.lunettes)
        }
        if (fenetre.map(\.temperature).max() ?? -.infinity) >= VilleSeuils.chaleurConseil {
            liste.append(.hydratation)
        }
        if (fenetre.map(\.temperature).min() ?? .infinity) <= VilleSeuils.gel {
            liste.append(.gel)
        }
        if (fenetre.map(\.windGusts).max() ?? 0) >= VilleSeuils.rafalesConseil {
            liste.append(.vent)
        }
        return liste
    }

    /// Le niveau d'un indice UV. L'indice s'arrondit avant de se classer : 2,6
    /// se lit « 3 » sur tous les bulletins, et doit donc dire « modéré ».
    static func niveauUv(_ indice: Double) -> NiveauUv {
        let i = arrondi(indice, 0)
        if i < 3 { return .faible }
        if i < 6 { return .modere }
        if i < 8 { return .eleve }
        if i < 11 { return .tresEleve }
        return .extreme
    }

    /// L'heure de la série qui contient `instant` — à défaut, la première.
    static func heure(contenant instant: Date, dans heures: [HourlySample]) -> HourlySample? {
        heures.last { $0.time <= instant && instant < $0.time.addingTimeInterval(3600) } ?? heures.first
    }

    /// `floor(x + 0,5)`, comme le cœur Rust : les deux écritures rendent le
    /// même nombre.
    static func arrondi(_ valeur: Double, _ decimales: Int) -> Double {
        let facteur = pow(10.0, Double(decimales))
        return (valeur * facteur + 0.5).rounded(.down) / facteur
    }
}
