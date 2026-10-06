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

    /// Les trois ciels en clair : un vrai ciel, pas un blanc bleuté. La
    /// première version, presque blanche, effaçait le dégradé et laissait les
    /// cartes de verre se fondre dans le fond. L'encre bleu nuit garde sur ces
    /// teintes un contraste d’au moins 5 pour 1, en haut comme en bas.
    private var clair: [Color] {
        guard isDay else {
            // Nuit, vue en clair : un crépuscule lavande.
            return [
                Color(red: 0.537, green: 0.565, blue: 0.784),
                Color(red: 0.659, green: 0.682, blue: 0.851),
                Color(red: 0.808, green: 0.820, blue: 0.918),
            ]
        }
        if weatherCode >= 45 {
            // Ciel couvert : un gris bleuté franc.
            return [
                Color(red: 0.561, green: 0.639, blue: 0.722),
                Color(red: 0.690, green: 0.757, blue: 0.824),
                Color(red: 0.839, green: 0.874, blue: 0.910),
            ]
        }
        // Ciel dégagé : un azur.
        return [
            Color(red: 0.384, green: 0.643, blue: 0.886),
            Color(red: 0.580, green: 0.773, blue: 0.937),
            Color(red: 0.800, green: 0.894, blue: 0.976),
        ]
    }

    /// Les trois ciels en sombre : sombres pour de bon. L'ancienne nuit
    /// commençait en ardoise moyenne ; elle part maintenant d'un bleu nuit et
    /// finit presque noire, comme le thème sombre du système.
    private var sombre: [Color] {
        guard isDay else {
            return [
                Color(red: 0.067, green: 0.110, blue: 0.180),
                Color(red: 0.039, green: 0.071, blue: 0.125),
                Color(red: 0.016, green: 0.031, blue: 0.067),
            ]
        }
        if weatherCode >= 45 {
            return [
                Color(red: 0.165, green: 0.208, blue: 0.259),
                Color(red: 0.110, green: 0.141, blue: 0.180),
                Color(red: 0.055, green: 0.071, blue: 0.098),
            ]
        }
        return [
            Color(red: 0.102, green: 0.267, blue: 0.459),
            Color(red: 0.067, green: 0.176, blue: 0.318),
            Color(red: 0.027, green: 0.075, blue: 0.157),
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
