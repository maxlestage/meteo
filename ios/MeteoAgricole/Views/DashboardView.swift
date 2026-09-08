import SwiftUI

/// Écran unique de l'application : la météo agricole de la parcelle, présentée
/// comme l'application Météo du système.
struct DashboardView: View {
    @StateObject private var viewModel = DashboardViewModel()
    @State private var query = ""

    private let tiles = [GridItem(.adaptive(minimum: 150), spacing: 12)]

    var body: some View {
        NavigationStack {
            ZStack {
                SkyBackground(
                    isDay: viewModel.forecast?.current.isDay ?? false,
                    weatherCode: viewModel.forecast?.current.weatherCode ?? 3
                )

                ScrollView {
                    VStack(spacing: 12) {
                        if viewModel.isLoading && viewModel.forecast == nil {
                            ProgressView()
                                .tint(.white)
                                .padding(.top, 80)
                        }

                        if let message = viewModel.errorMessage {
                            errorBanner(message)
                        }

                        if let forecast = viewModel.forecast, let summary = viewModel.summary {
                            HeroView(forecast: forecast)

                            HourlyStripView(
                                hours: forecast.hourly,
                                current: forecast.current,
                                timeZone: viewModel.timeZone
                            )

                            DailyListView(
                                days: forecast.daily,
                                currentTemperature: forecast.current.temperature,
                                timeZone: viewModel.timeZone
                            )

                            SprayCardView(
                                hours: forecast.hourly,
                                nextSpray: summary.nextSpray,
                                timeZone: viewModel.timeZone
                            )

                            LazyVGrid(columns: tiles, spacing: 12) {
                                ForEach(detailTiles(forecast, summary), id: \.label) { tile in
                                    DetailTile(
                                        label: tile.label,
                                        value: tile.value,
                                        caption: tile.caption,
                                        gauge: tile.gauge
                                    )
                                }
                            }

                            source(forecast)
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.bottom, 32)
                }
                .scrollIndicators(.hidden)
            }
            .foregroundStyle(.white)
            .toolbarBackground(.hidden, for: .navigationBar)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        viewModel.useCurrentLocation()
                    } label: {
                        Label("Me localiser", systemImage: "location")
                    }
                    .tint(.white)
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
        .preferredColorScheme(.dark)
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
                            .foregroundStyle(.white.opacity(0.62))
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 11)
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                Divider().overlay(Color.white.opacity(0.14))
            }
        }
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 14))
        .padding(.horizontal, 16)
    }

    private func errorBanner(_ message: String) -> some View {
        VStack(spacing: 10) {
            Text(message)
                .multilineTextAlignment(.center)
            Button("Réessayer") { Task { await viewModel.load() } }
                .buttonStyle(.bordered)
                .tint(.white)
        }
        .padding(.top, 40)
    }

    private func source(_ forecast: AgroForecast) -> some View {
        Text("Données Open-Meteo — modèle agricole : humidité et température du sol, ET0 FAO-56, déficit de pression de vapeur. Parcelle à \(Int(forecast.elevation.rounded())) m.")
            .font(.caption2)
            .foregroundStyle(.white.opacity(0.62))
            .multilineTextAlignment(.center)
            .padding(.top, 6)
    }

    // MARK: Tuiles de détail

    private struct TileModel {
        let label: String
        let value: String
        var caption: String?
        var gauge: (position: Double, colors: [Color])?
    }

    private func detailTiles(_ forecast: AgroForecast, _ summary: AgroSummary) -> [TileModel] {
        let soil = summary.soil
        let water = summary.water
        let zone = viewModel.timeZone
        let today = forecast.daily.first

        return [
            TileModel(
                label: "Humidité du sol",
                value: soil.state.label,
                caption: String(
                    format: "%.0f %% vol. · %.1f °C à 6 cm. %@",
                    soil.moisture * 100,
                    soil.temperature,
                    soil.trafficable ? "Le sol porte les engins." : "Risque de tassement."
                ),
                gauge: (
                    position: soil.moisture / 0.5,
                    colors: [
                        Color(red: 0.847, green: 0.702, blue: 0.416),
                        Color(red: 0.561, green: 0.769, blue: 0.416),
                        Color(red: 0.290, green: 0.639, blue: 0.847),
                        Color(red: 0.169, green: 0.373, blue: 0.620),
                    ]
                )
            ),
            TileModel(
                label: "Bilan hydrique",
                value: String(format: "%@%.1f mm", water.balance > 0 ? "+" : "", water.balance),
                caption: water.irrigationAdvice > 0
                    ? "Irrigation conseillée : \(Int(water.irrigationAdvice.rounded())) mm sur 7 jours."
                    : String(
                        format: "Pluie %.1f mm, ET0 %.1f mm sur 7 jours.",
                        water.precipitation,
                        water.evapotranspiration
                    )
            ),
            TileModel(
                label: "Vent",
                value: "\(Int(forecast.current.windSpeed.rounded())) km/h",
                caption: String(
                    format: "Rafales %d km/h. Limite de pulvérisation : %d km/h.",
                    Int(forecast.current.windGusts.rounded()),
                    Int(AgroThresholds.sprayWindMax)
                )
            ),
            TileModel(
                label: "Risque de gel",
                value: summary.frost.severity.label,
                caption: String(format: "Mini %.1f °C cette nuit", summary.frost.minTemperature)
                    + (summary.frost.hoarFrost ? ", gelée blanche probable." : ".")
            ),
            TileModel(
                label: "Pression maladie",
                value: summary.disease.level.label,
                caption: "\(summary.disease.leafWetnessHours) h d’humectation du feuillage sur 24 h."
            ),
            TileModel(
                label: "Degrés-jours",
                value: String(format: "%.1f °C·j", summary.gdd),
                caption: "Cumul sur 7 jours, base \(Int(AgroThresholds.gddBase)) °C."
            ),
            TileModel(
                label: "Lever",
                value: AgroFormat.time(today?.sunrise, in: zone),
                caption: "Coucher à \(AgroFormat.time(today?.sunset, in: zone))."
            ),
            TileModel(
                label: "Semis",
                value: soil.sowable ? "Possible" : "Déconseillé",
                caption: String(
                    format: "Sol à %.1f °C à 6 cm ; il faut 8 °C et un sol ressuyé.",
                    soil.temperature
                )
            ),
        ]
    }
}

#Preview {
    DashboardView()
}
