import SwiftUI

/// Écran unique de l'application : les indicateurs agronomiques de la parcelle.
struct DashboardView: View {
    @StateObject private var viewModel = DashboardViewModel()
    @State private var query = ""

    private let columns = [GridItem(.adaptive(minimum: 165), spacing: 12)]

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    if viewModel.isLoading && viewModel.forecast == nil {
                        ProgressView("Chargement des données agronomiques…")
                            .padding(.top, 60)
                    }

                    if let message = viewModel.errorMessage {
                        errorBanner(message)
                    }

                    if let summary = viewModel.summary, let forecast = viewModel.forecast {
                        LazyVGrid(columns: columns, spacing: 12) {
                            ForEach(cards(for: summary), id: \.title) { card in
                                IndicatorCard(
                                    title: card.title,
                                    value: card.value,
                                    caption: card.caption,
                                    tone: card.tone,
                                    tags: card.tags
                                )
                            }
                        }

                        SprayTimelineView(hours: forecast.hourly, timeZone: viewModel.timeZone)
                        WeeklyForecastView(days: forecast.daily, timeZone: viewModel.timeZone)
                        source(forecast)
                    }
                }
                .padding(16)
            }
            .background(Color(uiColor: .systemGroupedBackground))
            .navigationTitle(viewModel.parcelle.name)
            .navigationBarTitleDisplayMode(.large)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        viewModel.useCurrentLocation()
                    } label: {
                        Label("Me localiser", systemImage: "location")
                    }
                }
            }
            .searchable(text: $query, prompt: "Rechercher une commune")
            .onChange(of: query) { _, newValue in
                viewModel.search(newValue)
            }
            .overlay(alignment: .top) {
                if !viewModel.searchResults.isEmpty {
                    searchResults
                }
            }
            .refreshable { await viewModel.load() }
            .task { await viewModel.load() }
        }
    }

    // MARK: Sous-vues

    private var searchResults: some View {
        VStack(spacing: 0) {
            ForEach(viewModel.searchResults) { result in
                Button {
                    query = ""
                    viewModel.select(result)
                } label: {
                    HStack {
                        Text(result.name)
                        Spacer()
                        Text(result.subtitle)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 11)
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                Divider()
            }
        }
        .background(Color(uiColor: .secondarySystemGroupedBackground))
        .clipShape(RoundedRectangle(cornerRadius: 14))
        .shadow(color: .black.opacity(0.12), radius: 12, y: 4)
        .padding(.horizontal, 16)
    }

    private func errorBanner(_ message: String) -> some View {
        HStack(alignment: .firstTextBaseline) {
            Label(message, systemImage: "exclamationmark.triangle")
                .font(.subheadline)
            Spacer()
            Button("Réessayer") { viewModel.load() }
                .font(.subheadline.weight(.semibold))
        }
        .padding(14)
        .background(Tone.bad.color.opacity(0.12), in: RoundedRectangle(cornerRadius: 12))
        .foregroundStyle(Tone.bad.color)
    }

    private func source(_ forecast: AgroForecast) -> some View {
        VStack(spacing: 4) {
            Text("Données Open-Meteo — modèle agricole : humidité et température du sol, ET0 FAO-56, VPD.")
            Text("Parcelle à \(Int(forecast.elevation.rounded())) m · mise à jour \(updated(forecast.fetchedAt))")
        }
        .font(.caption2)
        .foregroundStyle(.secondary)
        .multilineTextAlignment(.center)
        .padding(.top, 4)
    }

    // MARK: Construction des cartes

    private struct CardModel {
        let title: String
        let value: String
        let caption: String
        let tone: Tone
        var tags: [String] = []
    }

    private func cards(for summary: AgroSummary) -> [CardModel] {
        let water = summary.water
        let soil = summary.soil
        let disease = summary.disease
        let frost = summary.frost

        return [
            CardModel(
                title: "Bilan hydrique 7 j",
                value: String(format: "%@%.1f mm", water.balance > 0 ? "+" : "", water.balance),
                caption: water.irrigationAdvice > 0
                    ? "Irrigation conseillée : \(Int(water.irrigationAdvice.rounded())) mm"
                    : String(format: "Pluie %.1f mm · ET0 %.1f mm", water.precipitation, water.evapotranspiration),
                tone: water.status == .deficit ? .bad : (water.status == .excedent ? .warn : .good)
            ),
            CardModel(
                title: "État du sol",
                value: soil.state.label,
                caption: String(format: "%.1f %% vol. · %.1f °C à 6 cm", soil.moisture * 100, soil.temperature),
                tone: soil.state == .sature ? .bad : (soil.state == .sec ? .warn : .good),
                tags: [
                    soil.trafficable ? "Portance correcte" : "Risque de tassement",
                    soil.sowable ? "Semis possible" : "Semis déconseillé",
                ]
            ),
            CardModel(
                title: "Fenêtre de traitement",
                value: summary.nextSpray.map { range($0) } ?? "Aucune",
                caption: summary.nextSpray.map { "Score \($0.score)/100 sur la plage" }
                    ?? "Rien d’exploitable sur 7 jours",
                tone: summary.nextSpray.map { $0.score >= 80 ? Tone.good : .warn } ?? .bad
            ),
            CardModel(
                title: "Pression maladie",
                value: disease.level.label,
                caption: "\(disease.leafWetnessHours) h d’humectation du feuillage sur 24 h",
                tone: disease.level == .elevee ? .bad : (disease.level == .moyenne ? .warn : .good)
            ),
            CardModel(
                title: "Risque de gel",
                value: frost.severity.label,
                caption: String(format: "Mini %.1f °C", frost.minTemperature)
                    + (frost.hoarFrost ? " · gelée blanche probable" : ""),
                tone: frost.severity == .aucun ? .good : (frost.severity == .faible ? .warn : .bad)
            ),
            CardModel(
                title: "Degrés-jours (base 10)",
                value: String(format: "%.1f °C·j", summary.gdd),
                caption: "Cumul sur les 7 jours de prévision",
                tone: .neutral
            ),
        ]
    }

    // MARK: Formatage

    private func range(_ opportunity: SprayOpportunity) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "fr_FR")
        formatter.timeZone = viewModel.timeZone
        formatter.dateFormat = "EEE HH 'h'"
        let endFormatter = DateFormatter()
        endFormatter.locale = Locale(identifier: "fr_FR")
        endFormatter.timeZone = viewModel.timeZone
        endFormatter.dateFormat = "HH 'h'"
        return "\(formatter.string(from: opportunity.start)) → \(endFormatter.string(from: opportunity.end))"
    }

    private func updated(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "fr_FR")
        formatter.timeZone = viewModel.timeZone
        formatter.dateFormat = "HH:mm"
        return formatter.string(from: date)
    }
}

#Preview {
    DashboardView()
}
