import SwiftUI
import WidgetKit

/// Widget d'écran verrouillé : la température de la parcelle, sous l'heure.
///
/// C'est l'équivalent, sur le téléphone, de la complication de cadran de la
/// montre : les mêmes formes réduites, le même besoin — savoir le temps qu'il
/// fait sans déverrouiller.
///
/// Séparé du widget d'écran d'accueil parce que le fond diffère : là-bas une
/// carte pleine, ici rien du tout, le système dessinant le sien. Deux widgets
/// plutôt qu'un fond conditionnel, et le sélecteur d'iOS les range de toute
/// façon dans deux galeries distinctes.
struct WeatherLockScreenWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: "KliimaWeatherLockScreenWidget",
            provider: WeatherTimelineProvider()
        ) { entry in
            WeatherLockScreenView(entry: entry)
                .containerBackground(.clear, for: .widget)
        }
        .configurationDisplayName(Localized.text("weather.title"))
        .description(Localized.text("weather.widgetDescription"))
        .supportedFamilies([
            .accessoryCircular,
            .accessoryRectangular,
            .accessoryInline,
        ])
    }
}

struct WeatherLockScreenView: View {
    @Environment(\.widgetFamily) private var family
    let entry: WeatherEntry

    var body: some View {
        switch family {
        case .accessoryCircular:
            VStack(spacing: 0) {
                Image(systemName: symbole)
                    .font(.system(size: 12))
                Text(courte)
                    .font(.system(size: 15, weight: .medium))
                    .minimumScaleFactor(0.6)
            }
        case .accessoryInline:
            Label(enLigne, systemImage: symbole)
        default:
            VStack(alignment: .leading, spacing: 1) {
                Text(entry.parcelleName)
                    .font(.system(size: 11, weight: .semibold))
                    .textCase(.uppercase)
                    .lineLimit(1)
                HStack(spacing: 5) {
                    Text(courte)
                        .font(.system(size: 20, weight: .medium))
                    Image(systemName: symbole)
                        .font(.system(size: 14))
                }
                if let condition {
                    Text(condition.label)
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
        }
    }

    private var condition: WeatherCondition? {
        entry.current.map { WeatherCondition.forCode($0.weatherCode) }
    }

    /// Le symbole du ciel, ou un nuage neutre tant qu'on ne sait pas.
    private var symbole: String {
        guard let current = entry.current, let condition else { return "cloud" }
        return condition.icon.symbolName(isDay: current.isDay)
    }

    /// Forme la plus courte : la température, ou un tiret.
    private var courte: String {
        guard let current = entry.current else { return "—" }
        return AgroFormat.temperature(current.temperature)
    }

    /// « 20° · 17° / 30° » — assez court pour une ligne d'écran verrouillé.
    private var enLigne: String {
        guard let current = entry.current else { return Localized.text("widget.unavailable") }
        guard let today = entry.today else { return AgroFormat.temperature(current.temperature) }
        return "\(AgroFormat.temperature(current.temperature)) · "
            + "\(AgroFormat.temperature(today.temperatureMin)) / "
            + AgroFormat.temperature(today.temperatureMax)
    }
}
