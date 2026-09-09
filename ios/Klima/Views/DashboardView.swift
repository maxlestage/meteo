import SwiftUI

/// Écran unique de l'application : la météo agricole de la parcelle, présentée
/// comme l'application Météo du système.
struct DashboardView: View {
    @StateObject private var viewModel = DashboardViewModel()
    @StateObject private var activity = SprayActivityController()
    @StateObject private var subscription = Subscription()
    @State private var query = ""
    /// Renseigné quand on ouvre l'écran d'abonnement : on sait alors sur quelle
    /// fonction l'utilisateur a buté.
    @State private var paywallFor: Feature?

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
                            ProgressView(Localized.text("app.loading"))
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
                                timeZone: viewModel.timeZone,
                                isFollowing: activity.isRunning,
                                canFollow: activity.isAvailable,
                                onFollow: { toggleFollow(forecast, summary) }
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
                        Label(Localized.text("app.locate"), systemImage: "location")
                    }
                    .tint(.white)
                }

                // L'accès à l'abonnement ne se montre qu'au palier libre :
                // rappeler à un abonné qu'il paie n'apporte rien.
                if subscription.plan == .libre {
                    ToolbarItem(placement: .topBarLeading) {
                        Button {
                            paywallFor = .recoupement
                        } label: {
                            Label(Localized.text("plan.pro"), systemImage: "sparkles")
                        }
                        .tint(.white)
                    }
                }
            }
            .searchable(text: $query, prompt: Localized.text("app.search"))
            .onChange(of: query) { _, newValue in
                viewModel.search(newValue)
            }
            .overlay(alignment: .top) {
                if !viewModel.searchResults.isEmpty {
                    searchResults
                }
            }
            .refreshable { await reload() }
            .task { await reload() }
            .sheet(item: $paywallFor) { feature in
                PaywallView(subscription: subscription, reason: feature)
            }
            .task {
                // À chaque ouverture : StoreKit est la source de vérité, le
                // stockage partagé n'en est que la copie pour les extensions.
                await subscription.refresh()
            }
        }
        .preferredColorScheme(.dark)
    }

    // MARK: Activité en direct

    /// Recharge la prévision, puis répercute les nouvelles conditions sur
    /// l'activité en direct si elle est en cours.
    private func reload() async {
        await viewModel.load()
        guard let forecast = viewModel.forecast, let summary = viewModel.summary else { return }
        await activity.refresh(hours: forecast.hourly, opportunity: summary.nextSpray)
    }

    private func toggleFollow(_ forecast: AgroForecast, _ summary: AgroSummary) {
        guard let opportunity = summary.nextSpray else { return }
        if activity.isRunning {
            Task { await activity.stop() }
        } else {
            activity.start(
                parcelle: forecast.parcelle,
                opportunity: opportunity,
                timeZone: viewModel.timeZone,
                hours: forecast.hourly
            )
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
            Button(Localized.text("app.retry")) { Task { await viewModel.load() } }
                .buttonStyle(.bordered)
                .tint(.white)
        }
        .padding(.top, 40)
    }

    private func source(_ forecast: AgroForecast) -> some View {
        Text(Localized.text("app.source", AgroFormat.unit(forecast.elevation, "m", decimals: 0)))
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

        var tiles: [TileModel] = [
            TileModel(
                label: Localized.text("tile.soil"),
                value: soil.state.label,
                caption: Localized.text(
                    "tile.soil.caption",
                    AgroFormat.percent(soil.moisture * 100),
                    AgroFormat.unit(soil.temperature, "°C"),
                    Localized.text(soil.trafficable ? "soil.trafficable" : "soil.compaction")
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
                label: Localized.text("tile.water"),
                value: AgroFormat.signedUnit(water.balance, "mm"),
                caption: water.irrigationAdvice > 0
                    ? Localized.text(
                        "tile.water.irrigation",
                        AgroFormat.unit(water.irrigationAdvice, "mm", decimals: 0)
                    )
                    : Localized.text(
                        "tile.water.caption",
                        AgroFormat.unit(water.precipitation, "mm"),
                        AgroFormat.unit(water.evapotranspiration, "mm")
                    )
            ),
            TileModel(
                label: Localized.text("tile.wind"),
                value: AgroFormat.unit(forecast.current.windSpeed, "km/h", decimals: 0),
                caption: Localized.text(
                    "tile.wind.caption",
                    AgroFormat.unit(forecast.current.windGusts, "km/h", decimals: 0),
                    AgroFormat.unit(AgroThresholds.sprayWindMax, "km/h", decimals: 0)
                )
            ),
            TileModel(
                label: Localized.text("tile.frost"),
                value: summary.frost.severity.label,
                caption: Localized.text(
                    "tile.frost.caption",
                    AgroFormat.unit(summary.frost.minTemperature, "°C"),
                    summary.frost.hoarFrost ? ", " + Localized.text("frost.hoarFrost") : ""
                )
            ),
            TileModel(
                label: Localized.text("tile.disease"),
                value: summary.disease.level.label,
                caption: Localized.text("tile.disease.caption", String(summary.disease.leafWetnessHours))
            ),
            TileModel(
                label: Localized.text("tile.gdd"),
                value: AgroFormat.unit(summary.gdd, "°C·j"),
                caption: Localized.text(
                    "tile.gdd.caption",
                    AgroFormat.unit(AgroThresholds.gddBase, "°C", decimals: 0)
                )
            ),
            TileModel(
                label: Localized.text("tile.sunrise"),
                value: AgroFormat.time(today?.sunrise, in: zone),
                caption: Localized.text("tile.sunrise.caption", AgroFormat.time(today?.sunset, in: zone))
            ),
            TileModel(
                label: Localized.text("tile.sowing"),
                value: Localized.text(soil.sowable ? "tile.sowing.yes" : "tile.sowing.no"),
                caption: Localized.text("tile.sowing.caption", AgroFormat.unit(soil.temperature, "°C"))
            ),
        ]

        // Le recoupement est une fonction du palier payant. On ne le cache
        // pas : on montre la tuile fermée, avec ce qu'elle contiendrait. Une
        // fonction invisible ne se vend pas, et une fonction qui disparaît
        // sans explication passe pour une panne.
        guard subscription.plan.allows(.recoupement) else {
            tiles.append(
                TileModel(
                    label: Localized.text("consensus.title"),
                    value: Localized.text("tile.locked"),
                    caption: Localized.text(Feature.recoupement.upgradeReasonKey)
                )
            )
            return tiles
        }

        // Le recoupement des modèles n'apparaît que s'il a abouti.
        if let consensus = viewModel.consensus {
            let detail = Localized.text(
                "consensus.detail",
                String(consensus.readings.count),
                AgroFormat.unit(consensus.temperature.spread, "°C")
            )
            let rain = consensus.agreeOnRain ? "" : " · " + Localized.text("consensus.rainDisagreement")
            let median = Localized.text(
                "consensus.median",
                AgroFormat.unit(consensus.temperature.median, "°C")
            )
            tiles.append(
                TileModel(
                    label: Localized.text("consensus.title"),
                    value: consensus.agreement.label,
                    caption: "\(detail)\(rain). \(median).",
                    gauge: (
                        position: 1 - min(consensus.temperature.spread / 5, 1),
                        colors: [
                            Color(red: 0.937, green: 0.541, blue: 0.353),
                            Color(red: 0.941, green: 0.757, blue: 0.294),
                            Color(red: 0.494, green: 0.816, blue: 0.478),
                        ]
                    )
                )
            )
        }

        return tiles
    }
}

#Preview {
    DashboardView()
}
