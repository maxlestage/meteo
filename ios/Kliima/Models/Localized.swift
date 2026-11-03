import Foundation

/// Accès aux textes traduits.
///
/// Les clés sont dynamiques — le domaine renvoie « wmo.drizzle » ou
/// « soil.ressuye », l'interface les résout — d'où ce passage explicite par
/// `NSLocalizedString` plutôt que par les littéraux de SwiftUI. Le catalogue
/// vit dans `Kliima/Resources/Localizable.xcstrings` et couvre le français,
/// l'anglais et l'espagnol.
enum Localized {

    /// Texte d'une clé, éventuellement complété de ses valeurs (`%1$@`, `%2$@`…).
    static func text(_ key: String, _ arguments: CVarArg...) -> String {
        let format = NSLocalizedString(key, comment: "")
        guard !arguments.isEmpty else { return format }
        return String(format: format, locale: .current, arguments: arguments)
    }

    /// Le minimum et le maximum du jour tels que VoiceOver doit les dire.
    ///
    /// À l'écran, des flèches — « ↑ 13°  ↓ 9° » —, que VoiceOver lisait
    /// « flèche vers le haut ». L'ordre suit celui de l'écran : l'en-tête
    /// commence par le maximum, les widgets par le minimum.
    static func amplitude(min: Double, max: Double, maximumDAbord: Bool = false) -> String {
        let minimum = text("range.min.a11y", AgroFormat.temperature(min))
        let maximum = text("range.max.a11y", AgroFormat.temperature(max))
        return maximumDAbord ? "\(maximum), \(minimum)" : "\(minimum), \(maximum)"
    }
}
