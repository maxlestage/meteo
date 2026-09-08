import SwiftUI

/// Liste des jours, avec la barre d'amplitude thermique de la semaine.
struct DailyListView: View {
    let days: [DailySample]
    /// Température du moment, repérée sur la barre d'aujourd'hui.
    let currentTemperature: Double
    let timeZone: TimeZone

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            CardLabel(text: "Prévision sur 7 jours")

            VStack(spacing: 0) {
                ForEach(Array(days.enumerated()), id: \.element.id) { index, day in
                    if index > 0 {
                        Divider().overlay(Color.white.opacity(0.14))
                    }
                    row(for: day, isToday: index == 0)
                }
            }
        }
        .cardBackground()
    }

    // Toutes les barres se lisent sur la même échelle : celle de la semaine.
    private var weekLow: Double { days.map(\.temperatureMin).min() ?? 0 }
    private var weekHigh: Double { days.map(\.temperatureMax).max() ?? 1 }
    private var span: Double { max(weekHigh - weekLow, 1) }

    private func row(for day: DailySample, isToday: Bool) -> some View {
        let condition = WeatherCondition.forCode(day.weatherCode)

        return HStack(spacing: 10) {
            Text(isToday ? "Auj." : AgroFormat.weekday(day.date, in: timeZone))
                .font(.body.weight(.medium))
                .frame(width: 48, alignment: .leading)

            VStack(spacing: 0) {
                Image(systemName: condition.icon.symbolName(isDay: true))
                    .symbolRenderingMode(.multicolor)
                    .font(.system(size: 18))
                Text(day.precipitationProbabilityMax >= 10
                     ? "\(Int(day.precipitationProbabilityMax.rounded())) %"
                     : " ")
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(Color(red: 0.498, green: 0.816, blue: 0.961))
            }
            .frame(width: 46)

            Text("\(Int(day.temperatureMin.rounded()))°")
                .foregroundStyle(.white.opacity(0.62))
                .frame(width: 34, alignment: .trailing)

            temperatureBar(for: day, isToday: isToday)

            Text("\(Int(day.temperatureMax.rounded()))°")
                .frame(width: 34, alignment: .trailing)
        }
        .font(.body)
        .monospacedDigit()
        .padding(.vertical, 9)
        .accessibilityElement(children: .combine)
    }

    private func temperatureBar(for day: DailySample, isToday: Bool) -> some View {
        GeometryReader { geometry in
            let width = geometry.size.width
            let start = (day.temperatureMin - weekLow) / span
            let length = max((day.temperatureMax - day.temperatureMin) / span, 0.06)

            ZStack(alignment: .leading) {
                Capsule()
                    .fill(Color.white.opacity(0.2))
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
                    .offset(x: width * start)

                if isToday {
                    let position = min(max((currentTemperature - weekLow) / span, 0), 1)
                    Circle()
                        .fill(.white)
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
