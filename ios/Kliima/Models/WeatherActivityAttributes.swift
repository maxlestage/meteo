import Foundation

/// Données de l'activité en direct qui affiche le temps qu'il fait.
///
/// Sa sœur, `SprayActivityAttributes`, suit un événement borné : une fenêtre
/// de traitement a un début et une fin. La météo, elle, n'en a pas — et c'est
/// la limite à connaître avant de s'en servir. iOS termine une activité en
/// direct au bout de huit heures environ ; celle-ci se relance donc chaque
/// jour, à la main. On garde malgré tout un `staleDate` : au-delà d'une heure
/// sans nouvelle, le système grise l'affichage plutôt que de laisser croire à
/// une température fraîche.
struct WeatherActivityAttributes: Codable, Hashable {

    /// Ce qui change d'une heure à l'autre.
    struct ContentState: Codable, Hashable {
        var temperature: Double
        /// Température ressentie, la seule qui dise s'il faut une veste.
        var apparentTemperature: Double
        var weatherCode: Int
        var isDay: Bool
        var windSpeed: Double
        var temperatureMin: Double
        var temperatureMax: Double
        /// Horodatage du relevé, pour dater l'affichage.
        var updatedAt: Date

        var condition: WeatherCondition { WeatherCondition.forCode(weatherCode) }
    }

    /// Parcelle suivie.
    var parcelleName: String
    /// Fuseau de la parcelle : l'activité affiche ses heures, pas les nôtres.
    var timeZoneIdentifier: String

    var timeZone: TimeZone {
        TimeZone(identifier: timeZoneIdentifier) ?? .current
    }
}

#if canImport(ActivityKit)
import ActivityKit

extension WeatherActivityAttributes: ActivityAttributes {}
#endif
