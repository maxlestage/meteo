import Foundation

/// Un modèle de prévision et le service qui le produit.
///
/// Open-Meteo redistribue les sorties brutes de plusieurs services météo
/// nationaux. Les comparer, c'est comparer de vraies sources indépendantes —
/// quatre centres de calcul différents, pas quatre habillages du même modèle.
/// Miroir de `core/src/models.ts`.
struct WeatherModel: Equatable, Hashable {
    /// Identifiant Open-Meteo, passé au paramètre `models`.
    let id: String
    /// Nom du modèle, tel que le nomme son service.
    let name: String
    /// Service qui le produit.
    let institution: String
    /// Code pays du service.
    let country: String

    static let all: [WeatherModel] = [
        WeatherModel(id: "meteofrance_seamless", name: "AROME / ARPEGE",
                     institution: "Météo-France", country: "FR"),
        WeatherModel(id: "ecmwf_ifs025", name: "IFS", institution: "ECMWF", country: "EU"),
        WeatherModel(id: "icon_seamless", name: "ICON",
                     institution: "Deutscher Wetterdienst", country: "DE"),
        WeatherModel(id: "gfs_seamless", name: "GFS", institution: "NOAA", country: "US"),
    ]

    static func named(_ id: String) -> WeatherModel? {
        all.first { $0.id == id }
    }
}

/// Relevé d'un modèle pour une échéance donnée.
struct ModelReading: Equatable {
    let model: WeatherModel
    let temperature: Double
    /// Précipitations sur l'heure (mm).
    let precipitation: Double
    let windSpeed: Double
}

/// Degré d'accord entre les modèles.
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
    let readings: [ModelReading]
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

    /// Recoupe les relevés. Renvoie `nil` s'il n'y a rien à comparer : un seul
    /// modèle ne fait pas un consensus, et le dire vaut mieux que de le laisser
    /// croire.
    static func consensus(_ readings: [ModelReading]) -> Consensus? {
        guard readings.count >= 2 else { return nil }

        let temperature = spread(readings.map(\.temperature))
        let precipitation = spread(readings.map(\.precipitation))
        let windSpeed = spread(readings.map(\.windSpeed))

        let rainy = readings.map { $0.precipitation >= ConsensusThresholds.rainThreshold }
        let agreeOnRain = rainy.allSatisfy { $0 == rainy[0] }

        return Consensus(
            readings: readings,
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
