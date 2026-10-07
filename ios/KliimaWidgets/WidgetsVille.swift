import SwiftUI
import WidgetKit

/// Les autres widgets : la pluie, les heures, la semaine, et de quoi sortir.
///
/// Un seul widget, c'était une seule question — « quel temps fait-il ? ».
/// Chacun de ceux-ci en pose une autre, celle qu'on se pose devant la porte :
/// va-t-il pleuvoir, et quand ; qu'en sera-t-il cet après-midi ; et cette
/// semaine ; faut-il un manteau. Tous lisent la même prévision
/// (`WeatherTimelineProvider`), basculent seuls à l'heure pile, et portent le
/// même ciel en fond que le widget Météo.

private let bleuPluie = Color(red: 0.62, green: 0.86, blue: 1.0)

/// Un intitulé en petites capitales, avec son symbole.
private struct Titre: View {
    let symbole: String
    let texte: String

    var body: some View {
        Label(texte, systemImage: symbole)
            .font(.caption2.weight(.bold))
            .textCase(.uppercase)
            .opacity(0.85)
            .lineLimit(1)
    }
}

/// Ce qu'on montre quand la prévision n'a pas pu être chargée.
private struct Indisponible: View {
    let ville: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(ville).font(.caption.weight(.semibold))
            Text(Localized.text("widget.unavailable")).font(.footnote).opacity(0.8)
            Spacer(minLength: 0)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

// MARK: - Pluie

/// « Va-t-il pleuvoir, et quand ? » — la phrase de la carte Pluie, les douze
/// heures en barres, et (en moyen) ce qu'il faut emporter.
struct PluieWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KliimaPluieWidget", provider: WeatherTimelineProvider()) { entry in
            PluieWidgetView(entry: entry)
                .containerBackground(for: .widget) { CielDuWidget(current: entry.current) }
        }
        .configurationDisplayName(Localized.text("widget.rain.name"))
        .description(Localized.text("widget.rain.description"))
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

struct PluieWidgetView: View {
    @Environment(\.widgetFamily) private var family
    let entry: WeatherEntry

    var body: some View {
        Group {
            if entry.current == nil || entry.hours.isEmpty {
                Indisponible(ville: entry.parcelleName)
            } else if family == .systemMedium {
                HStack(alignment: .top, spacing: 14) {
                    pluie
                    Emporter(conseils: Ville.conseils(entry.hours), limite: 4)
                        .frame(width: 118, alignment: .leading)
                }
            } else {
                pluie
            }
        }
        .foregroundStyle(.white)
    }

    private var pluie: some View {
        let phrase = entry.phraseDePluie
        return VStack(alignment: .leading, spacing: 4) {
            Titre(symbole: phrase?.seche == false ? "cloud.rain.fill" : "umbrella", texte: Localized.text("rain.title"))
            Text(phrase?.texte ?? "")
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(phrase?.seche == false ? bleuPluie : .white)
                .lineLimit(2)
                .minimumScaleFactor(0.75)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 2)
            BarresDePluie(heures: entry.hours, hauteur: 34)
            HStack {
                Text(Localized.text("hourly.now"))
                Spacer()
                if let derniere = entry.hours.last {
                    Text(AgroFormat.hour(derniere.time, in: entry.timeZone))
                }
            }
            .font(.system(size: 10, weight: .semibold))
            .opacity(0.75)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

/// Douze barres, une par heure, aussi hautes que le risque de pluie ; bleues
/// quand l'heure est pluvieuse au sens de la carte.
private struct BarresDePluie: View {
    let heures: [HourlySample]
    let hauteur: CGFloat

    var body: some View {
        HStack(alignment: .bottom, spacing: 2) {
            ForEach(heures) { heure in
                ZStack(alignment: .bottom) {
                    RoundedRectangle(cornerRadius: 2).fill(Color.white.opacity(0.12))
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Ville.pluvieuse(heure) ? bleuPluie : Color.white.opacity(0.35))
                        .frame(height: max(3, hauteur * min(heure.precipitationProbability, 100) / 100))
                }
                .frame(maxWidth: .infinity)
            }
        }
        .frame(height: hauteur)
    }
}

/// Ce qu'il faut emporter, en symboles.
private struct Emporter: View {
    let conseils: [Conseil]
    let limite: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            Titre(symbole: "bag", texte: Localized.text("advice.title"))
            if conseils.isEmpty {
                Text(Localized.text("advice.none"))
                    .font(.caption)
                    .opacity(0.85)
                    .lineLimit(3)
                    .minimumScaleFactor(0.8)
            } else {
                ForEach(conseils.prefix(limite), id: \.self) { conseil in
                    Label(conseil.label, systemImage: conseil.symbolName)
                        .font(.caption)
                        .symbolRenderingMode(.multicolor)
                        .lineLimit(1)
                        .minimumScaleFactor(0.7)
                }
            }
            Spacer(minLength: 0)
        }
    }
}

// MARK: - Prochaines heures

/// « Et cet après-midi ? » — six heures, et en grand les jours qui suivent.
struct HeuresWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KliimaHeuresWidget", provider: WeatherTimelineProvider()) { entry in
            HeuresWidgetView(entry: entry)
                .containerBackground(for: .widget) { CielDuWidget(current: entry.current) }
        }
        .configurationDisplayName(Localized.text("widget.hours.name"))
        .description(Localized.text("widget.hours.description"))
        .supportedFamilies([.systemMedium, .systemLarge])
    }
}

struct HeuresWidgetView: View {
    @Environment(\.widgetFamily) private var family
    let entry: WeatherEntry

    var body: some View {
        Group {
            if let current = entry.current {
                VStack(alignment: .leading, spacing: family == .systemLarge ? 10 : 6) {
                    entete(current)
                    Spacer(minLength: 0)
                    colonnes
                    if family == .systemLarge, entry.days.count > 1 {
                        Divider().overlay(Color.white.opacity(0.3))
                        ForEach(entry.days.dropFirst().prefix(4)) { jour in
                            LigneDeJour(jour: jour, bornes: bornes, timeZone: entry.timeZone, aujourdhui: false)
                        }
                    }
                }
            } else {
                Indisponible(ville: entry.parcelleName)
            }
        }
        .foregroundStyle(.white)
    }

    private func entete(_ current: CurrentSample) -> some View {
        let condition = WeatherCondition.forCode(current.weatherCode)
        return HStack(alignment: .center, spacing: 8) {
            VStack(alignment: .leading, spacing: 0) {
                Text(entry.parcelleName)
                    .font(.caption.weight(.semibold))
                    .lineLimit(1)
                Text(condition.label)
                    .font(.caption2)
                    .opacity(0.8)
                    .lineLimit(1)
            }
            Spacer(minLength: 4)
            Image(systemName: condition.icon.symbolName(isDay: current.isDay))
                .symbolRenderingMode(.multicolor)
                .font(family == .systemLarge ? .title : .title3)
            Text(AgroFormat.temperature(current.temperature))
                .font(.system(size: family == .systemLarge ? 40 : 22, weight: .light, design: .rounded))
                .lineLimit(1)
                .widgetAccentable()
        }
    }

    /// Six heures, après l'heure en cours.
    private var colonnes: some View {
        HStack(spacing: 0) {
            ForEach(Array(entry.hours.dropFirst().prefix(6))) { heure in
                VStack(spacing: 4) {
                    Text(AgroFormat.hour(heure.time, in: entry.timeZone))
                        .font(.system(size: 11, weight: .semibold))
                        .opacity(0.8)
                        .lineLimit(1)
                        .minimumScaleFactor(0.7)
                    Image(systemName: WeatherCondition.forCode(heure.weatherCode).icon.symbolName(isDay: heure.isDay))
                        .symbolRenderingMode(.multicolor)
                        .font(.system(size: 18))
                        .frame(height: 22)
                    Text(AgroFormat.temperature(heure.temperature))
                        .font(.system(size: 15, weight: .medium, design: .rounded))
                        .monospacedDigit()
                        .lineLimit(1)
                    Text(heure.precipitationProbability >= 30 ? AgroFormat.percent(heure.precipitationProbability) : " ")
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundStyle(bleuPluie)
                        .lineLimit(1)
                        .minimumScaleFactor(0.7)
                }
                .frame(maxWidth: .infinity)
            }
        }
        .padding(.vertical, 4)
        .background(Color.white.opacity(0.12), in: RoundedRectangle(cornerRadius: 14))
    }

    private var bornes: ClosedRange<Double> { Semaine.bornes(entry.days.dropFirst().prefix(4)) }
}

// MARK: - La semaine

/// « Et cette semaine ? » — les jours, du plus frais au plus chaud, comme la
/// liste de l'application.
struct SemaineWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KliimaSemaineWidget", provider: WeatherTimelineProvider()) { entry in
            SemaineWidgetView(entry: entry)
                .containerBackground(for: .widget) { CielDuWidget(current: entry.current) }
        }
        .configurationDisplayName(Localized.text("widget.week.name"))
        .description(Localized.text("widget.week.description"))
        .supportedFamilies([.systemMedium, .systemLarge])
    }
}

/// Les bornes d'une suite de jours, pour placer les barres sur une même
/// échelle.
private enum Semaine {
    static func bornes<S: Sequence>(_ jours: S) -> ClosedRange<Double> where S.Element == DailySample {
        let liste = Array(jours)
        let bas = liste.map(\.temperatureMin).min() ?? 0
        let haut = liste.map(\.temperatureMax).max() ?? 1
        return bas...max(haut, bas + 1)
    }
}

struct SemaineWidgetView: View {
    @Environment(\.widgetFamily) private var family
    let entry: WeatherEntry

    private var jours: [DailySample] { Array(entry.days.prefix(family == .systemLarge ? 7 : 4)) }

    var body: some View {
        Group {
            if jours.isEmpty {
                Indisponible(ville: entry.parcelleName)
            } else {
                VStack(alignment: .leading, spacing: family == .systemLarge ? 8 : 4) {
                    HStack {
                        Titre(symbole: "calendar", texte: Localized.text("widget.week.name"))
                        Spacer()
                        Text(entry.parcelleName)
                            .font(.caption2.weight(.semibold))
                            .opacity(0.85)
                            .lineLimit(1)
                    }
                    if family == .systemLarge { Spacer(minLength: 0) }
                    ForEach(Array(jours.enumerated()), id: \.element.id) { rang, jour in
                        LigneDeJour(jour: jour, bornes: Semaine.bornes(jours), timeZone: entry.timeZone, aujourdhui: rang == 0)
                        if family == .systemLarge && rang < jours.count - 1 {
                            Divider().overlay(Color.white.opacity(0.15))
                        }
                    }
                    Spacer(minLength: 0)
                }
            }
        }
        .foregroundStyle(.white)
    }
}

/// Une journée : son nom, son ciel, son risque de pluie, et sa barre du plus
/// frais au plus chaud.
private struct LigneDeJour: View {
    let jour: DailySample
    let bornes: ClosedRange<Double>
    let timeZone: TimeZone
    let aujourdhui: Bool

    var body: some View {
        HStack(spacing: 6) {
            Text(aujourdhui ? Localized.text("daily.today") : AgroFormat.weekday(jour.date, in: timeZone))
                .font(.caption.weight(.semibold))
                .lineLimit(1)
                .minimumScaleFactor(0.7)
                .frame(width: 38, alignment: .leading)
            Image(systemName: WeatherCondition.forCode(jour.weatherCode).icon.symbolName(isDay: true))
                .symbolRenderingMode(.multicolor)
                .font(.system(size: 15))
                .frame(width: 22)
            Text(jour.precipitationProbabilityMax >= 30 ? AgroFormat.percent(jour.precipitationProbabilityMax) : "")
                .font(.system(size: 10, weight: .semibold))
                .foregroundStyle(bleuPluie)
                .lineLimit(1)
                .minimumScaleFactor(0.7)
                .frame(width: 30, alignment: .leading)
            Text(AgroFormat.temperature(jour.temperatureMin))
                .font(.caption.monospacedDigit())
                .opacity(0.75)
                .frame(width: 30, alignment: .trailing)
            GeometryReader { geo in
                let etendue = bornes.upperBound - bornes.lowerBound
                let debut = (jour.temperatureMin - bornes.lowerBound) / etendue
                let fin = (jour.temperatureMax - bornes.lowerBound) / etendue
                ZStack(alignment: .leading) {
                    Capsule().fill(Color.white.opacity(0.18))
                    Capsule()
                        .fill(LinearGradient(
                            colors: [Color(red: 0.55, green: 0.80, blue: 1.0), Color(red: 1.0, green: 0.78, blue: 0.40)],
                            startPoint: .leading, endPoint: .trailing
                        ))
                        .frame(width: max(6, geo.size.width * (fin - debut)))
                        .offset(x: geo.size.width * debut)
                }
            }
            .frame(height: 5)
            Text(AgroFormat.temperature(jour.temperatureMax))
                .font(.caption.weight(.semibold).monospacedDigit())
                .frame(width: 30, alignment: .leading)
        }
    }
}

// MARK: - Pour sortir

/// « Qu'est-ce que je mets ? » — le ressenti, le vent, l'UV, et ce qu'il faut
/// emporter.
struct DehorsWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "KliimaDehorsWidget", provider: WeatherTimelineProvider()) { entry in
            DehorsWidgetView(entry: entry)
                .containerBackground(for: .widget) { CielDuWidget(current: entry.current) }
        }
        .configurationDisplayName(Localized.text("widget.outside.name"))
        .description(Localized.text("widget.outside.description"))
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

struct DehorsWidgetView: View {
    @Environment(\.widgetFamily) private var family
    let entry: WeatherEntry

    var body: some View {
        Group {
            if let current = entry.current {
                if family == .systemMedium {
                    HStack(alignment: .top, spacing: 14) {
                        mesures(current)
                        Emporter(conseils: Ville.conseils(entry.hours), limite: 4)
                            .frame(width: 128, alignment: .leading)
                    }
                } else {
                    VStack(alignment: .leading, spacing: 4) {
                        mesures(current)
                        Spacer(minLength: 0)
                        // Ce qu'il faut emporter, en symboles seulement.
                        let conseils = Ville.conseils(entry.hours)
                        if conseils.isEmpty {
                            Label(Localized.text("advice.title"), systemImage: "checkmark.circle")
                                .font(.caption2.weight(.semibold))
                                .opacity(0.8)
                        } else {
                            HStack(spacing: 8) {
                                ForEach(conseils.prefix(4), id: \.self) { conseil in
                                    Image(systemName: conseil.symbolName)
                                        .symbolRenderingMode(.multicolor)
                                        .font(.system(size: 16))
                                }
                            }
                        }
                    }
                }
            } else {
                Indisponible(ville: entry.parcelleName)
            }
        }
        .foregroundStyle(.white)
    }

    /// Le ressenti en grand, puis le vent et l'UV.
    private func mesures(_ current: CurrentSample) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            Titre(symbole: "figure.walk", texte: Localized.text("tile.feelsLike"))
            Text(AgroFormat.temperature(current.apparentTemperature))
                .font(.system(size: 38, weight: .light, design: .rounded))
                .lineLimit(1)
                .minimumScaleFactor(0.6)
                .widgetAccentable()
            Label(
                "\(AgroFormat.unit(current.windSpeed, "km/h", decimals: 0)) · \(AgroFormat.unit(current.windGusts, "km/h", decimals: 0))",
                systemImage: "wind"
            )
            .font(.caption2.weight(.semibold))
            .lineLimit(1)
            .minimumScaleFactor(0.7)
            if let jour = entry.today {
                Label(
                    "\(Localized.text("tile.uv")) \(AgroFormat.decimal(jour.uvIndexMax, decimals: 0)) · \(Ville.niveauUv(jour.uvIndexMax).label)",
                    systemImage: "sun.max.fill"
                )
                .font(.caption2.weight(.semibold))
                .lineLimit(1)
                .minimumScaleFactor(0.7)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}
