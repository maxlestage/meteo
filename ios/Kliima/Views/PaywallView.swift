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
