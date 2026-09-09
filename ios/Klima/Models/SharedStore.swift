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
    private static let planKey = "klima.plan"
    private static let alertStateKey = "klima.alertState"

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

    // MARK: Palier

    /// Le palier est recopié ici par l'application après chaque vérification
    /// auprès de la boutique.
    ///
    /// Les extensions le lisent sans interroger StoreKit : un widget qui
    /// consulterait la boutique à chaque rafraîchissement de sa chronologie
    /// serait lent et se ferait rationner par le système. Le prix de ce choix
    /// est un décalage possible de quelques minutes après un achat — acceptable
    /// pour un widget, inacceptable pour un écran d'achat, qui lui interroge
    /// StoreKit directement.
    static func save(_ plan: Plan) {
        defaults.set(plan.rawValue, forKey: planKey)
    }

    /// Palier connu de la dernière vérification. Le palier libre par défaut :
    /// en cas de doute, on n'ouvre pas ce qui se paie.
    static func loadPlan() -> Plan {
        Plan(rawValue: defaults.string(forKey: planKey) ?? "") ?? .libre
    }

    // MARK: État des alertes

    /// Ce qu'on sait déjà avoir dit, pour ne pas le redire au réveil suivant.
    static func save(_ state: AlertState) {
        guard let data = try? JSONEncoder().encode(state) else { return }
        defaults.set(data, forKey: alertStateKey)
    }

    static func loadAlertState() -> AlertState {
        guard let data = defaults.data(forKey: alertStateKey),
              let state = try? JSONDecoder().decode(AlertState.self, from: data)
        else { return .empty }
        return state
    }
}
