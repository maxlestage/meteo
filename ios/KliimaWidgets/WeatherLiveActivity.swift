import ActivityKit
import SwiftUI
import WidgetKit

/// Activité en direct de la météo : la température et le ciel de la parcelle,
/// sur l'écran verrouillé, dans l'île dynamique et dans la pile de la montre.
///
/// Contrairement à la fenêtre de traitement, elle ne suit pas un événement
/// borné. Elle ne porte donc ni compte à rebours ni verdict : ce qu'il fait,
/// et ce qu'il fera dans une heure.
///
/// **Elle bascule d'elle-même à l'heure pile.** Le contenu est périmé au début
/// de l'heure suivante ; iOS le redessine alors avec `isStale`, et
/// `now(stale:)` montre l'heure suivante, calculée d'avance, comme l'heure en
/// cours. Sans mise à jour, elle reste juste deux heures au lieu d'une.
///
/// Aucune couleur imposée : l'écran verrouillé suit le thème du système, clair
/// ou sombre. L'île dynamique, elle, est toujours noire — c'est le système qui
/// la dessine.
struct WeatherLiveActivity: Widget {
    var body: some WidgetConfiguration {
        ActivityConfiguration(for: WeatherActivityAttributes.self) { context in
            WeatherActivityView(context: context)
        } dynamicIsland: { context in
            let maintenant = context.state.now(stale: context.isStale)
            let suivante = context.state.upcoming(stale: context.isStale)

            return DynamicIsland {
                DynamicIslandExpandedRegion(.leading) {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(AgroFormat.temperature(maintenant.temperature))
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
                        Image(systemName: maintenant.condition.icon.symbolName(isDay: maintenant.isDay))
                            .symbolRenderingMode(.multicolor)
                            .font(.title3)
                        Text(AgroFormat.unit(maintenant.windSpeed, "km/h", decimals: 0))
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .fixedSize(horizontal: true, vertical: false)
                    }
                }

                DynamicIslandExpandedRegion(.bottom) {
                    HStack(spacing: 6) {
                        Text(maintenant.condition.label)
                            .lineLimit(1)
                        if let suivante {
                            Spacer(minLength: 4)
                            DansUneHeure(heure: suivante, timeZone: context.attributes.timeZone)
                        }
                    }
                    .font(.caption)
                    .foregroundStyle(.secondary)
                }
            } compactLeading: {
                Image(systemName: maintenant.condition.icon.symbolName(isDay: maintenant.isDay))
                    .symbolRenderingMode(.multicolor)
            } compactTrailing: {
                Text(AgroFormat.temperature(maintenant.temperature))
                    .monospacedDigit()
                    .lineLimit(1)
            } minimal: {
                Image(systemName: maintenant.condition.icon.symbolName(isDay: maintenant.isDay))
                    .symbolRenderingMode(.multicolor)
            }
        }
    }
}

/// « 15 h ☁︎ 18° · 40 % » — l'heure qui vient, calculée d'avance.
private struct DansUneHeure: View {
    let heure: WeatherActivityAttributes.NextHour
    let timeZone: TimeZone

    var body: some View {
        HStack(spacing: 3) {
            Text(AgroFormat.hour(heure.start, in: timeZone))
            Image(systemName: heure.condition.icon.symbolName(isDay: heure.isDay))
                .symbolRenderingMode(.multicolor)
            Text(AgroFormat.temperature(heure.temperature))
            if heure.precipitationProbability >= 20 {
                Text(AgroFormat.percent(heure.precipitationProbability))
            }
        }
        .monospacedDigit()
        .lineLimit(1)
        .fixedSize(horizontal: true, vertical: false)
    }
}

/// Écran verrouillé : la parcelle, la température, le ciel et les bornes du jour.
///
/// Nommée « activité » et non « écran verrouillé » : le widget d'écran
/// verrouillé, lui, a une vue qui porte déjà ce nom. Deux types de même nom
/// dans une même cible ne compilent pas, même quand l'un est privé — Swift
/// refuse la redéclaration au niveau du module.
private struct WeatherActivityView: View {
    let context: ActivityViewContext<WeatherActivityAttributes>

    private var maintenant: WeatherActivityAttributes.Shown {
        context.state.now(stale: context.isStale)
    }

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

                Image(systemName: maintenant.condition.icon.symbolName(isDay: maintenant.isDay))
                    .symbolRenderingMode(.multicolor)
                    .font(.title)
            }

            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(AgroFormat.temperature(maintenant.temperature))
                    .font(.system(size: 40, weight: .light))
                    .lineLimit(1)

                VStack(alignment: .leading, spacing: 1) {
                    Text(maintenant.condition.label)
                        .font(.subheadline)
                        .lineLimit(1)
                    Text(bornes)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                Spacer()

                Label(
                    AgroFormat.unit(maintenant.windSpeed, "km/h", decimals: 0),
                    systemImage: "wind"
                )
                .font(.subheadline)
                .monospacedDigit()
                .lineLimit(1)
                .fixedSize(horizontal: true, vertical: false)
            }

            HStack(spacing: 6) {
                // Le ressenti n'existe que pour la mesure : passée l'heure,
                // la prévision horaire n'en porte pas, et on ne l'invente pas.
                if let ressenti = maintenant.apparentTemperature {
                    Text(Localized.text("weather.feelsLike", AgroFormat.temperature(ressenti)))
                        .lineLimit(1)
                }
                Spacer(minLength: 4)
                if let suivante = context.state.upcoming(stale: context.isStale) {
                    DansUneHeure(heure: suivante, timeZone: context.attributes.timeZone)
                }
            }
            .font(.caption)
            .foregroundStyle(.secondary)
        }
        .padding(16)
    }

    /// « ↓ 17° ↑ 30° » — les bornes du jour sur la parcelle.
    private var bornes: String {
        "↓ \(AgroFormat.temperature(context.state.temperatureMin))"
            + "  ↑ \(AgroFormat.temperature(context.state.temperatureMax))"
    }
}
