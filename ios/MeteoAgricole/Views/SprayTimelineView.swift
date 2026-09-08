import SwiftUI

/// Frise horaire des conditions de pulvérisation sur les 48 prochaines heures.
struct SprayTimelineView: View {
    let hours: [HourlySample]
    let timeZone: TimeZone

    @State private var selection: SprayWindow?

    private var windows: [SprayWindow] {
        Array(AgroIndicators.sprayWindows(hours).prefix(48))
    }

    private var detail: SprayWindow? {
        selection ?? windows.first
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Conditions de pulvérisation — 48 h")
                    .font(.headline)
                Spacer()
            }

            legend

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(alignment: .bottom, spacing: 4) {
                    ForEach(windows) { window in
                        bar(for: window)
                    }
                }
                .padding(.vertical, 2)
            }

            if let detail {
                detailPanel(detail)
            }
        }
        .padding(16)
        .background(Color(uiColor: .secondarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 16))
    }

    private var legend: some View {
        HStack(spacing: 12) {
            legendItem(.good, "favorable")
            legendItem(.warn, "acceptable")
            legendItem(.bad, "défavorable")
        }
        .font(.caption2)
        .foregroundStyle(.secondary)
    }

    private func legendItem(_ tone: Tone, _ label: String) -> some View {
        HStack(spacing: 4) {
            Circle().fill(tone.color).frame(width: 7, height: 7)
            Text(label)
        }
    }

    private func bar(for window: SprayWindow) -> some View {
        let isSelected = detail?.time == window.time
        return VStack(spacing: 5) {
            ZStack(alignment: .bottom) {
                RoundedRectangle(cornerRadius: 5)
                    .fill(Color.secondary.opacity(0.12))
                RoundedRectangle(cornerRadius: 5)
                    .fill(tone(for: window.verdict).color)
                    // Un minimum de 8 % garde la barre visible même à score nul.
                    .frame(height: max(CGFloat(window.score), 8) / 100 * 92)
            }
            .frame(width: 20, height: 92)

            Text(hourLabel(window.time))
                .font(.system(size: 10))
                .monospacedDigit()
                .foregroundStyle(.secondary)
        }
        .padding(3)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(isSelected ? Color.secondary.opacity(0.14) : .clear)
        )
        .contentShape(Rectangle())
        .onTapGesture { selection = window }
        .accessibilityLabel("\(fullLabel(window.time)) : \(window.verdict.label), score \(window.score)")
    }

    private func detailPanel(_ window: SprayWindow) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("\(fullLabel(window.time)) — score \(window.score)/100")
                .font(.subheadline.weight(.semibold))

            if window.blockers.isEmpty {
                Label("Toutes les conditions sont réunies pour traiter.", systemImage: "checkmark.circle")
                    .font(.footnote)
                    .foregroundStyle(Tone.good.color)
            } else {
                ForEach(window.blockers, id: \.self) { blocker in
                    Label(blocker, systemImage: "exclamationmark.triangle")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(Color(uiColor: .tertiarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 10))
    }

    private func tone(for verdict: SprayVerdict) -> Tone {
        switch verdict {
        case .favorable: return .good
        case .acceptable: return .warn
        case .defavorable: return .bad
        }
    }

    private func hourLabel(_ date: Date) -> String {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = timeZone
        return String(calendar.component(.hour, from: date))
    }

    private func fullLabel(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "fr_FR")
        formatter.timeZone = timeZone
        formatter.dateFormat = "EEEE d MMMM, HH 'h'"
        return formatter.string(from: date)
    }
}
