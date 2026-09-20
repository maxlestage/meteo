import SwiftUI

/// Bandeau horaire sur 24 h : heure, temps, probabilité de pluie, température.
struct HourlyStripView: View {
    let hours: [HourlySample]
    /// Conditions observées, affichées sur la colonne « Maintenant ».
    let current: CurrentSample
    let timeZone: TimeZone

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            CardLabel(text: Localized.text("hourly.title"))

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 4) {
                    ForEach(Array(hours.prefix(24).enumerated()), id: \.element.id) { index, hour in
                        column(for: hour, isFirst: index == 0)
                    }
                }
            }
        }
        .cardBackground()
    }

    private func column(for hour: HourlySample, isFirst: Bool) -> some View {
        // La première colonne montre le relevé courant, pas la prévision de
        // l'heure déjà entamée.
        let code = isFirst ? current.weatherCode : hour.weatherCode
        let condition = WeatherCondition.forCode(code)
        let isDay = isFirst ? current.isDay : hour.isDay
        let temperature = isFirst ? current.temperature : hour.temperature

        return VStack(spacing: 7) {
            Text(isFirst ? Localized.text("hourly.now") : AgroFormat.hour(hour.time, in: timeZone))
                .font(.subheadline.weight(.semibold))

            Image(systemName: condition.icon.symbolName(isDay: isDay))
                .symbolRenderingMode(.multicolor)
                .font(.system(size: 20))
                .frame(height: 24)

            Text(hour.precipitationProbability >= 10
                 ? AgroFormat.percent(hour.precipitationProbability)
                 : " ")
                .font(.caption2.weight(.semibold))
                .foregroundStyle(Color(red: 0.498, green: 0.816, blue: 0.961))

            Text(AgroFormat.temperature(temperature))
                .font(.title3)
        }
        .frame(width: 58)
        .accessibilityElement(children: .combine)
    }
}
