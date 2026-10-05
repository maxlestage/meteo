import SwiftUI

/// Mes villes : celles qu'on a enregistrées, d'une à l'autre d'un geste.
///
/// Le palier libre en garde une, Kliima ‣ Pro autant qu'on veut
/// (`rust/klima-core/src/villes.rs` et son miroir `Villes.swift`). Après une
/// résiliation, les villes au-delà de la limite restent dans la liste, fermées
/// d'un cadenas : on les retrouve telles quelles en revenant.
///
/// Chaque ville montre sa température et son ciel du moment : c'est ce qui
/// fait d'une liste de noms un coup d'œil sur plusieurs villes.
struct VillesView: View {
    @ObservedObject var viewModel: DashboardViewModel
    @ObservedObject var subscription: Subscription
    @Environment(\.dismiss) private var dismiss

    @State private var paywall = false

    private var acces: ParcelleAccess<Parcelle> { subscription.plan.partition(viewModel.villes) }

    var body: some View {
        NavigationStack {
            List {
                Section(Localized.text("villes.current")) {
                    HStack {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(viewModel.parcelle.name)
                            Text(viewModel.parcelle.subtitle)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }
                        Spacer()
                        if viewModel.villeAfficheeEnregistree {
                            Label(Localized.text("villes.saved"), systemImage: "star.fill")
                                .font(.footnote.weight(.semibold))
                                .foregroundStyle(.yellow)
                        } else {
                            Button {
                                enregistrer()
                            } label: {
                                Label(Localized.text("villes.save"), systemImage: "star")
                                    .font(.footnote.weight(.semibold))
                            }
                            .buttonStyle(.borderedProminent)
                        }
                    }
                }

                Section {
                    if viewModel.villes.isEmpty {
                        Text(Localized.text("villes.empty"))
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                    ForEach(acces.readable) { ville in
                        Button {
                            viewModel.select(ville)
                            dismiss()
                        } label: {
                            ligne(ville, fermee: false)
                        }
                        .buttonStyle(.plain)
                    }
                    .onDelete { index in
                        for ville in index.map({ acces.readable[$0] }) { viewModel.retirer(ville) }
                    }
                    .onMove { depuis, vers in
                        // Les villes lisibles sont en tête de liste : leurs
                        // places sont celles de la liste entière.
                        viewModel.deplacer(depuis: depuis, vers: vers)
                    }

                    ForEach(acces.locked) { ville in
                        Button {
                            paywall = true
                        } label: {
                            ligne(ville, fermee: true)
                        }
                        .buttonStyle(.plain)
                    }
                    .onDelete { index in
                        for ville in index.map({ acces.locked[$0] }) { viewModel.retirer(ville) }
                    }
                } header: {
                    Text(Localized.text("villes.title"))
                } footer: {
                    Text(Localized.text(subscription.plan == .pro ? "villes.pro" : "villes.free"))
                }
            }
            .navigationTitle(Localized.text("villes.title"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    if !viewModel.villes.isEmpty { EditButton() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(Localized.text("villes.done")) { dismiss() }
                }
            }
            .task { await viewModel.chargerApercus(acces.readable) }
            .sheet(isPresented: $paywall) {
                PaywallView(subscription: subscription, reason: .villes)
            }
        }
    }

    /// Une ville : son nom, son ciel et sa température du moment — ou un
    /// cadenas si le palier la ferme.
    private func ligne(_ ville: Parcelle, fermee: Bool) -> some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(ville.name)
                    if Villes.memeVille(ville, viewModel.parcelle) {
                        Image(systemName: "location.fill")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                            .accessibilityLabel(Localized.text("villes.current"))
                    }
                }
                Text(fermee ? Localized.text("villes.locked") : ville.subtitle)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            if fermee {
                Image(systemName: "lock.fill")
                    .foregroundStyle(.secondary)
            } else if let apercu = viewModel.apercus[ville.id] {
                let condition = WeatherCondition.forCode(apercu.weatherCode)
                Image(systemName: condition.icon.symbolName(isDay: apercu.isDay))
                    .symbolRenderingMode(.multicolor)
                    .accessibilityLabel(condition.label)
                Text(AgroFormat.temperature(apercu.temperature))
                    .font(.title3.monospacedDigit())
            } else {
                ProgressView()
            }
        }
        .contentShape(Rectangle())
        .opacity(fermee ? 0.6 : 1)
    }

    /// Enregistre la ville affichée ; au plafond du palier, l'écran
    /// d'abonnement dit pourquoi.
    private func enregistrer() {
        if viewModel.enregistrerVilleAffichee(plan: subscription.plan) == .limite {
            paywall = true
        }
    }
}
