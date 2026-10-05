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

            VStack(spacing: 0) {
                Image(systemName: condition.icon.symbolName(isDay: true))
                    .symbolRenderingMode(.multicolor)
                    .font(.system(size: tailleIcone))
                Text(day.precipitationProbabilityMax >= 10
                     ? AgroFormat.percent(day.precipitationProbabilityMax)
                     : " ")
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(Color(red: 0.498, green: 0.816, blue: 0.961))
                    .lineLimit(1)
                    .minimumScaleFactor(0.75)
            }
            .frame(width: largeurCiel)

            Text(AgroFormat.temperature(day.temperatureMin))
                .foregroundStyle(Color.encreDouce)
                .lineLimit(1)
                .minimumScaleFactor(0.75)
                .frame(width: largeurTemperature, alignment: .trailing)

            temperatureBar(for: day, isToday: isToday, rang: rang)

            Text(AgroFormat.temperature(day.temperatureMax))
                .lineLimit(1)
                .minimumScaleFactor(0.75)
                .frame(width: largeurTemperature, alignment: .trailing)
        }
        .font(.body)
        .monospacedDigit()
        .padding(.vertical, 9)
        .accessibilityElement(children: .combine)
    }

    private func temperatureBar(for day: DailySample, isToday: Bool, rang: Int) -> some View {
        GeometryReader { geometry in
            let width = geometry.size.width
            let start = (day.temperatureMin - weekLow) / span
            let length = max((day.temperatureMax - day.temperatureMin) / span, 0.06)

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
                    let position = min(max((currentTemperature - weekLow) / span, 0), 1)
                    Circle()
                        .fill(Color.encre)
                        .frame(width: 7, height: 7)
                        .offset(x: width * position - 3.5)
                }
            }
            .frame(height: 4)
            .frame(maxHeight: .infinity)
        }
        .frame(height: 12)
    }
}
