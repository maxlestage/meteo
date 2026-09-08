import Foundation

/// Traduction des codes météo WMO renvoyés par Open-Meteo.
///
/// La même table est implémentée côté web (`web/src/domain/weather.ts`).
struct WeatherCondition: Equatable {
    let label: String
    let icon: Icon

    /// Famille de pictogramme, déclinée jour / nuit à l'affichage.
    enum Icon: String {
        case clear, partly, cloudy, fog, drizzle, rain, showers, snow, thunder

        /// Symbole système correspondant.
        func symbolName(isDay: Bool) -> String {
            switch self {
            case .clear: return isDay ? "sun.max.fill" : "moon.stars.fill"
            case .partly: return isDay ? "cloud.sun.fill" : "cloud.moon.fill"
            case .cloudy: return "cloud.fill"
            case .fog: return "cloud.fog.fill"
            case .drizzle: return "cloud.drizzle.fill"
            case .rain: return "cloud.rain.fill"
            case .showers: return isDay ? "cloud.sun.rain.fill" : "cloud.moon.rain.fill"
            case .snow: return "cloud.snow.fill"
            case .thunder: return "cloud.bolt.rain.fill"
            }
        }
    }

    private static let table: [Int: WeatherCondition] = [
        0: WeatherCondition(label: "Ciel dégagé", icon: .clear),
        1: WeatherCondition(label: "Peu nuageux", icon: .partly),
        2: WeatherCondition(label: "Partiellement nuageux", icon: .partly),
        3: WeatherCondition(label: "Couvert", icon: .cloudy),
        45: WeatherCondition(label: "Brouillard", icon: .fog),
        48: WeatherCondition(label: "Brouillard givrant", icon: .fog),
        51: WeatherCondition(label: "Bruine légère", icon: .drizzle),
        53: WeatherCondition(label: "Bruine", icon: .drizzle),
        55: WeatherCondition(label: "Bruine dense", icon: .drizzle),
        56: WeatherCondition(label: "Bruine verglaçante", icon: .drizzle),
        57: WeatherCondition(label: "Bruine verglaçante dense", icon: .drizzle),
        61: WeatherCondition(label: "Pluie faible", icon: .rain),
        63: WeatherCondition(label: "Pluie", icon: .rain),
        65: WeatherCondition(label: "Pluie forte", icon: .rain),
        66: WeatherCondition(label: "Pluie verglaçante", icon: .rain),
        67: WeatherCondition(label: "Pluie verglaçante forte", icon: .rain),
        71: WeatherCondition(label: "Neige faible", icon: .snow),
        73: WeatherCondition(label: "Neige", icon: .snow),
        75: WeatherCondition(label: "Neige forte", icon: .snow),
        77: WeatherCondition(label: "Grains de neige", icon: .snow),
        80: WeatherCondition(label: "Averses", icon: .showers),
        81: WeatherCondition(label: "Averses modérées", icon: .showers),
        82: WeatherCondition(label: "Averses violentes", icon: .showers),
        85: WeatherCondition(label: "Averses de neige", icon: .snow),
        86: WeatherCondition(label: "Averses de neige fortes", icon: .snow),
        95: WeatherCondition(label: "Orage", icon: .thunder),
        96: WeatherCondition(label: "Orage et grêle", icon: .thunder),
        99: WeatherCondition(label: "Orage et forte grêle", icon: .thunder),
    ]

    /// Libellé et pictogramme d'un code WMO. Un code inconnu retombe sur « Couvert ».
    static func forCode(_ code: Int) -> WeatherCondition {
        table[code] ?? WeatherCondition(label: "Couvert", icon: .cloudy)
    }
}
