import ActivityKit
import SwiftUI
import WidgetKit

/// Activité en direct de la météo : la température et le ciel de la parcelle,
/// sur l'écran verrouillé, dans l'île dynamique et dans la pile de la montre.
///
/// Contrairement à la fenêtre de traitement, elle ne suit pas un événement
/// borné. Elle ne porte donc ni compte à rebours ni verdict : juste ce qu'il
/// fait, daté, pour qu'on sache de quand ça parle.
struct WeatherLiveActivity: Widget {
    var body: some WidgetConfiguration {
        ActivityConfiguration(for: WeatherActivityAttributes.self) { context in
            WeatherLockScreenView(context: context)
                .activityBackgroundTint(Color.black.opacity(0.55))
                .activitySystemActionForegroundColor(.white)
        } dynamicIsland: { context in
            DynamicIsland {
                DynamicIslandExpandedRegion(.leading) {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(AgroFormat.temperature(context.state.temperature))
                            .font(.title2.weight(.semibold))
                            .lineLimit(1)
                        Text(context.attributes.parcelleName)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                }

                DynamicIslandExpandedRegion(.trailing) {
                    VStack(alignment: .trailing, spacing: 2) {
                        Image(systemName: context.state.condition.icon
                            .symbolName(isDay: context.state.isDay))
                            .symbolRenderingMode(.multicolor)
                            .font(.title3)
                        Text(AgroFormat.unit(context.state.windSpeed, "km/h", decimals: 0))
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .fixedSize(horizontal: true, vertical: false)
                    }
                }

                DynamicIslandExpandedRegion(.bottom) {
                    Text(context.state.condition.label)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            } compactLeading: {
                Image(systemName: context.state.condition.icon
                    .symbolName(isDay: context.state.isDay))
                    .symbolRenderingMode(.multicolor)
            } compactTrailing: {
                Text(AgroFormat.temperature(context.state.temperature))
                    .monospacedDigit()
                    .lineLimit(1)
            } minimal: {
                Image(systemName: context.state.condition.icon
                    .symbolName(isDay: context.state.isDay))
                    .symbolRenderingMode(.multicolor)
            }
        }
    }
}

/// Écran verrouillé : la parcelle, la température, le ciel et les bornes du jour.
private struct WeatherLockScreenView: View {
    let context: ActivityViewContext<WeatherActivityAttributes>

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                VStack(alignment: .leading, spacing: 1) {
                    Text(Localized.text("weather.title"))
                        .font(.caption2.weight(.semibold))
                        .textCase(.uppercase)
                        .foregroundStyle(.secondary)
                    Text(context.attributes.parcelleName)
                        .font(.headline)
                        .lineLimit(1)
                }

                Spacer()

                Image(systemName: context.state.condition.icon
                    .symbolName(isDay: context.state.isDay))
                    .symbolRenderingMode(.multicolor)
                    .font(.title)
            }

            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(AgroFormat.temperature(context.state.temperature))
                    .font(.system(size: 40, weight: .light))
                    .lineLimit(1)

                VStack(alignment: .leading, spacing: 1) {
                    Text(context.state.condition.label)
                        .font(.subheadline)
                        .lineLimit(1)
                    Text(bornes)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                Spacer()

                Label(
                    AgroFormat.unit(context.state.windSpeed, "km/h", decimals: 0),
                    systemImage: "wind"
                )
                .font(.subheadline)
                .monospacedDigit()
                .lineLimit(1)
                .fixedSize(horizontal: true, vertical: false)
            }

            Text(Localized.text(
                "weather.feelsLike",
                AgroFormat.temperature(context.state.apparentTemperature)
            ))
            .font(.caption)
            .foregroundStyle(.secondary)
            .lineLimit(1)
        }
        .padding(16)
    }

    /// « ↓ 17° ↑ 30° » — les bornes du jour sur la parcelle.
    private var bornes: String {
        "↓ \(AgroFormat.temperature(context.state.temperatureMin))"
            + "  ↑ \(AgroFormat.temperature(context.state.temperatureMax))"
    }
}
