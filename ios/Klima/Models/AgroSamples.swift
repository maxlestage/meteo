import Foundation

/// Une heure de prévision agricole sur la parcelle.
struct HourlySample: Equatable, Identifiable {
    /// Horodatage local de la parcelle.
    let time: Date
    /// Code temps WMO, traduit par `WeatherCondition`.
    let weatherCode: Int
    /// Vrai entre le lever et le coucher du soleil.
    let isDay: Bool
    /// Probabilité de précipitations sur l'heure (%).
    let precipitationProbability: Double
    /// Température de l'air à 2 m (°C).
    let temperature: Double
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
    /// Température du sol à 6 cm (°C).
    let soilTemperature6cm: Double
    /// Humidité volumique du sol entre 3 et 9 cm (m³/m³).
    let soilMoisture3to9cm: Double
    /// Évapotranspiration de référence FAO-56 sur l'heure (mm).
    let et0: Double
    /// Déficit de pression de vapeur (kPa).
    let vapourPressureDeficit: Double

    var id: Date { time }
}

/// Une journée de prévision agricole, en cumuls.
struct DailySample: Equatable, Identifiable {
    /// Jour local (minuit heure de la parcelle).
    let date: Date
    /// Code temps WMO dominant de la journée.
    let weatherCode: Int
    let temperatureMin: Double
    let temperatureMax: Double
    /// Cumul de pluie du jour (mm).
    let precipitationSum: Double
    /// Probabilité de pluie maximale du jour (%).
    let precipitationProbabilityMax: Double
    /// Cumul d'ET0 FAO-56 du jour (mm).
    let et0Sum: Double
    /// Rafales maximales du jour (km/h).
    let windGustsMax: Double
    let sunrise: Date?
    let sunset: Date?

    var id: Date { date }

    /// Bilan hydrique du jour : pluie − évapotranspiration (mm).
    var balance: Double { precipitationSum - et0Sum }
}

/// Parcelle suivie : une commune ou la position de l'utilisateur.
struct Parcelle: Equatable, Codable, Identifiable, Hashable {
    let name: String
    let latitude: Double
    let longitude: Double
    /// Région / département, pour lever l'ambiguïté entre homonymes.
    var admin: String?
    var country: String?

    var id: String { "\(latitude),\(longitude)" }

    /// Sous-titre affiché sous le nom de la parcelle.
    var subtitle: String {
        let parts = [admin, country].compactMap { $0 }.filter { !$0.isEmpty }
        if parts.isEmpty {
            return String(format: "%.3f, %.3f", latitude, longitude)
        }
        return parts.joined(separator: ", ")
    }

    /// Plaine céréalière de Beauce, au premier lancement.
    static let chartres = Parcelle(
        name: "Chartres",
        latitude: 48.4468,
        longitude: 1.4892,
        admin: "Eure-et-Loir",
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
}

/// Prévision agricole complète renvoyée par le service.
struct AgroForecast: Equatable {
    let parcelle: Parcelle
    /// Fuseau retenu par l'API pour cette parcelle.
    let timezone: String
    /// Altitude du point de grille (m).
    let elevation: Double
    let current: CurrentSample
    let hourly: [HourlySample]
    let daily: [DailySample]
    let fetchedAt: Date
}
