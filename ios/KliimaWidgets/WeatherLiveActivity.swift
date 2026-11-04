import ActivityKit
import SwiftUI
import WidgetKit

/// Activité en direct de la météo : la température et le ciel de la ville,
/// sur l'écran verrouillé, dans l'île dynamique et dans la pile de la montre.
///
/// Elle ne suit pas un événement borné. Elle ne porte donc ni compte à
/// rebours ni verdict : ce qu'il fait, et ce qu'il fera dans une heure.
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
            ActiviteSelonFamille(context: context)
        } dynamicIsland: { context in
            Self.ile(context)
        }
        // Sans famille supplémentaire, la montre n'en recevait que le compact
        // de l'île : une icône et une température, sur un écran noir. Avec
        // `.small`, elle a sa propre vue dans la pile intelligente.
        .supplementalActivityFamilies([.small, .medium])
    }

    /// L'île dynamique.
    static func ile(_ context: ActivityViewContext<WeatherActivityAttributes>) -> DynamicIsland {
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
                        .accessibilityHidden(true)
                    Text(AgroFormat.unit(maintenant.windSpeed, "km/h", decimals: 0))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .fixedSize(horizontal: true, vertical: false)
                }
            }

            DynamicIslandExpandedRegion(.bottom) {
                HStack(spacing: 6) {
                    Text(maintenant.condition.label(isDay: maintenant.isDay))
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
                .accessibilityLabel(maintenant.condition.label(isDay: maintenant.isDay))
        } compactTrailing: {
            Text(AgroFormat.temperature(maintenant.temperature))
                .monospacedDigit()
                .lineLimit(1)
        } minimal: {
            Image(systemName: maintenant.condition.icon.symbolName(isDay: maintenant.isDay))
                .symbolRenderingMode(.multicolor)
                .accessibilityLabel(maintenant.condition.label(isDay: maintenant.isDay))
        }
    }
}

/// L'écran verrouillé de l'iPhone en `.medium`, la montre en `.small`.
private struct ActiviteSelonFamille: View {
    let context: ActivityViewContext<WeatherActivityAttributes>

    @Environment(\.activityFamily) private var famille

    var body: some View {
        switch famille {
        case .small:
            ActiviteSurLaMontre(context: context)
        default:
            WeatherActivityView(context: context)
        }
    }
}

/// La montre : un bloc de la hauteur de la température.
///
/// La pile intelligente ne donne à la famille `.small` que la hauteur d'une
/// grande ligne : la première version, en trois rangées (ville et bornes, la
/// température, l'heure qui vient), y était coupée en haut et en bas. Tout
/// tient maintenant à côté de la température : le ciel, le ressenti, l'heure
/// qui vient, chacun sur une ligne qui rétrécit plutôt que de déborder.
private struct ActiviteSurLaMontre: View {
    let context: ActivityViewContext<WeatherActivityAttributes>

    var body: some View {
        let maintenant = context.state.now(stale: context.isStale)
        let suivante = context.state.upcoming(stale: context.isStale)

        HStack(alignment: .center, spacing: 6) {
            Image(systemName: maintenant.condition.icon.symbolName(isDay: maintenant.isDay))
                .symbolRenderingMode(.multicolor)
                .font(.title3)
                .accessibilityHidden(true)

            Text(AgroFormat.temperature(maintenant.temperature))
                .font(.system(size: 32, weight: .medium, design: .rounded))
                .monospacedDigit()
                .lineLimit(1)
                .minimumScaleFactor(0.7)
                .fixedSize(horizontal: true, vertical: false)

            VStack(alignment: .leading, spacing: 0) {
                Text(maintenant.condition.label(isDay: maintenant.isDay))
                    .font(.caption2.weight(.semibold))
                if let ressenti = maintenant.apparentTemperature {
                    Text(Localized.text("weather.feelsLike", AgroFormat.temperature(ressenti)))
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                } else {
                    Text("↓\(AgroFormat.temperature(context.state.temperatureMin)) ↑\(AgroFormat.temperature(context.state.temperatureMax))")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .monospacedDigit()
                        .accessibilityLabel(Localized.amplitude(min: context.state.temperatureMin,
                                                                max: context.state.temperatureMax))
                }
                if let suivante {
                    HeureCompacte(heure: suivante, timeZone: context.attributes.timeZone)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
            .lineLimit(1)
            .minimumScaleFactor(0.6)

            Spacer(minLength: 0)
        }
        .padding(.horizontal, 4)
    }
}

/// L'heure qui vient, dans la place qu'il reste : avec son ciel et son risque
/// de pluie si ça tient, sinon sans le ciel, sinon l'heure et la température.
private struct HeureCompacte: View {
    let heure: WeatherActivityAttributes.NextHour
    let timeZone: TimeZone

    var body: some View {
        let quand = AgroFormat.hour(heure.start, in: timeZone)
        let temperature = AgroFormat.temperature(heure.temperature)
        let risque = Ville.risqueAffiche(heure.precipitationProbability) ? AgroFormat.percent(heure.precipitationProbability) : nil

        ViewThatFits(in: .horizontal) {
            HStack(spacing: 2) {
                Text(quand)
                Image(systemName: heure.condition.icon.symbolName(isDay: heure.isDay))
                    .symbolRenderingMode(.multicolor)
                    .accessibilityLabel(heure.condition.label(isDay: heure.isDay))
                Text(temperature)
                if let risque { Text(risque) }
            }
            Text([quand, temperature, risque].compactMap { $0 }.joined(separator: " · "))
            Text("\(quand) · \(temperature)")
        }
        .monospacedDigit()
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
                .accessibilityLabel(heure.condition.label(isDay: heure.isDay))
            Text(AgroFormat.temperature(heure.temperature))
            if Ville.risqueAffiche(heure.precipitationProbability) {
                Text(AgroFormat.percent(heure.precipitationProbability))
            }
        }
        .monospacedDigit()
        .lineLimit(1)
        .fixedSize(horizontal: true, vertical: false)
    }
}

/// Écran verrouillé : la ville, la température, le ciel, et l'heure qui vient.
///
/// La première version était une carte blanche : le croissant de lune, blanc
/// lui aussi, y disparaissait, et « Ciel dégagé » s'affichait à 22 h. Elle
/// porte maintenant le ciel qu'il fait en fond — celui des widgets —, le texte
/// en blanc, le temps selon qu'il fait jour ou nuit, et en bas, sur une bande
/// de verre, les bornes du jour et l'heure qui vient.
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
        let condition = WeatherCondition.forCode(maintenant.weatherCode)
        let suivante = context.state.upcoming(stale: context.isStale)

        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 5) {
                Text(verbatim: "Kliima ‣")
                    .font(.caption.weight(.heavy))
                Text(context.attributes.parcelleName)
                    .font(.caption.weight(.semibold))
                    .opacity(0.85)
                    .lineLimit(1)
                Spacer(minLength: 6)
                Label(AgroFormat.unit(maintenant.windSpeed, "km/h", decimals: 0), systemImage: "wind")
                    .font(.caption.weight(.semibold))
                    .monospacedDigit()
                    .opacity(0.85)
                    .lineLimit(1)
                    .fixedSize(horizontal: true, vertical: false)
            }

            HStack(alignment: .center, spacing: 12) {
                Image(systemName: condition.icon.symbolName(isDay: maintenant.isDay))
                    .symbolRenderingMode(.multicolor)
                    .font(.system(size: 34))
                    .frame(width: 44)
                    .accessibilityHidden(true)
                Text(AgroFormat.temperature(maintenant.temperature))
                    .font(.system(size: 46, weight: .medium, design: .rounded))
                    .monospacedDigit()
                    .lineLimit(1)
                    .fixedSize(horizontal: true, vertical: false)
                VStack(alignment: .leading, spacing: 2) {
                    Text(condition.label(isDay: maintenant.isDay))
                        .font(.headline)
                        .lineLimit(1)
                        .minimumScaleFactor(0.75)
                    // Le ressenti n'existe que pour la mesure : passée
                    // l'heure, la prévision horaire n'en porte pas, et on ne
                    // l'invente pas.
                    if let ressenti = maintenant.apparentTemperature {
                        Text(Localized.text("weather.feelsLike", AgroFormat.temperature(ressenti)))
                            .font(.subheadline)
                            .opacity(0.8)
                            .lineLimit(1)
                    }
                }
                Spacer(minLength: 0)
            }

            HStack(spacing: 8) {
                Text(bornes)
                    .monospacedDigit()
                    .lineLimit(1)
                Spacer(minLength: 6)
                if let suivante {
                    DansUneHeure(heure: suivante, timeZone: context.attributes.timeZone)
                }
            }
            .font(.footnote.weight(.semibold))
            .padding(.horizontal, 12)
            .padding(.vertical, 7)
            .background(Color.white.opacity(0.16), in: Capsule())
        }
        .foregroundStyle(.white)
        .padding(16)
        .background(CielDeLActivite(weatherCode: maintenant.weatherCode, isDay: maintenant.isDay))
        .activityBackgroundTint(CielDeLActivite.teinte(weatherCode: maintenant.weatherCode, isDay: maintenant.isDay))
        .activitySystemActionForegroundColor(.white)
    }

    /// « ↓ 4°  ↑ 16° » — les bornes du jour dans la ville.
    private var bornes: String {
        "↓ \(AgroFormat.temperature(context.state.temperatureMin))"
            + "  ↑ \(AgroFormat.temperature(context.state.temperatureMax))"
    }
}

/// Le ciel de l'activité : les couleurs des widgets, d'après le code et
/// l'heure du jour — nuit profonde, pluie, couvert, azur.
private struct CielDeLActivite: View {
    let weatherCode: Int
    let isDay: Bool

    var body: some View {
        LinearGradient(colors: Self.couleurs(weatherCode: weatherCode, isDay: isDay),
                       startPoint: .topLeading, endPoint: .bottomTrailing)
    }

    /// La teinte unie que le système montre quand il ne dessine pas le fond.
    static func teinte(weatherCode: Int, isDay: Bool) -> Color {
        couleurs(weatherCode: weatherCode, isDay: isDay).last ?? .black
    }

    static func couleurs(weatherCode: Int, isDay: Bool) -> [Color] {
        if !isDay {
            return [Color(red: 0.12, green: 0.18, blue: 0.34), Color(red: 0.03, green: 0.06, blue: 0.15)]
        }
        switch WeatherCondition.forCode(weatherCode).icon {
        case .rain, .showers, .thunder, .drizzle:
            return [Color(red: 0.27, green: 0.36, blue: 0.48), Color(red: 0.11, green: 0.15, blue: 0.23)]
        case .cloudy, .fog, .snow:
            return [Color(red: 0.40, green: 0.47, blue: 0.56), Color(red: 0.17, green: 0.21, blue: 0.28)]
        default:
            return [Color(red: 0.24, green: 0.55, blue: 0.88), Color(red: 0.09, green: 0.27, blue: 0.55)]
        }
    }
}
