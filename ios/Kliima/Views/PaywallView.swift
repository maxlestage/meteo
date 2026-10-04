import AuthenticationServices
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
    @State private var connexion = false

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

                    // Pendant l'essai, c'est l'action principale de cet écran :
                    // elle vient avant l'argumentaire, pas sous les conditions
                    // de vente, où il fallait faire défiler pour la trouver.
                    compte

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

    /// Le compte, et seulement quand il y a un relais pour le reconnaître.
    ///
    /// Sans `KliimaRelay`, ce bloc n'existe pas : un bouton qui ne peut rien
    /// ouvrir serait une promesse en l'air, et la version de l'App Store n'a
    /// pas à exposer la mécanique de l'essai. Vider la clé le fait disparaître.
    ///
    /// Une tape sur le bouton d'Apple, et c'est tout : rien à taper. Apple
    /// prouve l'adresse, le relais dit si elle est invitée.
    @ViewBuilder
    private var compte: some View {
        if PlanGrant.relayURL != nil {
            VStack(alignment: .leading, spacing: 10) {
                Text(Localized.text("account.title"))
                    .font(.subheadline.weight(.semibold))

                if let ouvert = subscription.compte {
                    Text(Localized.text("account.signedIn", ouvert.courriel))
                        .font(.footnote)

                    if subscription.plan == .pro {
                        Text(Localized.text("account.active"))
                            .font(.footnote.weight(.semibold))
                    } else {
                        // Le compte est ouvert mais pas invité. On montre
                        // l'adresse qu'Apple a prouvée — c'est la sienne, il
                        // n'y a rien là à énumérer — parce que c'est elle qu'il
                        // faut ajouter à la liste. Y compris quand Apple a
                        // donné une adresse relais : celle-là, personne ne la
                        // devinerait.
                        Text(Localized.text("account.notInvited"))
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }

                    Button(Localized.text("account.signOut"), role: .destructive) {
                        subscription.deconnecter()
                    }
                    .font(.footnote)
                } else {
                    Text(Localized.text("account.hint"))
                        .font(.footnote)
                        .foregroundStyle(.secondary)

                    SignInWithAppleButton(.signIn) { demande in
                        // L'adresse, et rien d'autre : le nom ne sert à rien
                        // ici, et ce qu'on ne demande pas n'a pas à être gardé.
                        demande.requestedScopes = [.email]
                    } onCompletion: { resultat in
                        connexion = true
                        Task {
                            await subscription.connecter(resultat)
                            connexion = false
                        }
                    }
                    .signInWithAppleButtonStyle(.white)
                    .frame(height: 44)
                    .disabled(connexion)
                }

                if let echec = subscription.echecConnexion {
                    Text(texte(echec))
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
            }
            .padding(14)
            .background(Color.white.opacity(0.06), in: RoundedRectangle(cornerRadius: 14))
        }
    }

    /// La phrase d'un échec de connexion. Le domaine rend un motif, l'écran le
    /// traduit — comme partout ailleurs.
    ///
    /// Chaque panne a la sienne : « Apple n'a pas ouvert la connexion » et
    /// « le serveur n'a pas reconnu la preuve » se réparent à deux endroits
    /// différents, et les confondre rendait tout retour de testeur impossible à
    /// lire.
    private func texte(_ echec: Session.Echec) -> String {
        switch echec {
        case .sansRelais, .injoignable: return Localized.text("account.error.unreachable")
        case .comptesFermes: return Localized.text("account.error.closed")
        case .refuse: return Localized.text("account.error.refused")
        case .apple(let code): return Localized.text("account.error.apple", String(code))
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
