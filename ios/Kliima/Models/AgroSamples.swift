import Foundation

/// Une heure de prévision dans la ville.
///
/// Miroir de `klima-core/src/meteo.rs`.
struct HourlySample: Equatable, Identifiable {
    /// Horodatage, à l'heure de la ville.
    let time: Date
    /// Code temps WMO, traduit par `WeatherCondition`.
    let weatherCode: Int
    /// Vrai entre le lever et le coucher du soleil.
    let isDay: Bool
    /// Probabilité de précipitations sur l'heure (%).
    let precipitationProbability: Double
    /// Température de l'air à 2 m (°C).
    let temperature: Double
    /// Température ressentie (°C).
    let apparentTemperature: Double
    /// Humidité relative à 2 m (%).
    let relativeHumidity: Double
    /// Point de rosée à 2 m (°C).
    let dewPoint: Double
    /// Précipitations sur l'heure (mm).
    let precipitation: Double
    /// Vent moyen à 10 m (km/h).
    let windSpeed: Double
    /// Rafales à 10 m (km/h).
    let windGusts: Double
    /// Indice UV.
    let uvIndex: Double

    var id: Date { time }
}

/// Une journée de prévision, en cumuls et en extrêmes.
struct DailySample: Equatable, Identifiable {
    /// Jour local (minuit, heure de la ville).
    let date: Date
    /// Code temps WMO dominant de la journée.
    let weatherCode: Int
    let temperatureMin: Double
    let temperatureMax: Double
    /// Cumul de pluie du jour (mm).
    let precipitationSum: Double
    /// Probabilité de pluie maximale du jour (%).
    let precipitationProbabilityMax: Double
    /// Rafales maximales du jour (km/h).
    let windGustsMax: Double
    /// Indice UV le plus fort de la journée.
    let uvIndexMax: Double
    let sunrise: Date?
    let sunset: Date?

    var id: Date { date }
}

/// La ville suivie : une commune choisie, ou la position de l'utilisateur.
///
/// Le nom de type est celui de l'époque agricole ; l'interface dit « ville ».
struct Parcelle: Equatable, Codable, Identifiable, Hashable {
    let name: String
    let latitude: Double
    let longitude: Double
    /// Région / département, pour lever l'ambiguïté entre homonymes.
    var admin: String?
    var country: String?

    var id: String { "\(latitude),\(longitude)" }

    /// Sous-titre affiché sous le nom de la ville.
    var subtitle: String {
        let parts = [admin, country].compactMap { $0 }.filter { !$0.isEmpty }
        if parts.isEmpty {
            return String(format: "%.3f, %.3f", latitude, longitude)
        }
        return parts.joined(separator: ", ")
    }

    /// Paris, au premier lancement, quand on ne sait pas encore où l'on est.
    static let paris = Parcelle(
        name: "Paris",
        latitude: 48.8566,
        longitude: 2.3522,
        admin: "Île-de-France",
        country: "France"
    )
}

/// Conditions observées à l'instant, pour l'en-tête.
struct CurrentSample: Equatable {
    let time: Date
    let temperature: Double
    /// Température ressentie (°C).
    let apparentTemperature: Double
    let weatherCode: Int
    let isDay: Bool
    let relativeHumidity: Double
    let windSpeed: Double
    let windGusts: Double
    /// Pression ramenée au niveau de la mer (hPa).
    let pressure: Double
}

/// Prévision complète renvoyée par le service.
struct AgroForecast: Equatable {
    let parcelle: Parcelle
    /// Fuseau retenu par l'API pour cette ville.
    let timezone: String
    /// Altitude du point de grille (m).
    let elevation: Double
    let current: CurrentSample
    let hourly: [HourlySample]
    let daily: [DailySample]
    let fetchedAt: Date
    /// Les sources qui ont fait cette prévision, quand elle est recoupée ;
    /// vide pour la prévision de base, d'un seul modèle.
    var sources: [String] = []
    /// L'accord des sources pour l'heure en cours, quand elles ont répondu.
    var consensus: Consensus?
    /// Le ciel observé à l'aéroport le plus proche, quand il y en a un assez
    /// près et assez récent (`Ciel`).
    var ciel: CielObserve?
}
