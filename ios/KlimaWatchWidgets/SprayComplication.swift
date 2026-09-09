import SwiftUI
import WidgetKit

/// Complication de cadran : l'heure de la prochaine fenêtre de traitement,
/// dans les quatre formes que watchOS propose.
struct SprayComplication: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KlimaSprayComplication", provider: ComplicationProvider()) { entry in
            ComplicationView(entry: entry)
                .containerBackground(.clear, for: .widget)
        }
        .configurationDisplayName(Localized.text("spray.title"))
        .description(Localized.text("widget.description"))
        .supportedFamilies([
            .accessoryCircular,
            .accessoryRectangular,
            .accessoryInline,
            .accessoryCorner,
        ])
    }
}

struct ComplicationEntry: TimelineEntry {
    let date: Date
    let spray: SprayOpportunity?
    /// Absente quand la prévision n'a pas pu être chargée.
    let loaded: Bool
    let timeZone: TimeZone
}

struct ComplicationProvider: TimelineProvider {

    func placeholder(in context: Context) -> ComplicationEntry {
        ComplicationEntry(date: Date(), spray: nil, loaded: false, timeZone: .current)
    }

    func getSnapshot(in context: Context, completion: @escaping (ComplicationEntry) -> Void) {
        Task { completion(await entry()) }
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<ComplicationEntry>) -> Void) {
        Task {
            let entry = await entry()
            let next = Calendar.current.date(byAdding: .hour, value: 1, to: Date()) ?? Date()
            completion(Timeline(entries: [entry], policy: .after(next)))
        }
    }

    private func entry() async -> ComplicationEntry {
        let parcelle = SharedStore.loadParcelle() ?? .chartres

        guard let forecast = try? await AgroWeatherService().forecast(for: parcelle, days: 2) else {
            return ComplicationEntry(date: Date(), spray: nil, loaded: false, timeZone: .current)
        }

        let summary = AgroIndicators.summarize(hours: forecast.hourly, days: forecast.daily)
        return ComplicationEntry(
            date: Date(),
            spray: summary.nextSpray,
            loaded: true,
            timeZone: TimeZone(identifier: forecast.timezone) ?? .current
        )
    }
}

struct ComplicationView: View {
    @Environment(\.widgetFamily) private var family
    let entry: ComplicationEntry

    var body: some View {
        switch family {
        case .accessoryCircular:
            VStack(spacing: 0) {
                Image(systemName: "wind")
                    .font(.system(size: 12))
                Text(short)
                    .font(.system(size: 15, weight: .medium))
                    .minimumScaleFactor(0.6)
            }
        case .accessoryCorner:
            Text(short)
                .font(.system(size: 16, weight: .medium))
                .widgetLabel(Localized.text("spray.title"))
        case .accessoryInline:
            Label(inline, systemImage: "wind")
        default:
            VStack(alignment: .leading, spacing: 1) {
                Text(Localized.text("spray.title"))
                    .font(.system(size: 11, weight: .semibold))
                    .textCase(.uppercase)
                Text(long)
                    .font(.system(size: 18, weight: .medium))
                if let spray = entry.spray {
                    Text(Localized.text("spray.score", String(spray.score)))
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                }
            }
        }
    }

    /// Forme la plus courte : l'heure d'ouverture, ou un tiret.
    private var short: String {
        guard let spray = entry.spray else { return entry.loaded ? "—" : "…" }
        return AgroFormat.hour(spray.start, in: entry.timeZone)
    }

    private var inline: String {
        guard let spray = entry.spray else { return Localized.text("spray.none") }
        return "\(AgroFormat.hour(spray.start, in: entry.timeZone)) → "
            + AgroFormat.hour(spray.end, in: entry.timeZone)
    }

    private var long: String {
        guard entry.loaded else { return Localized.text("widget.unavailable") }
        return inline
    }
}
