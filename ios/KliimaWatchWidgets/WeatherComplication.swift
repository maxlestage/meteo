import CoreLocation
import SwiftUI
import WidgetKit

/// Complication de cadran : la température de la ville, dans les quatre
/// formes que watchOS propose.
///
/// Dans la rue, la montre est souvent le seul écran qu'on sort.
struct WeatherComplication: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KliimaWeatherComplication", provider: WeatherComplicationProvider()) { entry in
            WeatherComplicationView(entry: entry)
                // Sur un cadran, watchOS retire ce fond de lui-même. Dans la
                // pile intelligente, il reste : `.clear` y donnait une tuile
                // noire, sans rien du temps qu'il fait.
                .containerBackground(for: .widget) { CielDeLaMontre(current: entry.current) }
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
    /// Les heures qui suivent, pour dire la pluie qui vient.
    var hours: [HourlySample] = []
    var timeZone: TimeZone = .current
    /// Plus haut quand la pluie approche : la pile intelligente remonte alors
    /// Kliima ‣ d'elle-même, au moment où l'on en a besoin.
    var relevance: TimelineEntryRelevance? = nil
}

struct WeatherComplicationProvider: TimelineProvider {

    func placeholder(in context: Context) -> WeatherComplicationEntry {
        WeatherComplicationEntry(date: Date(), parcelleName: Parcelle.paris.name,
                                 current: nil, today: nil, loaded: false)
    }

    func getSnapshot(in context: Context, completion: @escaping (WeatherComplicationEntry) -> Void) {
        Task { completion(await entries().first ?? placeholder(in: context)) }
    }

    /// Une entrée par heure à venir, calculée d'avance — comme les widgets du
    /// téléphone : le cadran bascule seul à l'heure pile, sans réseau, et la
    /// prévision est redemandée au bout d'une heure — trois heures, c'était
    /// une température de l'après-midi encore affichée à la tombée du jour.
    /// Sans réseau, on retente dans un quart d'heure plutôt que dans une heure.
    func getTimeline(in context: Context, completion: @escaping (Timeline<WeatherComplicationEntry>) -> Void) {
        Task {
            let entries = await entries()
            let charge = entries.contains { $0.loaded }
            let relecture = Date().addingTimeInterval(charge ? 3600 : 15 * 60)
            completion(Timeline(entries: entries.isEmpty ? [placeholder(in: context)] : entries,
                                policy: .after(relecture)))
        }
    }

    private func entries() async -> [WeatherComplicationEntry] {
        // Là où est le poignet d'abord : sans groupe d'applications, ce que
        // l'application montre enregistre n'arrive pas jusqu'ici, et la
        // complication restait sur Paris. Puis la ville enregistrée, puis Paris.
        let parcelle: Parcelle
        if let ici = await PositionDeLaMontre.ville() {
            parcelle = ici
            SharedStore.save(ici)
        } else if let connue = SharedStore.loadParcelle() {
            parcelle = connue
        } else {
            parcelle = .paris
        }
        let maintenant = Date()

        guard let forecast = try? await AgroWeatherService().forecast(for: parcelle, days: 2) else {
            return [WeatherComplicationEntry(date: maintenant, parcelleName: parcelle.name,
                                             current: nil, today: nil, loaded: false)]
        }

        return Horizon.chronologie(
            courant: forecast.current,
            heures: forecast.hourly,
            jours: forecast.daily,
            depuis: maintenant
        ).map { moment in
            let heures = Array(moment.heures.prefix(12))
            return WeatherComplicationEntry(
                date: moment.date,
                parcelleName: parcelle.name,
                current: moment.courant,
                today: moment.jour,
                loaded: true,
                hours: heures,
                timeZone: TimeZone(identifier: forecast.timezone) ?? .current,
                relevance: Self.pertinence(heures)
            )
        }
    }

    /// La pluie dans les trois heures pèse plus qu'un ciel sec.
    static func pertinence(_ heures: [HourlySample]) -> TimelineEntryRelevance {
        switch Ville.prochainePluie(heures) {
        case .enCours:
            return TimelineEntryRelevance(score: 80, duration: 3600)
        case .prevue(let debut, _, _) where debut.timeIntervalSinceNow < 3 * 3600:
            return TimelineEntryRelevance(score: 60, duration: 3600)
        default:
            return TimelineEntryRelevance(score: 10, duration: 3600)
        }
    }
}

/// La position du poignet, pour une complication.
///
/// La complication n'a que quelques secondes : la dernière position connue
/// suffit si elle est récente, sinon un relevé borné dans le temps. Les
/// coordonnées sont arrondies à la maille (`Position`), et la ville prend le
/// nom de la commune.
enum PositionDeLaMontre {

    @MainActor
    static func ville() async -> Parcelle? {
        let gestionnaire = CLLocationManager()
        // `isAuthorizedForWidgetUpdates` n'existe pas sur watchOS : c'est
        // l'autorisation donnée à l'application montre qui vaut ici.
        switch gestionnaire.authorizationStatus {
        case .authorizedWhenInUse, .authorizedAlways: break
        default: return nil
        }

        var coordonnee: CLLocationCoordinate2D?
        if let connue = gestionnaire.location, connue.timestamp.timeIntervalSinceNow > -3 * 3600 {
            coordonnee = connue.coordinate
        } else {
            let service = LocationService()
            coordonnee = await avecDelai(6) { try? await service.currentCoordinate() }
        }
        guard let coordonnee else { return nil }

        let lieu = CLLocation(latitude: coordonnee.latitude, longitude: coordonnee.longitude)
        let commune = await avecDelai(4) { try? await CLGeocoder().reverseGeocodeLocation(lieu).first?.locality }
        return Position.parcelle(
            named: commune ?? Localized.text("search.myField"),
            latitude: coordonnee.latitude,
            longitude: coordonnee.longitude
        )
    }

    /// Le résultat de `travail`, ou rien s'il tarde.
    @MainActor
    private static func avecDelai<T>(_ secondes: Double, _ travail: @escaping () async -> T?) async -> T? {
        await withCheckedContinuation { (suite: CheckedContinuation<T?, Never>) in
            let reponse = ReponseUnique(suite)
            Task { @MainActor in reponse.rendre(await travail()) }
            Task { @MainActor in
                try? await Task.sleep(nanoseconds: UInt64(secondes * 1_000_000_000))
                reponse.rendre(nil)
            }
        }
    }
}

/// Une suite qu'on ne reprend qu'une fois, quel que soit le premier à répondre.
private final class ReponseUnique<T> {
    private let verrou = NSLock()
    private var suite: CheckedContinuation<T?, Never>?

    init(_ suite: CheckedContinuation<T?, Never>) { self.suite = suite }

    func rendre(_ valeur: T?) {
        verrou.lock()
        let attente = suite
        suite = nil
        verrou.unlock()
        attente?.resume(returning: valeur)
    }
}

/// Le fond de la pile intelligente : le ciel qu'il fait, sombre pour que le
/// texte blanc se lise — nuit, pluie, couvert ou dégagé.
struct CielDeLaMontre: View {
    let current: CurrentSample?

    var body: some View {
        LinearGradient(colors: couleurs, startPoint: .top, endPoint: .bottom)
    }

    private var couleurs: [Color] {
        guard let current else {
            return [Color(red: 0.20, green: 0.27, blue: 0.36), Color(red: 0.10, green: 0.14, blue: 0.20)]
        }
        if !current.isDay {
            return [Color(red: 0.10, green: 0.16, blue: 0.30), Color(red: 0.03, green: 0.06, blue: 0.14)]
        }
        switch WeatherCondition.forCode(current.weatherCode).icon {
        case .rain, .showers, .thunder, .drizzle:
            return [Color(red: 0.25, green: 0.33, blue: 0.45), Color(red: 0.11, green: 0.15, blue: 0.23)]
        case .cloudy, .fog, .snow:
            return [Color(red: 0.36, green: 0.43, blue: 0.52), Color(red: 0.17, green: 0.21, blue: 0.28)]
        default:
            return [Color(red: 0.22, green: 0.50, blue: 0.82), Color(red: 0.09, green: 0.27, blue: 0.55)]
        }
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
                        .symbolRenderingMode(.multicolor)
                    Spacer(minLength: 4)
                    if let today = entry.today {
                        Text("\(AgroFormat.temperature(today.temperatureMin)) / \(AgroFormat.temperature(today.temperatureMax))")
                            .font(.system(size: 11, weight: .medium))
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                }
                if let pluie = phraseDePluie {
                    // Ce qu'on vient chercher pour s'organiser : la pluie qui
                    // vient, plutôt que le ciel qu'on voit déjà.
                    Label(pluie.texte, systemImage: pluie.seche ? "umbrella" : "cloud.rain.fill")
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundStyle(pluie.seche ? Color.secondary : Color(red: 0.62, green: 0.86, blue: 1.0))
                        .lineLimit(1)
                        .minimumScaleFactor(0.7)
                } else if let condition = condition {
                    Text(condition.label(isDay: entry.current?.isDay ?? true))
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
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

    /// « Pluie vers 18 h », « Pas de pluie d'ici 12 h » — la règle de la
    /// carte Pluie de l'application.
    private var phraseDePluie: (texte: String, seche: Bool)? {
        guard !entry.hours.isEmpty else { return nil }
        switch Ville.prochainePluie(entry.hours) {
        case .aucune(let heures):
            return (Localized.text("rain.none", String(heures)), true)
        case .enCours(let fin?):
            return (Localized.text("rain.now", AgroFormat.hour(fin, in: entry.timeZone)), false)
        case .enCours(.none):
            return (Localized.text("rain.nowLasting"), false)
        case .prevue(let debut, _, _):
            return (Localized.text("rain.soon", AgroFormat.hour(debut, in: entry.timeZone)), false)
        }
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
