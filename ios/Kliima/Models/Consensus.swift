import Foundation

/// Une source de prévision et le service qui la produit.
///
/// Un fournisseur peut en livrer plusieurs : Open-Meteo redistribue quatre
/// modèles nationaux, MET Norway n'en livre qu'un. Ce que le recoupement
/// compare, ce sont les sources. Miroir de `core/src/providers/`.
struct WeatherSource: Equatable, Hashable {
    let id: String
    /// Nom du modèle ou du produit.
    let name: String
    /// Service qui le produit.
    let institution: String
    /// Code pays ou zone du service.
    let country: String
    /// Fournisseur par lequel on l'obtient.
    let provider: String
    /// Mention que la licence impose d'afficher.
    let attribution: String
}

/// Relevé d'une source pour l'heure en cours.
struct SourceReading: Equatable {
    let source: WeatherSource
    let temperature: Double
    /// Précipitations sur l'heure (mm).
    let precipitation: Double
    /// Vent moyen (km/h).
    let windSpeed: Double
}

/// Degré d'accord entre les sources.
enum Agreement: String {
    case forte, moyenne, faible

    var labelKey: String { "consensus.\(rawValue)" }
    var label: String { Localized.text(labelKey) }
}

/// Étendue d'une variable entre les modèles.
struct Spread: Equatable {
    /// Valeur retenue : la médiane, moins sensible qu'une moyenne à un modèle isolé.
    let median: Double
    let min: Double
    let max: Double
    /// Écart entre les extrêmes.
    let spread: Double
}

struct Consensus: Equatable {
    let readings: [SourceReading]
    /// Fournisseurs ayant répondu, sur ceux qui ont été interrogés.
    let providersAnswered: Int
    let providersQueried: Int
    let temperature: Spread
    let precipitation: Spread
    let windSpeed: Spread
    /// Vrai si tous les modèles s'accordent sur la présence ou l'absence de pluie.
    let agreeOnRain: Bool
    let agreement: Agreement
}

/// Seuils de lecture de l'accord entre modèles. Miroir de `ConsensusThresholds`
/// côté TypeScript.
enum ConsensusThresholds {
    /// Écart de température en deçà duquel l'accord est jugé fort (°C).
    static let strongTemperatureSpread = 1.5
    /// Au-delà, l'accord est jugé faible (°C).
    static let weakTemperatureSpread = 3.0
    /// Pluie considérée comme annoncée à partir de ce cumul horaire (mm).
    static let rainThreshold = 0.1
}

enum ModelConsensus {

    /// Recoupe les relevés. Renvoie `nil` s'il n'y a rien à comparer : une
    /// seule source ne fait pas un consensus, et le dire vaut mieux que de le
    /// laisser croire.
    static func consensus(
        _ readings: [SourceReading],
        answered: Int = 0,
        queried: Int = 0
    ) -> Consensus? {
        guard readings.count >= 2 else { return nil }

        let temperature = spread(readings.map(\.temperature))
        let precipitation = spread(readings.map(\.precipitation))
        let windSpeed = spread(readings.map(\.windSpeed))

        let rainy = readings.map { $0.precipitation >= ConsensusThresholds.rainThreshold }
        let agreeOnRain = rainy.allSatisfy { $0 == rainy[0] }

        let distinct = Set(readings.map(\.source.provider)).count

        return Consensus(
            readings: readings,
            providersAnswered: answered == 0 ? distinct : answered,
            providersQueried: queried == 0 ? distinct : queried,
            temperature: temperature,
            precipitation: precipitation,
            windSpeed: windSpeed,
            agreeOnRain: agreeOnRain,
            agreement: agreement(temperatureSpread: temperature.spread, agreeOnRain: agreeOnRain)
        )
    }

    /// L'accord se juge d'abord sur la température — la variable la mieux
    /// prévue — puis sur le désaccord franc que constitue « il pleut / il ne
    /// pleut pas ».
    private static func agreement(temperatureSpread: Double, agreeOnRain: Bool) -> Agreement {
        if temperatureSpread > ConsensusThresholds.weakTemperatureSpread { return .faible }
        if !agreeOnRain { return .moyenne }
        return temperatureSpread <= ConsensusThresholds.strongTemperatureSpread ? .forte : .moyenne
    }

    private static func spread(_ values: [Double]) -> Spread {
        let sorted = values.sorted()
        let low = sorted.first ?? 0
        let high = sorted.last ?? 0
        return Spread(
            median: round(median(sorted), decimals: 1),
            min: round(low, decimals: 1),
            max: round(high, decimals: 1),
            spread: round(high - low, decimals: 1)
        )
    }

    private static func median(_ sorted: [Double]) -> Double {
        guard !sorted.isEmpty else { return 0 }
        let middle = sorted.count / 2
        if sorted.count % 2 == 1 { return sorted[middle] }
        return (sorted[middle - 1] + sorted[middle]) / 2
    }

    private static func round(_ value: Double, decimals: Int) -> Double {
        let factor = pow(10.0, Double(decimals))
        return (value * factor).rounded() / factor
    }
}
