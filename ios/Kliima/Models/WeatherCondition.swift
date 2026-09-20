import Foundation

/// Traduction des codes météo WMO renvoyés par Open-Meteo.
///
/// Le code renvoie une clé, pas un libellé : le texte affiché dépend de la
/// langue et vit dans `Localizable.xcstrings`. La même table est implémentée
/// côté web (`core/src/weather.ts`).
struct WeatherCondition: Equatable {
    /// Clé de catalogue, par exemple « wmo.drizzle ».
    let labelKey: String
    let icon: Icon

    /// Libellé traduit dans la langue de l'appareil.
    var label: String { Localized.text(labelKey) }

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
        0: WeatherCondition(labelKey: "wmo.clearSky", icon: .clear),
        1: WeatherCondition(labelKey: "wmo.mainlyClear", icon: .partly),
        2: WeatherCondition(labelKey: "wmo.partlyCloudy", icon: .partly),
        3: WeatherCondition(labelKey: "wmo.overcast", icon: .cloudy),
        45: WeatherCondition(labelKey: "wmo.fog", icon: .fog),
        48: WeatherCondition(labelKey: "wmo.rimeFog", icon: .fog),
        51: WeatherCondition(labelKey: "wmo.lightDrizzle", icon: .drizzle),
        53: WeatherCondition(labelKey: "wmo.drizzle", icon: .drizzle),
        55: WeatherCondition(labelKey: "wmo.denseDrizzle", icon: .drizzle),
        56: WeatherCondition(labelKey: "wmo.freezingDrizzle", icon: .drizzle),
        57: WeatherCondition(labelKey: "wmo.denseFreezingDrizzle", icon: .drizzle),
        61: WeatherCondition(labelKey: "wmo.slightRain", icon: .rain),
        63: WeatherCondition(labelKey: "wmo.rain", icon: .rain),
        65: WeatherCondition(labelKey: "wmo.heavyRain", icon: .rain),
        66: WeatherCondition(labelKey: "wmo.freezingRain", icon: .rain),
        67: WeatherCondition(labelKey: "wmo.heavyFreezingRain", icon: .rain),
        71: WeatherCondition(labelKey: "wmo.slightSnow", icon: .snow),
        73: WeatherCondition(labelKey: "wmo.snow", icon: .snow),
        75: WeatherCondition(labelKey: "wmo.heavySnow", icon: .snow),
        77: WeatherCondition(labelKey: "wmo.snowGrains", icon: .snow),
        80: WeatherCondition(labelKey: "wmo.showers", icon: .showers),
        81: WeatherCondition(labelKey: "wmo.moderateShowers", icon: .showers),
        82: WeatherCondition(labelKey: "wmo.violentShowers", icon: .showers),
        85: WeatherCondition(labelKey: "wmo.snowShowers", icon: .snow),
        86: WeatherCondition(labelKey: "wmo.heavySnowShowers", icon: .snow),
        95: WeatherCondition(labelKey: "wmo.thunderstorm", icon: .thunder),
        96: WeatherCondition(labelKey: "wmo.thunderstormHail", icon: .thunder),
        99: WeatherCondition(labelKey: "wmo.thunderstormHeavyHail", icon: .thunder),
    ]

    /// Clé de libellé et pictogramme d'un code WMO. Un code inconnu retombe sur « couvert ».
    static func forCode(_ code: Int) -> WeatherCondition {
        table[code] ?? WeatherCondition(labelKey: "wmo.overcast", icon: .cloudy)
    }
}
