import SwiftUI
import WidgetKit

/// Widget d'écran d'accueil : la prochaine fenêtre de traitement, et ce qu'il
/// faut savoir avant de sortir le pulvérisateur.
struct SprayWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KlimaSprayWidget", provider: SprayTimelineProvider()) { entry in
            SprayWidgetView(entry: entry)
                .containerBackground(.fill.tertiary, for: .widget)
        }
        .configurationDisplayName(Localized.text("spray.title"))
        .description(Localized.text("widget.description"))
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

struct SprayEntry: TimelineEntry {
    let date: Date
    let parcelleName: String
    /// Absente quand la prévision n'a pas pu être chargée.
    let summary: AgroSummary?
    let current: CurrentSample?
    let timeZone: TimeZone
}

/// Le widget interroge l'API lui-même, pour la parcelle enregistrée par
/// l'application dans le groupe partagé.
struct SprayTimelineProvider: TimelineProvider {

    func placeholder(in context: Context) -> SprayEntry {
        SprayEntry(date: Date(), parcelleName: Parcelle.chartres.name,
                   summary: nil, current: nil, timeZone: .current)
    }

    func getSnapshot(in context: Context, completion: @escaping (SprayEntry) -> Void) {
        Task { completion(await entry()) }
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<SprayEntry>) -> Void) {
        Task {
            let entry = await entry()
            // Une prévision horaire ne bouge pas plus vite qu'une heure.
            let next = Calendar.current.date(byAdding: .hour, value: 1, to: Date()) ?? Date()
            completion(Timeline(entries: [entry], policy: .after(next)))
        }
    }

    private func entry() async -> SprayEntry {
        let parcelle = SharedStore.loadParcelle() ?? .chartres

        guard let forecast = try? await AgroWeatherService().forecast(for: parcelle, days: 2) else {
            return SprayEntry(date: Date(), parcelleName: parcelle.name,
                              summary: nil, current: nil, timeZone: .current)
        }

        return SprayEntry(
            date: Date(),
            parcelleName: parcelle.name,
            summary: AgroIndicators.summarize(hours: forecast.hourly, days: forecast.daily),
            current: forecast.current,
            timeZone: TimeZone(identifier: forecast.timezone) ?? .current
        )
    }
}

struct SprayWidgetView: View {
    @Environment(\.widgetFamily) private var family
    let entry: SprayEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 4) {
                Image(systemName: "wind")
                    .font(.caption2)
                Text(entry.parcelleName)
                    .font(.caption2)
                    .lineLimit(1)
                Spacer()
                if let current = entry.current {
                    Text(AgroFormat.temperature(current.temperature))
                        .font(.caption2)
                }
            }
            .foregroundStyle(.secondary)

            if let spray = entry.summary?.nextSpray {
                Text(window(spray))
                    .font(family == .systemSmall ? .title3 : .title2)
                    .foregroundStyle(spray.score >= 80 ? SprayPalette.favorable : SprayPalette.acceptable)
                Text(Localized.text("spray.score", String(spray.score)))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            } else if entry.summary != nil {
                Text(Localized.text("spray.none"))
                    .font(.headline)
                    .foregroundStyle(SprayPalette.unsuitable)
            } else {
                Text(Localized.text("widget.unavailable"))
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            if family == .systemMedium, let summary = entry.summary, let current = entry.current {
                Spacer(minLength: 4)
                HStack(spacing: 14) {
                    Detail(label: Localized.text("tile.wind"),
                           value: AgroFormat.unit(current.windSpeed, "km/h", decimals: 0))
                    Detail(label: Localized.text("tile.soil"), value: summary.soil.state.label)
                    Detail(label: Localized.text("tile.water"),
                           value: AgroFormat.signedUnit(summary.water.balance, "mm"))
                }
            }

            Spacer(minLength: 0)
        }
    }

    private func window(_ spray: SprayOpportunity) -> String {
        let start = AgroFormat.hour(spray.start, in: entry.timeZone)
        let end = AgroFormat.hour(spray.end, in: entry.timeZone)
        return family == .systemSmall ? start : "\(start) → \(end)"
    }
}

private struct Detail: View {
    let label: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 1) {
            Text(label)
                .font(.system(size: 9, weight: .semibold))
                .textCase(.uppercase)
                .foregroundStyle(.secondary)
                .lineLimit(1)
            Text(value)
                .font(.caption)
                .lineLimit(1)
        }
    }
}
