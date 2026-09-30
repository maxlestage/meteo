import SwiftUI
import WidgetKit

/// Widget d'écran d'accueil : le temps qu'il fait sur la parcelle.
///
/// Les autres widgets de Kliima parlent tous de la fenêtre de traitement. Il
/// manquait le plus simple — la température et le ciel, d'un coup d'œil, sans
/// ouvrir l'application.
struct WeatherWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KliimaWeatherWidget", provider: WeatherTimelineProvider()) { entry in
            WeatherWidgetView(entry: entry)
                .containerBackground(.fill.tertiary, for: .widget)
        }
        .configurationDisplayName(Localized.text("weather.title"))
        .description(Localized.text("weather.widgetDescription"))
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

struct WeatherEntry: TimelineEntry {
    let date: Date
    let parcelleName: String
    /// Absents quand la prévision n'a pas pu être chargée.
    let current: CurrentSample?
    let today: DailySample?
    /// Les prochaines heures, pour la forme moyenne.
    let hours: [HourlySample]
    let timeZone: TimeZone
}

/// Le widget interroge l'API lui-même, pour la parcelle enregistrée par
/// l'application dans le groupe partagé.
struct WeatherTimelineProvider: TimelineProvider {

    func placeholder(in context: Context) -> WeatherEntry {
        WeatherEntry(date: Date(), parcelleName: Parcelle.chartres.name,
                     current: nil, today: nil, hours: [], timeZone: .current)
    }

    func getSnapshot(in context: Context, completion: @escaping (WeatherEntry) -> Void) {
        Task { completion(await entry()) }
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<WeatherEntry>) -> Void) {
        Task {
            let entry = await entry()
            // Une prévision horaire ne bouge pas plus vite qu'une heure.
            let next = Calendar.current.date(byAdding: .hour, value: 1, to: Date()) ?? Date()
            completion(Timeline(entries: [entry], policy: .after(next)))
        }
    }

    private func entry() async -> WeatherEntry {
        let parcelle = SharedStore.loadParcelle() ?? .chartres

        guard let forecast = try? await AgroWeatherService().forecast(for: parcelle, days: 2) else {
            return WeatherEntry(date: Date(), parcelleName: parcelle.name,
                                current: nil, today: nil, hours: [], timeZone: .current)
        }

        return WeatherEntry(
            date: Date(),
            parcelleName: parcelle.name,
            current: forecast.current,
            today: forecast.daily.first,
            hours: Array(forecast.hourly.prefix(6)),
            timeZone: TimeZone(identifier: forecast.timezone) ?? .current
        )
    }
}

struct WeatherWidgetView: View {
    @Environment(\.widgetFamily) private var family
    let entry: WeatherEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(entry.parcelleName)
                .font(.caption2)
                .foregroundStyle(.secondary)
                .lineLimit(1)

            if let current = entry.current {
                let condition = WeatherCondition.forCode(current.weatherCode)

                HStack(alignment: .firstTextBaseline, spacing: 6) {
                    Text(AgroFormat.temperature(current.temperature))
                        .font(family == .systemSmall ? .largeTitle : .system(size: 44))
                        .lineLimit(1)
                        .minimumScaleFactor(0.7)

                    Image(systemName: condition.icon.symbolName(isDay: current.isDay))
                        .symbolRenderingMode(.multicolor)
                        .font(.title3)
                }

                Text(condition.label)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)

                if let today = entry.today {
                    Text(bornes(today))
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                if family == .systemMedium && !entry.hours.isEmpty {
                    Spacer(minLength: 4)
                    HStack(alignment: .top, spacing: 10) {
                        ForEach(entry.hours) { hour in
                            VStack(spacing: 3) {
                                Text(AgroFormat.hour(hour.time, in: entry.timeZone))
                                    .font(.system(size: 10, weight: .semibold))
                                    .foregroundStyle(.secondary)
                                    .lineLimit(1)
                                Image(systemName: WeatherCondition.forCode(hour.weatherCode)
                                    .icon.symbolName(isDay: hour.isDay))
                                    .symbolRenderingMode(.multicolor)
                                    .font(.caption)
                                Text(AgroFormat.temperature(hour.temperature))
                                    .font(.caption2)
                                    .lineLimit(1)
                            }
                        }
                    }
                }
            } else {
                Text(Localized.text("widget.unavailable"))
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            Spacer(minLength: 0)
        }
    }

    /// « 17° / 30° » — les bornes du jour, sur la parcelle.
    private func bornes(_ today: DailySample) -> String {
        "\(AgroFormat.temperature(today.temperatureMin)) / \(AgroFormat.temperature(today.temperatureMax))"
    }
}
