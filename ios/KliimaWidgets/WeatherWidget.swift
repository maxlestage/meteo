import SwiftUI
import WidgetKit

/// Widget d'écran d'accueil : le temps qu'il fait en ville — la température
/// et le ciel, d'un coup d'œil, sans ouvrir l'application.
struct WeatherWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KliimaWeatherWidget", provider: WeatherTimelineProvider()) { entry in
            WeatherWidgetView(entry: entry)
                .containerBackground(for: .widget) { CielDuWidget(current: entry.current) }
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
    /// Les douze prochaines heures : la pluie qui vient, et le bandeau de la
    /// forme moyenne.
    let hours: [HourlySample]
    /// Les jours qui viennent, à partir de celui de l'entrée : le widget de la
    /// semaine.
    var days: [DailySample] = []
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

        // Sept jours : la semaine a son widget.
        guard let forecast = try? await AgroWeatherService().forecast(for: parcelle, days: 7) else {
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
                hours: Array(moment.heures.prefix(12)),
                days: Array(forecast.daily.filter { $0.date >= (moment.jour?.date ?? .distantPast) }.prefix(7)),
                timeZone: zone
            )
        }
    }
}

/// Le fond : le ciel qu'il fait, comme dans l'application — azur le jour,
/// ardoise sous les nuages, bleu de pluie quand il tombe quelque chose, nuit
/// profonde après le coucher. Assez sombre pour que le texte blanc se lise
/// partout ; en mode teinté, le système le retire et garde le texte.
struct CielDuWidget: View {
    let current: CurrentSample?

    var body: some View {
        LinearGradient(colors: couleurs, startPoint: .top, endPoint: .bottom)
    }

    private var couleurs: [Color] {
        guard let current else {
            return [Color(red: 0.20, green: 0.27, blue: 0.36), Color(red: 0.10, green: 0.14, blue: 0.20)]
        }
        let icone = WeatherCondition.forCode(current.weatherCode).icon
        if !current.isDay {
            return [Color(red: 0.08, green: 0.13, blue: 0.24), Color(red: 0.02, green: 0.04, blue: 0.10)]
        }
        switch icone {
        case .rain, .showers, .thunder, .drizzle:
            return [Color(red: 0.25, green: 0.33, blue: 0.45), Color(red: 0.11, green: 0.15, blue: 0.23)]
        case .cloudy, .fog, .snow:
            return [Color(red: 0.36, green: 0.43, blue: 0.52), Color(red: 0.17, green: 0.21, blue: 0.28)]
        default:
            return [Color(red: 0.22, green: 0.50, blue: 0.82), Color(red: 0.09, green: 0.27, blue: 0.55)]
        }
    }
}

/// Le widget d'écran d'accueil.
///
/// La première version empilait tout dans une colonne — ville, température,
/// ciel, bornes, puis le bandeau des heures — et la forme moyenne débordait en
/// haut et en bas, sur un fond gris qui ne disait rien du temps. Le petit dit
/// maintenant l'essentiel et la pluie qui vient ; le moyen met l'essentiel à
/// gauche et quatre heures à droite, sur un fond qui est le ciel.
struct WeatherWidgetView: View {
    @Environment(\.widgetFamily) private var family
    let entry: WeatherEntry

    private let bleuPluie = Color(red: 0.62, green: 0.86, blue: 1.0)

    var body: some View {
        Group {
            if let current = entry.current {
                switch family {
                case .systemMedium:
                    HStack(alignment: .top, spacing: 12) {
                        essentiel(current)
                            .frame(maxWidth: .infinity, alignment: .leading)
                        bandeau
                    }
                default:
                    essentiel(current)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } else {
                VStack(alignment: .leading, spacing: 4) {
                    Text(entry.parcelleName).font(.caption.weight(.semibold))
                    Text(Localized.text("widget.unavailable")).font(.footnote).opacity(0.8)
                    Spacer(minLength: 0)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .foregroundStyle(.white)
    }

    /// La ville, la température et le ciel, puis la pluie qui vient.
    private func essentiel(_ current: CurrentSample) -> some View {
        let condition = WeatherCondition.forCode(current.weatherCode)

        return VStack(alignment: .leading, spacing: 2) {
            Text(entry.parcelleName)
                .font(.caption.weight(.semibold))
                .lineLimit(1)

            // L'icône à côté de la température, pas sur une ligne à elle : un
            // petit widget n'a qu'environ 126 points de haut.
            HStack(alignment: .center, spacing: 6) {
                Text(AgroFormat.temperature(current.temperature))
                    .font(.system(size: 40, weight: .light, design: .rounded))
                    .lineLimit(1)
                    .minimumScaleFactor(0.6)
                    .widgetAccentable()
                Image(systemName: condition.icon.symbolName(isDay: current.isDay))
                    .symbolRenderingMode(.multicolor)
                    .font(.title2)
            }

            Spacer(minLength: 0)

            Text(condition.label)
                .font(.caption.weight(.medium))
                .lineLimit(1)
                .minimumScaleFactor(0.8)

            if let today = entry.today {
                Text("↓ \(AgroFormat.temperature(today.temperatureMin))  ↑ \(AgroFormat.temperature(today.temperatureMax))")
                    .font(.caption2)
                    .monospacedDigit()
                    .opacity(0.8)
                    .lineLimit(1)
            }

            if let pluie = phraseDePluie {
                Label(pluie.texte, systemImage: pluie.seche ? "umbrella" : "cloud.rain.fill")
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(pluie.seche ? Color.white.opacity(0.8) : bleuPluie)
                    .lineLimit(1)
                    .minimumScaleFactor(0.7)
                    .padding(.top, 2)
            }
        }
    }

    /// Quatre heures, sur un panneau de verre.
    private var bandeau: some View {
        HStack(spacing: 0) {
            ForEach(Array(entry.hours.dropFirst().prefix(4))) { hour in
                VStack(spacing: 4) {
                    Text(AgroFormat.hour(hour.time, in: entry.timeZone))
                        .font(.system(size: 11, weight: .semibold))
                        .opacity(0.8)
                        .lineLimit(1)
                        .minimumScaleFactor(0.7)
                    Image(systemName: WeatherCondition.forCode(hour.weatherCode).icon.symbolName(isDay: hour.isDay))
                        .symbolRenderingMode(.multicolor)
                        .font(.system(size: 18))
                        .frame(height: 22)
                    Text(AgroFormat.temperature(hour.temperature))
                        .font(.system(size: 15, weight: .medium, design: .rounded))
                        .monospacedDigit()
                        .lineLimit(1)
                    Text(hour.precipitationProbability >= 30 ? AgroFormat.percent(hour.precipitationProbability) : " ")
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundStyle(bleuPluie)
                        .lineLimit(1)
                        .minimumScaleFactor(0.7)
                }
                .frame(maxWidth: .infinity)
            }
        }
        .padding(.vertical, 8)
        .padding(.horizontal, 4)
        .frame(maxHeight: .infinity)
        .background(Color.white.opacity(0.12), in: RoundedRectangle(cornerRadius: 14))
        .frame(width: 160)
    }

    private var phraseDePluie: (texte: String, seche: Bool)? { entry.phraseDePluie }
}

extension WeatherEntry {
    /// « Pluie vers 11 h », « Il pleut, et pour un moment », « Pas de pluie
    /// d'ici 12 h » — la même règle que la carte de l'application.
    var phraseDePluie: (texte: String, seche: Bool)? {
        guard !hours.isEmpty else { return nil }
        switch Ville.prochainePluie(hours) {
        case .aucune(let heures):
            return (Localized.text("rain.none", String(heures)), true)
        case .enCours(let fin?):
            return (Localized.text("rain.now", AgroFormat.hour(fin, in: timeZone)), false)
        case .enCours(.none):
            return (Localized.text("rain.nowLasting"), false)
        case .prevue(let debut, _, _):
            return (Localized.text("rain.soon", AgroFormat.hour(debut, in: timeZone)), false)
        }
    }
}
