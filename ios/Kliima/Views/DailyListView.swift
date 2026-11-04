import SwiftUI

/// Liste des jours, avec la barre d'amplitude thermique de la semaine.
struct DailyListView: View {
    let days: [DailySample]
    /// Température du moment, repérée sur la barre d'aujourd'hui.
    let currentTemperature: Double
    let timeZone: TimeZone

    /// Les colonnes grandissent avec la taille de texte choisie.
    ///
    /// Figées en points, elles allaient pour le réglage par défaut et lui
    /// seul : un cran au-dessus, « 18° » ne tenait plus dans ses 34 points et
    /// le degré passait seul à la ligne, comme le point de « Sam. ». Même
    /// remède que le bandeau horaire : `ScaledMetric` fait grandir la colonne
    /// avec le texte, et une seule ligne reste une seule ligne.
    @ScaledMetric(relativeTo: .body) private var largeurJour: CGFloat = 50
    @ScaledMetric(relativeTo: .body) private var largeurCiel: CGFloat = 46
    @ScaledMetric(relativeTo: .body) private var largeurTemperature: CGFloat = 38
    @ScaledMetric(relativeTo: .body) private var tailleIcone: CGFloat = 18

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            CardLabel(text: Localized.text("daily.title"))

            VStack(spacing: 0) {
                ForEach(Array(days.enumerated()), id: \.element.id) { index, day in
                    if index > 0 {
                        Divider().overlay(Color.filet)
                    }
                    row(for: day, isToday: index == 0, rang: index)
                }
            }
        }
        .cardBackground()
    }

    // Toutes les barres se lisent sur la même échelle : celle de la semaine.
    private var weekLow: Double { days.map(\.temperatureMin).min() ?? 0 }
    private var weekHigh: Double { days.map(\.temperatureMax).max() ?? 1 }
    private var span: Double { max(weekHigh - weekLow, 1) }

    private func row(for day: DailySample, isToday: Bool, rang: Int) -> some View {
        let condition = WeatherCondition.forCode(day.weatherCode)

        return HStack(spacing: 10) {
            Text(isToday ? Localized.text("daily.today") : AgroFormat.weekday(day.date, in: timeZone))
                .font(.body.weight(.medium))
                .lineLimit(1)
                .minimumScaleFactor(0.75)
                .frame(width: largeurJour, alignment: .leading)
                // « Auj. » se lit « auj point » : VoiceOver dit le mot.
                .accessibilityLabel(isToday ? Localized.text("daily.today.a11y") : AgroFormat.weekday(day.date, in: timeZone))

            VStack(spacing: 0) {
                Image(systemName: condition.icon.symbolName(isDay: true))
                    .symbolRenderingMode(.multicolor)
                    .font(.system(size: tailleIcone))
                    // Sans libellé, VoiceOver lisait le nom anglais du symbole.
                    .accessibilityLabel(condition.label(isDay: true))
                Text(day.precipitationProbabilityMax >= 10
                     ? AgroFormat.percent(day.precipitationProbabilityMax)
                     : " ")
                    .font(.caption2.weight(.semibold))
                    // L'azur pâle, fixe, s'effaçait sur le ciel clair : le bleu de
                    // la pluie suit le thème.
                    .foregroundStyle(Color.bleuPluie)
                    .lineLimit(1)
                    .minimumScaleFactor(0.75)
                    .accessibilityLabel(Localized.text("rain.chance.a11y", AgroFormat.percent(day.precipitationProbabilityMax)))
                    .accessibilityHidden(day.precipitationProbabilityMax < 10)
            }
            .frame(width: largeurCiel)

            Text(AgroFormat.temperature(day.temperatureMin))
                .foregroundStyle(Color.encreDouce)
                .lineLimit(1)
                .minimumScaleFactor(0.75)
                .frame(width: largeurTemperature, alignment: .trailing)
                // Deux degrés côte à côte ne disent pas lequel est le minimum.
                .accessibilityLabel(Localized.text("range.min.a11y", AgroFormat.temperature(day.temperatureMin)))

            temperatureBar(for: day, isToday: isToday, rang: rang)
                .accessibilityHidden(true)

            Text(AgroFormat.temperature(day.temperatureMax))
                .lineLimit(1)
                .minimumScaleFactor(0.75)
                .frame(width: largeurTemperature, alignment: .trailing)
                .accessibilityLabel(Localized.text("range.max.a11y", AgroFormat.temperature(day.temperatureMax)))
        }
        .font(.body)
        .monospacedDigit()
        .padding(.vertical, 9)
        .accessibilityElement(children: .combine)
    }

    private func temperatureBar(for day: DailySample, isToday: Bool, rang: Int) -> some View {
        GeometryReader { geometry in
            let width = geometry.size.width
            let length = max((day.temperatureMax - day.temperatureMin) / span, 0.06)
            // La longueur minimale ne pousse pas la barre hors du rail : le
            // jour le plus chaud, presque sans amplitude, la faisait dépasser
            // à droite. Elle recule alors d'autant.
            let start = min((day.temperatureMin - weekLow) / span, 1 - length)

            ZStack(alignment: .leading) {
                Capsule()
                    .fill(Color.encre.opacity(0.2))
                Capsule()
                    .fill(LinearGradient(
                        colors: [
                            Color(red: 0.435, green: 0.761, blue: 0.910),
                            Color(red: 0.941, green: 0.757, blue: 0.294),
                            Color(red: 0.937, green: 0.541, blue: 0.353),
                        ],
                        startPoint: .leading,
                        endPoint: .trailing
                    ))
                    .frame(width: width * length)
                    // L'amplitude s'étire depuis le minimum du jour.
                    .pousse(rang + 4, horizontal: true)
                    .offset(x: width * start)

                if isToday {
                    // Le point court à l'intérieur du rail, comme celui des
                    // jauges : centré sur la position, il débordait de sa
                    // moitié quand il faisait le plus froid ou le plus chaud
                    // de la semaine.
                    let course = max(width - 7, 0)
                    let position = min(max((currentTemperature - weekLow) / span, 0), 1)
                    Circle()
                        .fill(Color.encre)
                        .frame(width: 7, height: 7)
                        .offset(x: course * position)
                }
            }
            .frame(height: 4)
            .frame(maxHeight: .infinity)
        }
        .frame(height: 12)
    }
}
