import SwiftUI

/// Le fond suit le ciel : nuit, journée couverte ou journée dégagée.
struct SkyBackground: View {
    let isDay: Bool
    let weatherCode: Int

    var body: some View {
        LinearGradient(colors: colors, startPoint: .top, endPoint: .bottom)
            .ignoresSafeArea()
    }

    private var colors: [Color] {
        guard isDay else {
            return [
                Color(red: 0.235, green: 0.333, blue: 0.431),
                Color(red: 0.169, green: 0.239, blue: 0.322),
                Color(red: 0.114, green: 0.157, blue: 0.212),
            ]
        }
        if weatherCode >= 45 {
            return [
                Color(red: 0.420, green: 0.518, blue: 0.608),
                Color(red: 0.302, green: 0.388, blue: 0.475),
                Color(red: 0.200, green: 0.278, blue: 0.361),
            ]
        }
        return [
            Color(red: 0.290, green: 0.565, blue: 0.851),
            Color(red: 0.227, green: 0.447, blue: 0.706),
            Color(red: 0.169, green: 0.318, blue: 0.514),
        ]
    }
}

/// Style commun des cartes : un verre dépoli sur le ciel.
struct CardBackground: ViewModifier {
    func body(content: Content) -> some View {
        content
            .padding(14)
            .background(.ultraThinMaterial.opacity(0.6), in: RoundedRectangle(cornerRadius: 18))
            .overlay(
                RoundedRectangle(cornerRadius: 18)
                    .strokeBorder(Color.white.opacity(0.14), lineWidth: 1)
            )
    }
}

extension View {
    func cardBackground() -> some View { modifier(CardBackground()) }
}

/// Intitulé en petites capitales, comme les cartes du système.
struct CardLabel: View {
    let text: String

    var body: some View {
        Text(text.uppercased())
            .font(.caption2.weight(.semibold))
            .kerning(0.6)
            .foregroundStyle(.white.opacity(0.62))
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}
