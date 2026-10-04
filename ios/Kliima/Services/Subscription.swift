import Foundation
import StoreKit

/// L'abonnement, vu de l'application.
///
/// Pas de comptes, pas de serveur d'identité : StoreKit tient le registre des
/// achats et l'App Store gère le paiement, la TVA et les remboursements. Kliima
/// se contente de lire ce que StoreKit lui dit.
///
/// Ce service ne décide de rien : il traduit l'état de la boutique en `Plan`,
/// et c'est `Plan` — testé des deux côtés — qui dit ce qui s'ouvre. Aucune
/// règle de palier n'a sa place ici.
///
/// **Ce qui n'a jamais tourné.** Ce fichier n'a pas d'appareil pour s'exécuter
/// dans l'environnement où il a été écrit : il est vérifié syntaxiquement, pas
/// à l'usage. Le premier lancement sur un appareil réel reste à faire.
@MainActor
final class Subscription: ObservableObject {

    /// Identifiant du produit, à déclarer à l'identique dans App Store Connect.
    static let productID = "com.kliima.app.pro.mensuel"

    @Published private(set) var plan: Plan = SharedStore.loadPlan()
    @Published private(set) var product: Product?
    /// Renseigné quand la boutique est injoignable : l'interface le montre
    /// plutôt que de laisser un bouton d'achat inerte.
    @Published private(set) var storeUnavailable = false

    private var updates: Task<Void, Never>?

    init() {
        // Les achats faits ailleurs — autre appareil, partage familial,
        // remboursement — arrivent par ce flux. Sans l'écouter, un abonné
        // resterait bloqué au palier libre jusqu'au prochain lancement.
        updates = Task { [weak self] in
            for await update in Transaction.updates {
                if case .verified(let transaction) = update {
                    await transaction.finish()
                }
                await self?.refresh()
            }
        }
    }

    deinit { updates?.cancel() }

    /// Relit le palier auprès de StoreKit et le recopie pour les extensions.
    ///
    /// Deux sources, et l'une n'efface jamais l'autre : ce que la boutique
    /// atteste, et ce que le déploiement accorde (`KLIMA_PRO`, côté relais)
    /// pendant l'essai. Un relais muet ne fait donc pas perdre un abonnement
    /// réel, et une boutique vide n'annule pas l'accord du relais.
    ///
    /// **Un silence n'est pas un refus.** Le relais injoignable — un tunnel,
    /// un avion, un redémarrage de serveur — laisse le palier tel qu'il était.
    /// Sans cela, un testeur perdait son accès au premier creux de réseau, et
    /// la perte était écrite sur le disque : au lancement suivant, l'écran
    /// repartait du palier libre avant même d'avoir pu redemander. Seul un
    /// `libre` dit franchement par le relais retire quelque chose.
    func refresh() async {
        var entitled = false
        for await result in Transaction.currentEntitlements {
            // Une transaction non vérifiée est ignorée : la signature de
            // l'App Store est ce qui distingue un achat d'une affirmation.
            guard case .verified(let transaction) = result else { continue }
            if transaction.productID == Self.productID, transaction.revocationDate == nil {
                entitled = true
            }
        }

        // Ce que la boutique atteste ne dépend d'aucun relais.
        let next: Plan
        if entitled {
            next = .pro
        } else {
            switch await PlanGrant.verdict() {
            case .accorde: next = .pro
            case .refuse: next = .libre
            case .injoignable: next = plan
            }
        }

        plan = next
        SharedStore.save(next)
    }

    /// Charge le produit pour afficher son prix — celui d'App Store Connect,
    /// dans la devise du lecteur. On ne code jamais un prix en dur : il varie
    /// par pays et il changera.
    func loadProduct() async {
        do {
            product = try await Product.products(for: [Self.productID]).first
            storeUnavailable = product == nil
        } catch {
            storeUnavailable = true
        }
    }

    /// Lance l'achat. Renvoie vrai si le palier a changé.
    @discardableResult
    func purchase() async -> Bool {
        guard let product else { return false }
        do {
            let result = try await product.purchase()
            switch result {
            case .success(let verification):
                if case .verified(let transaction) = verification {
                    await transaction.finish()
                }
                await refresh()
                return plan == .pro
            case .userCancelled, .pending:
                return false
            @unknown default:
                return false
            }
        } catch {
            return false
        }
    }

    /// Restaure les achats.
    ///
    /// Indispensable à ce niveau de prix : un courriel de support coûte plus
    /// cher qu'une année d'abonnement, et « j'ai réinstallé et j'ai tout
    /// perdu » est le premier motif d'écriture.
    func restore() async {
        try? await AppStore.sync()
        await refresh()
    }
}
