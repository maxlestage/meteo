import Foundation

/// Accès aux textes traduits.
///
/// Les clés sont dynamiques — le domaine renvoie « wmo.drizzle » ou
/// « soil.ressuye », l'interface les résout — d'où ce passage explicite par
/// `NSLocalizedString` plutôt que par les littéraux de SwiftUI. Le catalogue
/// vit dans `Klima/Resources/Localizable.xcstrings` et couvre le français,
/// l'anglais et l'espagnol.
enum Localized {

    /// Texte d'une clé, éventuellement complété de ses valeurs (`%1$@`, `%2$@`…).
    static func text(_ key: String, _ arguments: CVarArg...) -> String {
        let format = NSLocalizedString(key, comment: "")
        guard !arguments.isEmpty else { return format }
        return String(format: format, locale: .current, arguments: arguments)
    }
}
