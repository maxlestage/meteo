import Foundation

/// L'air de la ville : sa qualité, et les pollens.
///
/// Miroir de `rust/klima-core/src/air.rs`, avec les mêmes cas de test.
///
/// La source est le service de qualité de l'air d'Open-Meteo, qui redistribue
/// les prévisions européennes de Copernicus (CAMS). L'indice est l'indice
/// européen, et ses six classes sont les siennes. Les pollens ne sont prévus
/// qu'en Europe : ailleurs, la liste est vide.
struct AirSample: Equatable, Sendable {
    var europeanAqi: Double?
    var pm25: Double?
    var pm10: Double?
    var nitrogenDioxide: Double?
    var ozone: Double?
    /// Grains par mètre cube, espèce par espèce, dans l'ordre de `Pollen`.
    var pollens: [(Pollen, Double)] = []

    static func == (a: AirSample, b: AirSample) -> Bool {
        a.europeanAqi == b.europeanAqi && a.pm25 == b.pm25 && a.pm10 == b.pm10
            && a.nitrogenDioxide == b.nitrogenDioxide && a.ozone == b.ozone
            && a.pollens.map(\.0) == b.pollens.map(\.0) && a.pollens.map(\.1) == b.pollens.map(\.1)
    }
}

/// Les six classes de l'indice européen.
enum QualiteAir: String, CaseIterable, Sendable {
    case bonne, correcte, moyenne, mediocre, tresMediocre, extremementMediocre

    var key: String { "air.\(rawValue)" }
    var label: String { Localized.text(key) }
}

/// Les espèces que prévoit Copernicus.
enum Pollen: String, CaseIterable, Sendable {
    case aulne, bouleau, graminees, armoise, olivier, ambroisie

    /// Le nom de la variable chez Open-Meteo.
    var variable: String {
        switch self {
        case .aulne: return "alder_pollen"
        case .bouleau: return "birch_pollen"
        case .graminees: return "grass_pollen"
        case .armoise: return "mugwort_pollen"
        case .olivier: return "olive_pollen"
        case .ambroisie: return "ragweed_pollen"
        }
    }

    var key: String { "pollen.\(rawValue)" }
    var label: String { Localized.text(key) }
}

/// L'intensité d'un pollen, sur une seule échelle pour toutes les espèces —
/// une simplification assumée, voir `air.rs`.
enum NiveauPollen: String, CaseIterable, Sendable {
    case faible, modere, eleve, tresEleve

    var key: String { "pollenLevel.\(rawValue)" }
    var label: String { Localized.text(key) }
}

enum Air {
    /// La mention que la licence de Copernicus impose d'afficher.
    static let attribution =
        "Qualité de l’air et pollens : Copernicus Atmosphere Monitoring Service, par Open-Meteo (CC BY 4.0)"

    /// Moins d'un grain par mètre cube : rien à signaler.
    static let pollenPresent = 1.0

    /// La classe d'un indice européen : bornes de 20 en 20, jusqu'à 100.
    static func qualite(_ indice: Double) -> QualiteAir {
        if indice <= 20 { return .bonne }
        if indice <= 40 { return .correcte }
        if indice <= 60 { return .moyenne }
        if indice <= 80 { return .mediocre }
        if indice <= 100 { return .tresMediocre }
        return .extremementMediocre
    }

    static func niveauPollen(_ grains: Double) -> NiveauPollen {
        if grains < 10 { return .faible }
        if grains < 50 { return .modere }
        if grains < 200 { return .eleve }
        return .tresEleve
    }

    /// Le pollen le plus présent, s'il y en a un qui vaille d'être dit. À
    /// égalité, le premier de la liste.
    static func pollenDominant(_ air: AirSample) -> (pollen: Pollen, grains: Double, niveau: NiveauPollen)? {
        var meilleur: (Pollen, Double)?
        for (pollen, grains) in air.pollens where grains.isFinite && grains >= pollenPresent {
            if let actuel = meilleur, actuel.1 >= grains { continue }
            meilleur = (pollen, grains)
        }
        guard let meilleur else { return nil }
        return (meilleur.0, meilleur.1, niveauPollen(meilleur.1))
    }
}
