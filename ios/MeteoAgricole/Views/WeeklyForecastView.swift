import SwiftUI

/// Cumuls journaliers orientés conduite de culture : pluie, ET0, bilan, gel.
struct WeeklyForecastView: View {
    let days: [DailySample]
    let timeZone: TimeZone

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Semaine agronomique")
                .font(.headline)

            VStack(spacing: 0) {
                ForEach(Array(days.enumerated()), id: \.element.id) { index, day in
                    row(for: day)
                    if index < days.count - 1 {
                        Divider()
                    }
                }
            }
        }
        .padding(16)
        .background(Color(uiColor: .secondarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 16))
    }

    private func row(for day: DailySample) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 12) {
            Text(dayLabel(day.date))
                .font(.subheadline.weight(.medium))
                .frame(width: 76, alignment: .leading)

            Text("\(Int(day.temperatureMin.rounded()))° / \(Int(day.temperatureMax.rounded()))°")
                .font(.subheadline)
                .monospacedDigit()
                .frame(width: 74, alignment: .leading)

            VStack(alignment: .leading, spacing: 2) {
                Label(String(format: "%.1f mm", day.precipitationSum), systemImage: "cloud.rain")
                Label(String(format: "ET0 %.1f mm", day.et0Sum), systemImage: "sun.max")
            }
            .font(.caption)
            .foregroundStyle(.secondary)
            .labelStyle(.titleAndIcon)

            Spacer(minLength: 0)

            VStack(alignment: .trailing, spacing: 2) {
                Text(String(format: "%@%.1f", day.balance > 0 ? "+" : "", day.balance))
                    .font(.subheadline.weight(.semibold))
                    .monospacedDigit()
                    .foregroundStyle(day.balance < 0 ? Tone.bad.color : Tone.good.color)
                Text("\(Int(AgroIndicators.growingDegreeDays(temperatureMin: day.temperatureMin, temperatureMax: day.temperatureMax).rounded())) °C·j")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 10)
        .accessibilityElement(children: .combine)
    }

    private func dayLabel(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "fr_FR")
        formatter.timeZone = timeZone
        formatter.dateFormat = "EEE d MMM"
        return formatter.string(from: date)
    }
}
