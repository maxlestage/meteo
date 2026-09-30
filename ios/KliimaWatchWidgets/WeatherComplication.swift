import SwiftUI
import WidgetKit

/// Complication de cadran : la température de la parcelle, dans les quatre
/// formes que watchOS propose.
///
/// Au champ, la montre est souvent le seul écran qu'on sort — et jusqu'ici
/// elle ne savait dire que la prochaine fenêtre de traitement.
struct WeatherComplication: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KliimaWeatherComplication", provider: WeatherComplicationProvider()) { entry in
            WeatherComplicationView(entry: entry)
                .containerBackground(.clear, for: .widget)
        }
        .configurationDisplayName(Localized.text("weather.title"))
        .description(Localized.text("weather.widgetDescription"))
        .supportedFamilies([
            .accessoryCircular,
            .accessoryRectangular,
            .accessoryInline,
            .accessoryCorner,
        ])
    }
}

struct WeatherComplicationEntry: TimelineEntry {
    let date: Date
    let parcelleName: String
    /// Absents quand la prévision n'a pas pu être chargée.
    let current: CurrentSample?
    let today: DailySample?
    let loaded: Bool
}

struct WeatherComplicationProvider: TimelineProvider {

    func placeholder(in context: Context) -> WeatherComplicationEntry {
        WeatherComplicationEntry(date: Date(), parcelleName: Parcelle.chartres.name,
                                 current: nil, today: nil, loaded: false)
    }

    func getSnapshot(in context: Context, completion: @escaping (WeatherComplicationEntry) -> Void) {
        Task { completion(await entry()) }
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<WeatherComplicationEntry>) -> Void) {
        Task {
            let entry = await entry()
            let next = Calendar.current.date(byAdding: .hour, value: 1, to: Date()) ?? Date()
            completion(Timeline(entries: [entry], policy: .after(next)))
        }
    }

    private func entry() async -> WeatherComplicationEntry {
        let parcelle = SharedStore.loadParcelle() ?? .chartres

        guard let forecast = try? await AgroWeatherService().forecast(for: parcelle, days: 2) else {
            return WeatherComplicationEntry(date: Date(), parcelleName: parcelle.name,
                                            current: nil, today: nil, loaded: false)
        }

        return WeatherComplicationEntry(
            date: Date(),
            parcelleName: parcelle.name,
            current: forecast.current,
            today: forecast.daily.first,
            loaded: true
        )
    }
}

struct WeatherComplicationView: View {
    @Environment(\.widgetFamily) private var family
    let entry: WeatherComplicationEntry

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
        case .accessoryCorner:
            Text(courte)
                .font(.system(size: 16, weight: .medium))
                .widgetLabel(entry.parcelleName)
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
                if let condition = condition {
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
        guard let current = entry.current else { return entry.loaded ? "—" : "…" }
        return AgroFormat.temperature(current.temperature)
    }

    /// « 20° · 17° / 30° » — assez court pour une ligne de cadran.
    private var enLigne: String {
        guard let current = entry.current else { return Localized.text("widget.unavailable") }
        guard let today = entry.today else { return AgroFormat.temperature(current.temperature) }
        return "\(AgroFormat.temperature(current.temperature)) · "
            + "\(AgroFormat.temperature(today.temperatureMin)) / "
            + AgroFormat.temperature(today.temperatureMax)
    }
}
