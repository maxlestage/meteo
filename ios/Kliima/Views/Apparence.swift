import SwiftUI
import UIKit

/// L'apparence de l'application : celle du téléphone, ou claire, ou sombre.
///
/// Par défaut, Kliima ‣ suit le réglage du système — clair le jour, sombre le
/// soir si l'iPhone bascule tout seul. Le sélecteur de la barre d'outils
/// permet d'imposer l'un ou l'autre, pour l'application seulement.
///
/// Le choix est posé sur les fenêtres (`overrideUserInterfaceStyle`) plutôt
/// que par `preferredColorScheme` : revenir de « sombre » à « automatique » ne
/// rafraîchissait pas toujours l'écran avec ce dernier, et les feuilles
/// ouvertes par-dessus gardaient l'ancienne apparence. Les couleurs de
/// l'interface sont dynamiques (`Color.encre`) : elles suivent d'elles-mêmes.
enum Apparence: String, CaseIterable, Identifiable {
    case automatique, clair, sombre

    /// Clé du réglage, gardé d'un lancement à l'autre.
    static let cle = "kliima.apparence"

    var id: String { rawValue }

    var libelle: String {
        switch self {
        case .automatique: return Localized.text("theme.auto")
        case .clair: return Localized.text("theme.light")
        case .sombre: return Localized.text("theme.dark")
        }
    }

    var symbole: String {
        switch self {
        case .automatique: return "circle.lefthalf.filled"
        case .clair: return "sun.max"
        case .sombre: return "moon"
        }
    }

    var style: UIUserInterfaceStyle {
        switch self {
        case .automatique: return .unspecified
        case .clair: return .light
        case .sombre: return .dark
        }
    }

    /// Pose l'apparence sur toutes les fenêtres de l'application — en fondu
    /// quand on la change, d'un coup au lancement.
    @MainActor
    func appliquer(enFondu: Bool = false) {
        for scene in UIApplication.shared.connectedScenes {
            guard let scene = scene as? UIWindowScene else { continue }
            for window in scene.windows where window.overrideUserInterfaceStyle != style {
                if enFondu {
                    UIView.transition(with: window, duration: 0.35, options: .transitionCrossDissolve) {
                        window.overrideUserInterfaceStyle = style
                    }
                } else {
                    window.overrideUserInterfaceStyle = style
                }
            }
        }
    }
}

/// Le sélecteur d'apparence : un menu dans la barre d'outils, qui montre le
/// choix en cours par son symbole.
struct SelecteurApparence: View {
    @AppStorage(Apparence.cle) private var apparence: Apparence = .automatique

    var body: some View {
        Menu {
            Picker(Localized.text("theme.title"), selection: $apparence) {
                ForEach(Apparence.allCases) { choix in
                    Label(choix.libelle, systemImage: choix.symbole).tag(choix)
                }
            }
        } label: {
            Label(Localized.text("theme.title"), systemImage: apparence.symbole)
        }
        .onChange(of: apparence) { _, nouvelle in
            nouvelle.appliquer(enFondu: true)
        }
    }
}
