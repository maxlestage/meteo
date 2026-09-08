import SwiftUI

/// Carte pleine largeur des conditions de pulvérisation : la prochaine fenêtre
/// en clair, puis une frise des 24 prochaines heures.
struct SprayCardView: View {
    let hours: [HourlySample]
    let nextSpray: SprayOpportunity?
    let timeZone: TimeZone

    private var windows: [SprayWindow] {
        Array(AgroIndicators.sprayWindows(hours).prefix(24))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            CardLabel(text: "Fenêtre de traitement")

            Text(headline)
                .font(.system(size: 26, weight: .regular))

            Text(caption)
                .font(.subheadline)
                .foregroundStyle(.white.opacity(0.62))
                .padding(.bottom, 8)

            HStack(spacing: 2) {
                ForEach(windows) { window in
                    RoundedRectangle(cornerRadius: 3)
                        .fill(color(for: window.verdict))
                        .accessibilityLabel(
                            "\(AgroFormat.hour(window.time, in: timeZone)) : \(window.verdict.label)"
                        )
                }
            }
            .frame(height: 26)

            HStack {
                Text("Maintenant")
                Spacer()
                Text("+12 h")
                Spacer()
                Text("+24 h")
            }
            .font(.system(size: 11))
            .foregroundStyle(.white.opacity(0.62))
            .padding(.top, 5)
        }
        .cardBackground()
    }

    private var headline: String {
        guard let nextSpray else { return "Aucune fenêtre sur 7 jours" }
        return "\(AgroFormat.weekdayHour(nextSpray.start, in: timeZone)) → \(AgroFormat.hour(nextSpray.end, in: timeZone))"
    }

    private var caption: String {
        if let nextSpray {
            return "Score \(nextSpray.score)/100 sur la plage"
        }
        // Le premier motif de blocage résume la situation.
        let blocked = windows.first { $0.verdict == .defavorable && !$0.blockers.isEmpty }
        guard let blocker = blocked?.blockers.first else { return "Conditions défavorables" }
        return "Blocage principal : \(blocker.lowercased())"
    }

    private func color(for verdict: SprayVerdict) -> Color {
        switch verdict {
        case .favorable: return Color(red: 0.494, green: 0.816, blue: 0.478)
        case .acceptable: return Color(red: 0.941, green: 0.757, blue: 0.294)
        case .defavorable: return Color.white.opacity(0.22)
        }
    }
}
