import StoreKit
import SwiftUI

/// L'écran d'abonnement.
///
/// Il ne dit pas ce qu'il retire, il dit ce qu'il ajoute — parce que le palier
/// libre ne retire rien : la journée du jour et sa parcelle restent entières
/// sans payer. Ce qui s'achète est l'échelle et l'anticipation.
///
/// Le prix n'est jamais écrit en dur : il vient d'App Store Connect, dans la
/// devise et le format du lecteur. Un prix codé dans l'application serait faux
/// dans la moitié des pays et le jour où il change.
///
/// **Ce qui n'a jamais tourné.** Aucun appareil n'a affiché cet écran : il est
/// vérifié syntaxiquement, pas à l'usage.
struct PaywallView: View {
    @ObservedObject var subscription: Subscription
    @Environment(\.dismiss) private var dismiss

    /// Renseigné quand on arrive ici en butant sur une fonction précise : on
    /// rappelle laquelle plutôt que d'afficher un argumentaire générique.
    var reason: Feature?

    @State private var busy = false

    /// L'adresse d'essai, relue à l'ouverture pour ne pas la redemander.
    @State private var courriel = SharedStore.loadCourriel() ?? ""
    @State private var verification = false
    @State private var refuse = false

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 22) {
                    if let reason {
                        Text(Localized.text(reason.upgradeReasonKey))
                            .font(.callout)
                            .foregroundStyle(.secondary)
                    }

                    Text(Localized.text("paywall.title"))
                        .font(.largeTitle.weight(.bold))

                    VStack(alignment: .leading, spacing: 14) {
                        ForEach(Feature.allCases, id: \.self) { feature in
                            Label {
                                Text(Localized.text("plan.feature.\(feature.rawValue)"))
                            } icon: {
                                Image(systemName: "checkmark.circle.fill")
                                    .foregroundStyle(Color.accentColor)
                            }
                            .font(.body)
                        }
                    }

                    Text(Localized.text("paywall.free"))
                        .font(.footnote)
                        .foregroundStyle(.secondary)

                    buyButton

                    Button(Localized.text("paywall.restore")) {
                        Task { await subscription.restore() }
                    }
                    .font(.footnote)
                    .frame(maxWidth: .infinity)

                    Text(Localized.text("paywall.terms"))
                        .font(.caption2)
                        .foregroundStyle(.secondary)

                    accesDEssai
                }
                .padding(24)
            }
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(Localized.text("paywall.close")) { dismiss() }
                }
            }
        }
        .task {
            await subscription.loadProduct()
            await subscription.refresh()
        }
        .onChange(of: subscription.plan) { _, plan in
            // L'achat abouti referme l'écran : rester dessus donnerait à croire
            // qu'il a échoué.
            if plan == .pro { dismiss() }
        }
    }

    /// L'accès de test, et seulement quand il y a un relais pour l'accorder.
    ///
    /// Sans `KliimaRelay`, ce bloc n'existe pas : un champ qui ne peut rien
    /// ouvrir serait une promesse en l'air, et la version de l'App Store n'a
    /// pas à exposer la mécanique de l'essai. Vider la clé le fait disparaître.
    ///
    /// Ce n'est pas une identification : le relais ne vérifie pas que l'adresse
    /// est relevée par celui qui la présente. Il compare à une liste d'invités,
    /// et c'est tout ce qu'on lui demande pendant un essai fermé.
    @ViewBuilder
    private var accesDEssai: some View {
        if PlanGrant.relayURL != nil {
            Divider()

            VStack(alignment: .leading, spacing: 8) {
                Text(Localized.text("paywall.grant.title"))
                    .font(.subheadline.weight(.semibold))

                Text(Localized.text("paywall.grant.hint"))
                    .font(.footnote)
                    .foregroundStyle(.secondary)

                HStack(spacing: 8) {
                    TextField(Localized.text("paywall.grant.field"), text: $courriel)
                        .textFieldStyle(.roundedBorder)
                        .textContentType(.emailAddress)
                        .keyboardType(.emailAddress)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .submitLabel(.go)
                        .onSubmit { verifier() }

                    Button(Localized.text("paywall.grant.check")) { verifier() }
                        .disabled(verification || saisieVide)
                }

                if refuse {
                    // On ne dit pas « adresse inconnue » : le relais ne le dit
                    // pas non plus, et pour la même raison — ce serait donner
                    // de quoi énumérer la liste une adresse à la fois.
                    Text(Localized.text("paywall.grant.refused"))
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
            }
            .onChange(of: courriel) { _, _ in refuse = false }
        }
    }

    private var saisieVide: Bool {
        courriel.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    /// Enregistre l'adresse et redemande le palier.
    ///
    /// L'adresse est gardée même si elle est refusée : une invitation qui
    /// arrive plus tard trouvera la saisie en place au lancement suivant, et
    /// retaper son adresse à chaque essai serait une punition pour rien.
    @MainActor
    private func verifier() {
        guard !saisieVide, !verification else { return }
        verification = true
        refuse = false
        SharedStore.saveCourriel(courriel)

        Task {
            await subscription.refresh()
            // Le succès referme l'écran tout seul — l'`onChange` du palier s'en
            // charge. Il n'y a donc que l'échec à dire.
            refuse = subscription.plan != .pro
            verification = false
        }
    }

    @ViewBuilder
    private var buyButton: some View {
        if let product = subscription.product {
            Button {
                busy = true
                Task {
                    await subscription.purchase()
                    busy = false
                }
            } label: {
                Text(Localized.text("paywall.buy", product.displayPrice))
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
            .disabled(busy)
        } else if subscription.storeUnavailable {
            // Un bouton d'achat inerte est pire qu'un message : on dit
            // pourquoi, et « Restaurer » reste accessible en dessous.
            Text(Localized.text("paywall.unavailable"))
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity)
        } else {
            ProgressView().frame(maxWidth: .infinity)
        }
    }
}
