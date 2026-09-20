import SwiftUI

/// Carte pleine largeur des conditions de pulvérisation : la prochaine fenêtre
/// en clair, puis une frise des 24 prochaines heures.
struct SprayCardView: View {
    let hours: [HourlySample]
    let nextSpray: SprayOpportunity?
    let timeZone: TimeZone
    /// Vrai si une activité en direct suit déjà cette fenêtre.
    var isFollowing = false
    /// Faux si l'appareil ou les réglages refusent les activités en direct.
    var canFollow = false
    var onFollow: () -> Void = {}

    private var windows: [SprayWindow] {
        Array(AgroIndicators.sprayWindows(hours).prefix(24))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            CardLabel(text: Localized.text("spray.title"))

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
                Text(Localized.text("spray.now"))
                Spacer()
                Text(Localized.text("spray.plus12"))
                Spacer()
                Text(Localized.text("spray.plus24"))
            }
            .font(.system(size: 11))
            .foregroundStyle(.white.opacity(0.62))
            .padding(.top, 5)

            if canFollow, nextSpray != nil {
                Button(action: onFollow) {
                    Label(
                        Localized.text(isFollowing ? "activity.stop" : "activity.follow"),
                        systemImage: isFollowing ? "bell.slash" : "bell"
                    )
                    .font(.subheadline)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
                    .background(Color.white.opacity(0.14), in: RoundedRectangle(cornerRadius: 10))
                }
                .buttonStyle(.plain)
                .padding(.top, 10)
            }
        }
        .cardBackground()
    }

    private var headline: String {
        guard let nextSpray else { return Localized.text("spray.none") }
        return "\(AgroFormat.weekdayHour(nextSpray.start, in: timeZone)) → \(AgroFormat.hour(nextSpray.end, in: timeZone))"
    }

    private var caption: String {
        if let nextSpray {
            return Localized.text("spray.score", String(nextSpray.score))
        }
        // Le premier motif de blocage résume la situation.
        let blocked = windows.first { $0.verdict == .defavorable && !$0.blockers.isEmpty }
        guard let blocker = blocked?.blockers.first else { return Localized.text("spray.unsuitable") }
        return Localized.text("spray.mainBlocker", blocker.text.lowercased())
    }

    private func color(for verdict: SprayVerdict) -> Color {
        switch verdict {
        case .favorable: return Color(red: 0.494, green: 0.816, blue: 0.478)
        case .acceptable: return Color(red: 0.941, green: 0.757, blue: 0.294)
        case .defavorable: return Color.white.opacity(0.22)
        }
    }
}
