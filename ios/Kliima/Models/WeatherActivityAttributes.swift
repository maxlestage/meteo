import Foundation

/// Données de l'activité en direct qui affiche le temps qu'il fait.
///
/// Sa sœur, `SprayActivityAttributes`, suit un événement borné : une fenêtre
/// de traitement a un début et une fin. La météo, elle, n'en a pas — et c'est
/// la limite à connaître avant de s'en servir. iOS termine une activité en
/// direct au bout de huit heures environ ; celle-ci se relance donc chaque
/// jour, à la main.
///
/// **L'heure suivante voyage avec l'heure en cours.** La date de péremption
/// est posée au début de l'heure suivante. Passé ce moment, iOS redessine
/// l'activité en la marquant périmée — et la vue montre alors l'heure
/// suivante, calculée d'avance, comme l'heure en cours. L'île bascule ainsi
/// d'elle-même à l'heure pile, sans attendre qu'on lui envoie quoi que ce
/// soit : elle reste juste deux heures au lieu d'une.
struct WeatherActivityAttributes: Codable, Hashable {

    /// Ce qui change d'une heure à l'autre.
    ///
    /// Les clés et la forme sont un contrat : un relais qui pousse une mise à
    /// jour doit produire exactement ce JSON, les dates en secondes depuis le
    /// 1er janvier 2001 — c'est ce qu'ActivityKit décode par défaut.
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
        /// L'heure qui vient, calculée d'avance. Absente en fin de série.
        var next: NextHour?

        var condition: WeatherCondition { WeatherCondition.forCode(weatherCode) }

        /// Ce qu'il faut montrer comme « maintenant ».
        ///
        /// Tant que le contenu est frais, l'heure en cours. Une fois périmé —
        /// c'est-à-dire passé le début de l'heure suivante —, l'heure suivante,
        /// qui est devenue l'heure en cours.
        func now(stale: Bool) -> Shown {
            if stale, let next {
                return Shown(temperature: next.temperature, weatherCode: next.weatherCode,
                             isDay: next.isDay, windSpeed: next.windSpeed,
                             apparentTemperature: nil)
            }
            return Shown(temperature: temperature, weatherCode: weatherCode, isDay: isDay,
                         windSpeed: windSpeed, apparentTemperature: apparentTemperature)
        }

        /// Ce qu'il faut annoncer pour « dans une heure » : rien une fois que
        /// l'heure suivante est devenue l'heure en cours.
        func upcoming(stale: Bool) -> NextHour? { stale ? nil : next }
    }

    /// L'heure qui suit, telle que la prévision la donne.
    struct NextHour: Codable, Hashable {
        /// Son début : le moment où elle devient l'heure en cours.
        var start: Date
        var temperature: Double
        var weatherCode: Int
        var isDay: Bool
        var precipitationProbability: Double
        var windSpeed: Double

        var condition: WeatherCondition { WeatherCondition.forCode(weatherCode) }
    }

    /// Ce qu'une vue affiche comme « maintenant ».
    struct Shown: Equatable {
        let temperature: Double
        let weatherCode: Int
        let isDay: Bool
        let windSpeed: Double
        /// Absente quand « maintenant » vient de la prévision horaire, qui ne
        /// porte pas de ressenti.
        let apparentTemperature: Double?

        var condition: WeatherCondition { WeatherCondition.forCode(weatherCode) }
    }

    /// Parcelle suivie.
    var parcelleName: String
    /// Fuseau de la parcelle : l'activité affiche ses heures, pas les nôtres.
    var timeZoneIdentifier: String
    /// Où est la parcelle, pour que le relais sache quelle prévision pousser.
    /// Facultatifs : une activité ouverte par une version précédente n'en a
    /// pas, et doit se relire quand même.
    var latitude: Double?
    var longitude: Double?

    var timeZone: TimeZone {
        TimeZone(identifier: timeZoneIdentifier) ?? .current
    }
}

#if canImport(ActivityKit)
import ActivityKit

extension WeatherActivityAttributes: ActivityAttributes {}
#endif
