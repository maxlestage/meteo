import SwiftUI
import WidgetKit

/// Widget d'écran d'accueil : le temps qu'il fait en ville — la température
/// et le ciel, d'un coup d'œil, sans ouvrir l'application.
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

/// Le widget interroge l'API lui-même, pour la ville enregistrée par
/// l'application dans le groupe partagé.
struct WeatherTimelineProvider: TimelineProvider {

    func placeholder(in context: Context) -> WeatherEntry {
        WeatherEntry(date: Date(), parcelleName: Parcelle.paris.name,
                     current: nil, today: nil, hours: [], timeZone: .current)
    }

    func getSnapshot(in context: Context, completion: @escaping (WeatherEntry) -> Void) {
        Task { completion(await entries().first ?? placeholder(in: context)) }
    }

    /// Une entrée par heure à venir, calculée d'avance.
    ///
    /// C'était une seule entrée, figée au chargement : le widget montrait la
    /// mesure de 14 h 05 jusqu'à ce que WidgetKit veuille bien le recharger —
    /// parfois bien après 15 h. Les entrées suivantes commencent chacune au
    /// début d'une heure de la série, avec sa prévision : le widget bascule tout
    /// seul à l'heure pile, sans réseau.
    ///
    /// On redemande malgré tout la prévision au bout de trois heures : les
    /// modèles tournent, et douze heures calculées d'avance restent un filet,
    /// pas une raison de ne plus rien demander.
    func getTimeline(in context: Context, completion: @escaping (Timeline<WeatherEntry>) -> Void) {
        Task {
            let entries = await entries()
            let relecture = Date().addingTimeInterval(3 * 3600)
            completion(Timeline(entries: entries.isEmpty ? [placeholder(in: context)] : entries,
                                policy: .after(relecture)))
        }
    }

    private func entries() async -> [WeatherEntry] {
        let parcelle = SharedStore.loadParcelle() ?? .paris
        let maintenant = Date()

        guard let forecast = try? await AgroWeatherService().forecast(for: parcelle, days: 2) else {
            return [WeatherEntry(date: maintenant, parcelleName: parcelle.name,
                                 current: nil, today: nil, hours: [], timeZone: .current)]
        }

        let zone = TimeZone(identifier: forecast.timezone) ?? .current
        return Horizon.chronologie(
            courant: forecast.current,
            heures: forecast.hourly,
            jours: forecast.daily,
            depuis: maintenant
        ).map { moment in
            WeatherEntry(
                date: moment.date,
                parcelleName: parcelle.name,
                current: moment.courant,
                today: moment.jour,
                hours: Array(moment.heures.prefix(6)),
                timeZone: zone
            )
        }
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
