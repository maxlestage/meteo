import SwiftUI

/// Ce qui bouge dans l'interface, au même rythme que l'application web
/// (`rust/klima-web/styles.css`) : les cartes arrivent en cascade, les barres
/// montent depuis leur base. Pour qui demande moins de mouvement, tout est en
/// place d'emblée.

extension Color {
    /// Le bleu de la pluie : clair sur un ciel sombre, plus soutenu sur un ciel
    /// clair — l'azur pâle n'y avait pas assez de contraste pour un intitulé.
    static let bleuPluie = Color(uiColor: UIColor { trait in
        trait.userInterfaceStyle == .dark
            ? UIColor(red: 0.498, green: 0.816, blue: 0.961, alpha: 1)
            : UIColor(red: 0.098, green: 0.420, blue: 0.690, alpha: 1)
    })
}

/// Une carte qui arrive : elle monte de quelques points en apparaissant, un
/// peu après celle d'avant.
struct EntreeEnCascade: ViewModifier {
    let rang: Int

    @State private var apparu = false
    @Environment(\.accessibilityReduceMotion) private var moinsDeMouvement

    func body(content: Content) -> some View {
        let visible = apparu || moinsDeMouvement
        return content
            .opacity(visible ? 1 : 0)
            .offset(y: visible ? 0 : 22)
            .scaleEffect(visible ? 1 : 0.985)
            .onAppear {
                guard !apparu else { return }
                withAnimation(.spring(response: 0.6, dampingFraction: 0.85).delay(Double(rang) * 0.07)) {
                    apparu = true
                }
            }
    }
}

/// Une barre qui monte depuis sa base — ou s'étire depuis sa gauche.
struct Pousse: ViewModifier {
    let rang: Int
    let horizontal: Bool

    @State private var pousse = false
    @Environment(\.accessibilityReduceMotion) private var moinsDeMouvement

    func body(content: Content) -> some View {
        let plein = pousse || moinsDeMouvement
        return content
            .scaleEffect(
                x: horizontal && !plein ? 0.02 : 1,
                y: !horizontal && !plein ? 0.02 : 1,
                anchor: horizontal ? .leading : .bottom
            )
            .onAppear {
                guard !pousse else { return }
                withAnimation(.spring(response: 0.7, dampingFraction: 0.75).delay(0.15 + Double(rang) * 0.035)) {
                    pousse = true
                }
            }
    }
}

extension View {
    func entreeEnCascade(_ rang: Int) -> some View { modifier(EntreeEnCascade(rang: rang)) }
    func pousse(_ rang: Int, horizontal: Bool = false) -> some View {
        modifier(Pousse(rang: rang, horizontal: horizontal))
    }
}
