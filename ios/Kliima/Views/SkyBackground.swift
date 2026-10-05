import SwiftUI

/// L'encre de l'interface : blanche sur un ciel sombre, bleu nuit sur un ciel clair.
///
/// Tout était écrit en blanc, et l'écran forcé en sombre pour que ça reste
/// lisible — de jour comme de nuit, quel que soit le réglage du téléphone. Ces
/// couleurs suivent maintenant le thème du système, comme le reste d'iOS.
extension Color {
    static let encre = Color(uiColor: UIColor { trait in
        trait.userInterfaceStyle == .dark
            ? .white
            : UIColor(red: 0.071, green: 0.129, blue: 0.220, alpha: 1)
    })
    /// Pour ce qui accompagne : intitulés, unités, compléments.
    static let encreDouce = Color.encre.opacity(0.62)
    /// Pour les filets et les séparations.
    static let filet = Color.encre.opacity(0.14)
}

/// Le fond suit le ciel : nuit, journée couverte ou journée dégagée — et le
/// thème du système, clair ou sombre. Par-dessus, le ciel vivant
/// (`CielVivant`) dessine le temps qu'il fait.
struct SkyBackground: View {
    let isDay: Bool
    let weatherCode: Int

    @Environment(\.colorScheme) private var scheme

    var body: some View {
        ZStack {
            LinearGradient(colors: colors, startPoint: .top, endPoint: .bottom)
                .ignoresSafeArea()
                // Un changement de ville ne fait pas sauter la couleur.
                .animation(.easeInOut(duration: 1.2), value: colors)
            // Le temps qu'il fait, par-dessus : soleil ou étoiles, nuages,
            // pluie, neige, éclairs, brouillard.
            CielVivant(isDay: isDay, weatherCode: weatherCode)
        }
    }

    private var colors: [Color] {
        scheme == .dark ? sombre : clair
    }

    /// Les mêmes trois ciels, en clair : l'encre bleu nuit y garde son contraste.
    private var clair: [Color] {
        guard isDay else {
            return [
                Color(red: 0.800, green: 0.808, blue: 0.918),
                Color(red: 0.871, green: 0.875, blue: 0.949),
                Color(red: 0.941, green: 0.941, blue: 0.980),
            ]
        }
        if weatherCode >= 45 {
            return [
                Color(red: 0.820, green: 0.851, blue: 0.890),
                Color(red: 0.882, green: 0.902, blue: 0.929),
                Color(red: 0.949, green: 0.957, blue: 0.969),
            ]
        }
        return [
            Color(red: 0.761, green: 0.871, blue: 0.973),
            Color(red: 0.859, green: 0.929, blue: 0.992),
            Color(red: 0.953, green: 0.973, blue: 1.000),
        ]
    }

    private var sombre: [Color] {
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
                    .strokeBorder(Color.filet, lineWidth: 1)
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
            .foregroundStyle(Color.encreDouce)
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}
