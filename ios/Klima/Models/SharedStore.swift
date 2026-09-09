import Foundation

/// Stockage partagé entre l'application et ses extensions.
///
/// Le widget d'écran d'accueil doit savoir quelle parcelle afficher : il lit
/// donc celle que l'application a enregistrée, via un groupe d'applications.
/// Sans groupe configuré — le cas au premier lancement d'un projet non signé —
/// on retombe sur les réglages locaux plutôt que de planter.
enum SharedStore {

    /// Groupe d'applications à déclarer dans les entitlements des deux cibles.
    static let appGroup = "group.com.klima.app"

    private static let parcelleKey = "klima.parcelle"

    static var defaults: UserDefaults {
        UserDefaults(suiteName: appGroup) ?? .standard
    }

    static func save(_ parcelle: Parcelle) {
        guard let data = try? JSONEncoder().encode(parcelle) else { return }
        defaults.set(data, forKey: parcelleKey)
    }

    static func loadParcelle() -> Parcelle? {
        guard let data = defaults.data(forKey: parcelleKey) else { return nil }
        return try? JSONDecoder().decode(Parcelle.self, from: data)
    }
}
