import ActivityKit
import SwiftUI
import WidgetKit

/// Activité en direct suivant une fenêtre de traitement : sur l'écran
/// verrouillé et dans l'île dynamique, on veut savoir d'un coup d'œil s'il
/// faut y aller maintenant, et ce qui bloque si ce n'est pas le cas.
struct SprayLiveActivity: Widget {
    var body: some WidgetConfiguration {
        ActivityConfiguration(for: SprayActivityAttributes.self) { context in
            LockScreenView(context: context)
                .activityBackgroundTint(Color.black.opacity(0.55))
                .activitySystemActionForegroundColor(.white)
        } dynamicIsland: { context in
            DynamicIsland {
                DynamicIslandExpandedRegion(.leading) {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(context.state.verdict.label)
                            .font(.headline)
                            .foregroundStyle(SprayPalette.color(for: context.state.verdict))
                        Text(context.attributes.parcelleName)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                DynamicIslandExpandedRegion(.trailing) {
                    VStack(alignment: .trailing, spacing: 2) {
                        Text(timerRange(context))
                            .font(.headline)
                            .monospacedDigit()
                        Text(AgroFormat.unit(context.state.windSpeed, "km/h", decimals: 0))
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                DynamicIslandExpandedRegion(.bottom) {
                    if let blocker = context.state.blocker {
                        Label(blocker.text, systemImage: "exclamationmark.triangle")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    } else {
                        Label(
                            Localized.text("activity.allClear"),
                            systemImage: "checkmark.circle"
                        )
                        .font(.caption)
                        .foregroundStyle(SprayPalette.favorable)
                    }
                }
            } compactLeading: {
                Image(systemName: "wind")
                    .foregroundStyle(SprayPalette.color(for: context.state.verdict))
            } compactTrailing: {
                Text(context.attributes.windowEnd, style: .timer)
                    .monospacedDigit()
                    .frame(maxWidth: 44)
            } minimal: {
                Image(systemName: context.state.isBlocked ? "xmark.circle.fill" : "checkmark.circle.fill")
                    .foregroundStyle(SprayPalette.color(for: context.state.verdict))
            }
            .keylineTint(SprayPalette.color(for: context.state.verdict))
        }
    }

    /// Compte à rebours jusqu'à la fin de la fenêtre.
    private func timerRange(_ context: ActivityViewContext<SprayActivityAttributes>) -> String {
        AgroFormat.hour(context.attributes.windowEnd, in: context.attributes.timeZone)
    }
}

/// Écran verrouillé : la fenêtre, le verdict, et ce qu'il faut surveiller.
private struct LockScreenView: View {
    let context: ActivityViewContext<SprayActivityAttributes>

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                VStack(alignment: .leading, spacing: 1) {
                    Text(Localized.text("spray.title"))
                        .font(.caption2.weight(.semibold))
                        .textCase(.uppercase)
                        .foregroundStyle(.secondary)
                    Text(context.attributes.parcelleName)
                        .font(.headline)
                }

                Spacer()

                VStack(alignment: .trailing, spacing: 1) {
                    Text(window)
                        .font(.headline)
                        .monospacedDigit()
                    Text(context.attributes.windowEnd, style: .relative)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }

            HStack(alignment: .firstTextBaseline, spacing: 10) {
                Text(context.state.verdict.label)
                    .font(.title2.weight(.semibold))
                    .foregroundStyle(SprayPalette.color(for: context.state.verdict))

                Text(Localized.text("activity.score", String(context.state.score)))
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                Spacer()

                Label(
                    AgroFormat.unit(context.state.windSpeed, "km/h", decimals: 0),
                    systemImage: "wind"
                )
                .font(.subheadline)
                .monospacedDigit()
            }

            if let blocker = context.state.blocker {
                Text(blocker.text)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            } else {
                Text(Localized.text("activity.allClear"))
                    .font(.caption)
                    .foregroundStyle(SprayPalette.favorable)
            }
        }
        .padding(16)
    }

    private var window: String {
        let zone = context.attributes.timeZone
        return "\(AgroFormat.hour(context.attributes.windowStart, in: zone)) → "
            + AgroFormat.hour(context.attributes.windowEnd, in: zone)
    }
}

/// Les trois couleurs de verdict, reprises de l'application.
enum SprayPalette {
    static let favorable = Color(red: 0.494, green: 0.816, blue: 0.478)
    static let acceptable = Color(red: 0.941, green: 0.757, blue: 0.294)
    static let unsuitable = Color(red: 0.937, green: 0.541, blue: 0.353)

    static func color(for verdict: SprayVerdict) -> Color {
        switch verdict {
        case .favorable: return favorable
        case .acceptable: return acceptable
        case .defavorable: return unsuitable
        }
    }
}
