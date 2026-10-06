import SwiftUI

/// Écran unique de l'application : la météo de la ville, présentée comme
/// l'application Météo du système.
struct DashboardView: View {
    @StateObject private var viewModel = DashboardViewModel()
    @StateObject private var weather = WeatherActivityController()
    @StateObject private var subscription = Subscription()
    @State private var query = ""
    /// Renseigné quand on ouvre l'écran d'abonnement : on sait alors sur quelle
    /// fonction l'utilisateur a buté.
    @State private var paywallFor: Feature?
    /// La liste des villes enregistrées est ouverte.
    @State private var villesOuvertes = false

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
                                .tint(Color.encre)
                                .padding(.top, 80)
                        }

                        if let message = viewModel.errorMessage {
                            errorBanner(message)
                        }

                        if let forecast = viewModel.forecast {
                            // Une colonne par ville : quand elle change, les cartes
                            // reviennent en cascade, comme sur le web.
                            VStack(spacing: 12) {
                                HeroView(forecast: forecast)
                                    .entreeEnCascade(0)

                                // L'activité en direct de la météo s'ouvre et se
                                // ferme à la main : elle ne suit aucun événement
                                // borné, et iOS la termine de lui-même au bout de
                                // huit heures environ.
                                if weather.isAvailable {
                                    Button {
                                        toggleWeather(forecast)
                                    } label: {
                                        Label(
                                            Localized.text(weather.isRunning ? "weather.stop" : "weather.follow"),
                                            systemImage: weather.isRunning ? "livephoto.slash" : "livephoto"
                                        )
                                        .font(.footnote.weight(.semibold))
                                    }
                                    .buttonStyle(.plain)
                                    .foregroundStyle(Color.encre.opacity(0.85))
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                }

                                // La demi-heure en cours et les deux heures qui
                                // viennent, au quart d'heure : la question qu'on
                                // pose la main sur la poignée.
                                GuetteurCardView(
                                    quarts: viewModel.quartsDeLaVille,
                                    luA: viewModel.quartsLusA,
                                    enLecture: viewModel.veilleEnLecture,
                                    timeZone: viewModel.timeZone,
                                    ciel: forecast.ciel
                                )
                                .entreeEnCascade(1)

                                // Puis la journée : faut-il un parapluie, et
                                // jusqu'à quand — puis ce qu'il faut emporter.
                                PluieCardView(hours: forecast.hourly, timeZone: viewModel.timeZone)
                                    .entreeEnCascade(2)

                                HourlyStripView(
                                    hours: forecast.hourly,
                                    current: forecast.current,
                                    timeZone: viewModel.timeZone
                                )
                                .entreeEnCascade(3)

                                DailyListView(
                                    days: forecast.daily,
                                    currentTemperature: forecast.current.temperature,
                                    timeZone: viewModel.timeZone
                                )
                                .entreeEnCascade(4)

                                // Chaque source, nommée : qui annonce quoi pour
                                // la ville. Même palier que la tuile d'accord, qui
                                // la résume — fermée, c'est elle qui le dit.
                                if subscription.plan.allows(.recoupement),
                                   let consensus = viewModel.consensus {
                                    SourcesCardView(consensus: consensus)
                                        .entreeEnCascade(5)
                                }

                                LazyVGrid(columns: tiles, spacing: 12) {
                                    ForEach(detailTiles(forecast), id: \.label) { tile in
                                        DetailTile(
                                            label: tile.label,
                                            value: tile.value,
                                            caption: tile.caption,
                                            gauge: tile.gauge
                                        )
                                    }
                                }
                                .entreeEnCascade(6)

                                source(forecast)
                                    .entreeEnCascade(7)
                            }
                            .id(forecast.parcelle)
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.bottom, 32)
                }
                .scrollIndicators(.hidden)
                .bordsNets()
            }
            .foregroundStyle(Color.encre)
            .toolbar {
                // Mes villes : une au palier libre, autant qu'on veut avec
                // Kliima ‣ Pro.
                // L'apparence : celle du téléphone, claire ou sombre.
                ToolbarItem(placement: .topBarTrailing) {
                    SelecteurApparence()
                        .tint(Color.encre)
                }

                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        villesOuvertes = true
                    } label: {
                        Label(Localized.text("villes.title"), systemImage: "list.star")
                    }
                    .tint(Color.encre)
                }

                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        viewModel.useCurrentLocation()
                    } label: {
                        Label(Localized.text("app.locate"), systemImage: "location")
                    }
                    .tint(Color.encre)
                }

                // L'accès à l'abonnement ne se montre qu'au palier libre :
                // rappeler à un abonné qu'il paie n'apporte rien. Sauf compte
                // d'essai connecté : c'est par là qu'on voit avec quelle
                // adresse, et qu'on se déconnecte.
                if subscription.plan == .libre || subscription.compte != nil {
                    ToolbarItem(placement: .topBarLeading) {
                        Button {
                            paywallFor = .recoupement
                        } label: {
                            Label(Localized.text("plan.pro"), systemImage: "sparkles")
                        }
                        .tint(Color.encre)
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
            // À part du chargement, et pas avant lui : la prévision par défaut
            // s'affiche pendant que le système demande l'autorisation.
            .task { viewModel.locateIfUnchosen() }
            // Le guetteur relit à chaque quart d'heure ; il repart de zéro
            // quand la ville change.
            .task(id: viewModel.parcelle) { await viewModel.veiller() }
            // Le direct : le relais pousse la météo dès qu'elle change.
            .task(id: viewModel.parcelle) { viewModel.suivreEnDirect() }
            .sheet(isPresented: $villesOuvertes) {
                VillesView(viewModel: viewModel, subscription: subscription)
            }
            .sheet(item: $paywallFor) { feature in
                PaywallView(subscription: subscription, reason: feature)
            }
            .task {
                // À chaque ouverture : StoreKit est la source de vérité, le
                // stockage partagé n'en est que la copie pour les extensions.
                await subscription.refresh()
            }
            .onChange(of: subscription.plan) { _, plan in
                // Le palier vient de s'ouvrir — achat ou compte d'essai : c'est
                // le moment de demander la permission des alertes et de les
                // poser, plutôt qu'au prochain chargement.
                guard plan.allows(.alertes), let forecast = viewModel.forecast else { return }
                Task { await scheduleAlerts(forecast: forecast) }
            }
        }
    }

    // MARK: Activité en direct

    /// Recharge la prévision, puis répercute les nouvelles conditions sur
    /// l'activité en direct si elle est en cours.
    private func reload() async {
        await viewModel.load()
        guard let forecast = viewModel.forecast else { return }
        await weather.refresh(forecast: forecast)
        await scheduleAlerts(forecast: forecast)
    }

    /// Demande la permission, puis pose les alertes — à chaque chargement.
    ///
    /// Deux défauts, et aucune notification ne pouvait partir. La permission
    /// n'était demandée nulle part : `requestAuthorization()` existait et
    /// personne ne l'appelait, et une notification posée sans autorisation ne
    /// s'affiche jamais. Et les alertes n'étaient examinées qu'au réveil
    /// d'arrière-plan, qu'iOS accorde quand il veut — parfois pas de la
    /// journée.
    ///
    /// La permission ne se demande qu'au palier qui ouvre les alertes : la
    /// réclamer à quelqu'un qui n'en recevra aucune serait gâcher l'unique
    /// question que le système accepte de poser. Après un refus, il ne la
    /// repose pas, et on n'insiste pas.
    private func scheduleAlerts(forecast: AgroForecast) async {
        let plan = subscription.plan
        guard plan.allows(.alertes), await AlertScheduler.requestAuthorization() else { return }
        let state = await AlertScheduler.schedule(
            hours: forecast.hourly,
            plan: plan,
            state: SharedStore.loadAlertState()
        )
        SharedStore.save(state)
    }

    private func toggleWeather(_ forecast: AgroForecast) {
        if weather.isRunning {
            Task { await weather.stop() }
        } else {
            weather.start(parcelle: viewModel.parcelle, forecast: forecast)
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
                            .foregroundStyle(Color.encreDouce)
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 11)
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                Divider().overlay(Color.filet)
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
                .tint(Color.encre)
        }
        .padding(.top, 40)
    }

    private func source(_ forecast: AgroForecast) -> some View {
        VStack(spacing: 4) {
            Text(Localized.text("app.source", AgroFormat.unit(forecast.elevation, "m", decimals: 0)))
            // La licence de Copernicus demande la mention dès qu'une mesure
            // d'air est montrée.
            if subscription.plan.allows(.air), viewModel.air != nil {
                Text(Air.attribution)
            }
        }
        .font(.caption2)
        .foregroundStyle(Color.encreDouce)
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

    private func detailTiles(_ forecast: AgroForecast) -> [TileModel] {
        let zone = viewModel.timeZone
        let today = forecast.daily.first
        let current = forecast.current
        let maintenant = Ville.heure(contenant: current.time, dans: forecast.hourly)
        let uv = maintenant?.uvIndex ?? 0
        let uvMax = today?.uvIndexMax ?? uv

        var tiles: [TileModel] = [
            TileModel(
                label: Localized.text("tile.feelsLike"),
                value: AgroFormat.temperature(current.apparentTemperature),
                caption: Localized.text("tile.feelsLike.caption", AgroFormat.temperature(current.temperature))
            ),
            TileModel(
                label: Localized.text("tile.humidity"),
                value: AgroFormat.percent(current.relativeHumidity),
                caption: maintenant.map {
                    Localized.text("tile.humidity.caption", AgroFormat.temperature($0.dewPoint))
                }
            ),
            TileModel(
                label: Localized.text("tile.wind"),
                value: AgroFormat.unit(current.windSpeed, "km/h", decimals: 0),
                caption: Localized.text(
                    "tile.wind.caption",
                    AgroFormat.unit(current.windGusts, "km/h", decimals: 0)
                )
            ),
            TileModel(
                label: Localized.text("tile.uv"),
                value: "\(AgroFormat.decimal(uv, decimals: 0)) · \(Ville.niveauUv(uv).label)",
                caption: Localized.text(
                    "tile.uv.caption",
                    AgroFormat.decimal(uvMax, decimals: 0),
                    Ville.niveauUv(uvMax).label.lowercased()
                ),
                gauge: (
                    position: uv / 11,
                    colors: [
                        Color(red: 0.494, green: 0.816, blue: 0.478),
                        Color(red: 0.941, green: 0.757, blue: 0.294),
                        Color(red: 0.937, green: 0.541, blue: 0.353),
                        Color(red: 0.851, green: 0.325, blue: 0.310),
                        Color(red: 0.608, green: 0.349, blue: 0.714),
                    ]
                )
            ),
            TileModel(
                label: Localized.text("tile.rainToday"),
                value: AgroFormat.unit(today?.precipitationSum ?? 0, "mm"),
                caption: Localized.text(
                    "tile.rainToday.caption",
                    AgroFormat.percent(today?.precipitationProbabilityMax ?? 0)
                )
            ),
            TileModel(
                label: Localized.text("tile.pressure"),
                value: AgroFormat.unit(current.pressure, "hPa", decimals: 0),
                caption: Localized.text("tile.pressure.caption")
            ),
            TileModel(
                label: Localized.text("tile.sunrise"),
                value: AgroFormat.time(today?.sunrise, in: zone),
                caption: Localized.text("tile.sunrise.caption", AgroFormat.time(today?.sunset, in: zone))
            ),
        ]

        tiles += airTiles()

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

    /// L'air et les pollens : fermés au palier libre, montrés avec ce qu'ils
    /// contiendraient — comme le recoupement.
    private func airTiles() -> [TileModel] {
        guard subscription.plan.allows(.air) else {
            return [TileModel(
                label: Localized.text("tile.air"),
                value: Localized.text("tile.locked"),
                caption: Localized.text(Feature.air.upgradeReasonKey)
            )]
        }
        guard let air = viewModel.air else { return [] }

        var tiles: [TileModel] = []
        if let aqi = air.europeanAqi {
            tiles.append(TileModel(
                label: Localized.text("tile.air"),
                value: Air.qualite(aqi).label,
                caption: Localized.text(
                    "tile.air.caption",
                    AgroFormat.decimal(aqi, decimals: 0),
                    air.pm25.map { AgroFormat.unit($0, "µg/m³", decimals: 0) } ?? "—"
                ),
                gauge: (
                    position: aqi / 100,
                    colors: [
                        Color(red: 0.314, green: 0.941, blue: 0.902),
                        Color(red: 0.314, green: 0.800, blue: 0.667),
                        Color(red: 0.941, green: 0.902, blue: 0.255),
                        Color(red: 1.000, green: 0.314, blue: 0.314),
                        Color(red: 0.588, green: 0.000, blue: 0.196),
                        Color(red: 0.490, green: 0.129, blue: 0.506),
                    ]
                )
            ))
        }
        // Hors d'Europe, pas de pollens prévus : la tuile ne dit rien plutôt
        // que « aucun », qui serait faux.
        if !air.pollens.isEmpty {
            if let dominant = Air.pollenDominant(air) {
                tiles.append(TileModel(
                    label: Localized.text("tile.pollen"),
                    value: dominant.pollen.label,
                    caption: Localized.text(
                        "tile.pollen.caption",
                        AgroFormat.decimal(dominant.grains, decimals: 0),
                        dominant.niveau.label.lowercased()
                    )
                ))
            } else {
                tiles.append(TileModel(
                    label: Localized.text("tile.pollen"),
                    value: Localized.text("tile.pollen.none"),
                    caption: Localized.text("tile.pollen.noneCaption")
                ))
            }
        }
        return tiles
    }
}

#Preview {
    DashboardView()
}

private extension View {
    /// Ce qui défile ne se lit plus sous les boutons du haut ni sous la
    /// recherche.
    ///
    /// La barre était déclarée transparente pour laisser voir le ciel : le
    /// texte glissait alors sous ✦ et ➤ sans rien entre eux, et « Conditions
    /// météo » se lisait à travers les boutons. La barre reprend son
    /// comportement par défaut — transparente en haut de page, voilée dès
    /// qu'un contenu passe dessous —, et depuis iOS 26, le bord franc remplace
    /// le fondu, qui laissait le texte mi-lisible sous les boutons.
    @ViewBuilder
    func bordsNets() -> some View {
        #if compiler(>=6.2)
        if #available(iOS 26.0, *) {
            self.scrollEdgeEffectStyle(.hard, for: [.top, .bottom])
        } else {
            self
        }
        #else
        self
        #endif
    }
}
