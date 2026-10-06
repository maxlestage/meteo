import SwiftUI

/// Écran unique de la montre, habillé comme l'iPhone : le ciel animé derrière,
/// la température en grand, puis ce qu'on vient chercher au poignet en
/// sortant — le guetteur et sa demi-heure, la pluie qui vient et ce qu'il faut
/// emporter, les heures suivantes, le ressenti, le vent et l'UV.
///
/// La montre est toujours en sombre : le fond reprend les ciels sombres de
/// `SkyBackground`, et `CielVivant` y dessine le même temps qu'il fait.
struct WatchDashboardView: View {
    @StateObject private var viewModel = WatchViewModel()

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 8) {
                    if viewModel.isLoading && viewModel.forecast == nil {
                        ProgressView()
                            .frame(maxWidth: .infinity)
                            .padding(.top, 24)
                    }

                    if let message = viewModel.errorMessage {
                        Text(message)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }

                    if let forecast = viewModel.forecast {
                        WatchHeader(forecast: forecast)
                            .apparition(0)
                        WatchGuetteur(
                            quarts: viewModel.quarts,
                            enLecture: viewModel.veilleEnLecture,
                            timeZone: viewModel.timeZone,
                            tombe: forecast.ciel?.tombe
                        )
                        .apparition(1)
                        WatchPluie(forecast: forecast, timeZone: viewModel.timeZone)
                            .apparition(2)
                        WatchHeures(forecast: forecast, timeZone: viewModel.timeZone)
                            .apparition(3)
                        WatchTuiles(forecast: forecast)
                            .apparition(4)
                    }
                }
                .padding(.horizontal, 2)
                // Une autre ville rejoue l'arrivée des cartes.
                .id(viewModel.forecast?.parcelle)
            }
            .navigationTitle(viewModel.parcelle.name)
            .navigationBarTitleDisplayMode(.inline)
            .containerBackground(for: .navigation) {
                WatchCiel(
                    isDay: viewModel.forecast?.current.isDay ?? true,
                    weatherCode: viewModel.forecast?.current.weatherCode ?? 0
                )
            }
        }
        .task { await viewModel.load() }
        .task { await viewModel.veiller() }
        .refreshable {
            await viewModel.load()
            await viewModel.relireQuarts()
        }
    }
}

// MARK: - Le ciel

/// Le bleu de la pluie, celui du thème sombre de l'iPhone.
private let bleuPluie = Color(red: 0.498, green: 0.816, blue: 0.961)

/// Le fond : le ciel sombre de l'iPhone, et le temps qu'il fait par-dessus.
private struct WatchCiel: View {
    let isDay: Bool
    let weatherCode: Int

    var body: some View {
        ZStack {
            LinearGradient(colors: couleurs, startPoint: .top, endPoint: .bottom)
                .animation(.easeInOut(duration: 1.2), value: couleurs)
            CielVivant(isDay: isDay, weatherCode: weatherCode)
        }
        .ignoresSafeArea()
    }

    /// Les ciels sombres de `SkyBackground` : nuit, couvert, dégagé.
    private var couleurs: [Color] {
        guard isDay else {
            return [
                Color(red: 0.067, green: 0.110, blue: 0.180),
                Color(red: 0.039, green: 0.071, blue: 0.125),
                Color(red: 0.016, green: 0.031, blue: 0.067),
            ]
        }
        if weatherCode >= 45 {
            return [
                Color(red: 0.165, green: 0.208, blue: 0.259),
                Color(red: 0.110, green: 0.141, blue: 0.180),
                Color(red: 0.055, green: 0.071, blue: 0.098),
            ]
        }
        return [
            Color(red: 0.102, green: 0.267, blue: 0.459),
            Color(red: 0.067, green: 0.176, blue: 0.318),
            Color(red: 0.027, green: 0.075, blue: 0.157),
        ]
    }
}

// MARK: - Les cartes

/// Une carte de verre sur le ciel, comme celles de l'iPhone.
private struct Verre: ViewModifier {
    func body(content: Content) -> some View {
        content
            .padding(10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.white.opacity(0.10), in: RoundedRectangle(cornerRadius: 14))
            .overlay(
                RoundedRectangle(cornerRadius: 14)
                    .strokeBorder(Color.white.opacity(0.12), lineWidth: 1)
            )
    }
}

/// Une carte qui arrive : elle monte en apparaissant, un peu après celle
/// d'avant. Pour qui demande moins de mouvement, tout est en place d'emblée.
private struct Apparition: ViewModifier {
    let rang: Int

    @State private var apparu = false
    @Environment(\.accessibilityReduceMotion) private var moinsDeMouvement

    func body(content: Content) -> some View {
        let visible = apparu || moinsDeMouvement
        return content
            .opacity(visible ? 1 : 0)
            .offset(y: visible ? 0 : 14)
            .onAppear {
                guard !apparu else { return }
                withAnimation(.spring(response: 0.55, dampingFraction: 0.85).delay(Double(rang) * 0.08)) {
                    apparu = true
                }
            }
    }
}

private extension View {
    func verre() -> some View { modifier(Verre()) }
    func apparition(_ rang: Int) -> some View { modifier(Apparition(rang: rang)) }
}

/// Intitulé en petites capitales, avec son symbole.
private struct Intitule: View {
    let symbole: String
    let texte: String
    var couleur: Color = .secondary

    var body: some View {
        Label(texte, systemImage: symbole)
            .font(.system(size: 11, weight: .semibold))
            .textCase(.uppercase)
            .foregroundStyle(couleur)
            .labelStyle(.titleAndIcon)
            .lineLimit(1)
    }
}

// MARK: En-tête

private struct WatchHeader: View {
    let forecast: AgroForecast

    @Environment(\.accessibilityReduceMotion) private var moinsDeMouvement

    var body: some View {
        let current = forecast.current
        let condition = WeatherCondition.forCode(current.weatherCode)

        VStack(alignment: .leading, spacing: 2) {
            HStack(alignment: .center, spacing: 6) {
                Text(AgroFormat.temperature(current.temperature))
                    .font(.system(size: 44, weight: .thin, design: .rounded))
                    .contentTransition(.numericText(value: current.temperature))
                    .animation(.snappy, value: current.temperature)
                    .minimumScaleFactor(0.7)
                    .lineLimit(1)

                Spacer(minLength: 0)

                Image(systemName: condition.icon.symbolName(isDay: current.isDay))
                    .symbolRenderingMode(.multicolor)
                    .font(.system(size: 30))
                    .symbolEffect(.pulse, options: .repeating, isActive: !moinsDeMouvement && tombe(condition))
                    .contentTransition(.symbolEffect(.replace))
                    .accessibilityHidden(true)
            }

            HStack(spacing: 6) {
                Text(condition.label)
                    .lineLimit(1)
                Spacer(minLength: 0)
                if let today = forecast.daily.first {
                    Text("↑\(AgroFormat.temperature(today.temperatureMax)) ↓\(AgroFormat.temperature(today.temperatureMin))")
                        .monospacedDigit()
                        .lineLimit(1)
                }
            }
            .font(.caption2)
            .foregroundStyle(.secondary)

            if forecast.sources.count > 1 {
                Label(
                    Localized.text("forecast.blended", String(forecast.sources.count)),
                    systemImage: "checkmark.seal.fill"
                )
                .font(.system(size: 10, weight: .semibold))
                .foregroundStyle(bleuPluie)
                .padding(.horizontal, 7)
                .padding(.vertical, 3)
                .background(bleuPluie.opacity(0.16), in: Capsule())
                .padding(.top, 3)
                .lineLimit(1)
                .minimumScaleFactor(0.8)
            }

            // Ce qu'on voit tomber à l'aéroport le plus proche.
            if let vu = VeilleTextes.vu(forecast.ciel, in: TimeZone(identifier: forecast.timezone) ?? .current) {
                Label(vu, systemImage: "drop.fill")
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(bleuPluie)
                    .fixedSize(horizontal: false, vertical: true)
                    .padding(.top, 3)
            }
        }
        .accessibilityElement(children: .combine)
    }

    private func tombe(_ condition: WeatherCondition) -> Bool {
        switch condition.icon {
        case .drizzle, .rain, .showers, .snow, .thunder: return true
        default: return false
        }
    }
}

// MARK: Guetteur

/// Le guetteur au poignet : la demi-heure en une phrase, les huit quarts en
/// barres, puis ce que disent les deux heures à venir.
private struct WatchGuetteur: View {
    let quarts: [QuartSample]?
    let enLecture: Bool
    let timeZone: TimeZone
    /// Ce que l'aéroport le plus proche voit tomber.
    let tombe: Int?

    /// Le débit qui remplit une barre (mm/h), comme sur l'iPhone.
    private let debitPlein = 8.0
    private let hauteur: CGFloat = 26

    var body: some View {
        TimelineView(.periodic(from: .now, by: 30)) { contexte in
            VStack(alignment: .leading, spacing: 6) {
                HStack(spacing: 4) {
                    Image(systemName: "dot.radiowaves.left.and.right")
                        .symbolEffect(.variableColor.iterative, isActive: enLecture)
                    Text(Localized.text("veille.title"))
                        .textCase(.uppercase)
                }
                .font(.system(size: 11, weight: .semibold))
                .foregroundStyle(bleuPluie)

                if let lecture = quarts.flatMap({
                    Veille.veille(Veille.observer($0, maintenant: contexte.date, tombe: tombe), maintenant: contexte.date)
                }) {
                    Text(VeilleTextes.immediat(lecture, in: timeZone))
                        .font(.footnote)
                        .fixedSize(horizontal: false, vertical: true)
                    barres(lecture)
                    if let suite = VeilleTextes.suite(lecture, in: timeZone).first {
                        Text(suite)
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                } else {
                    Text(Localized.text(enLecture ? "veille.loading" : "veille.unavailable"))
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .verre()
        .accessibilityElement(children: .combine)
    }

    /// Huit barres, une par quart ; la première, le quart en cours, est
    /// cerclée. L'heure du début et celle de la fin bornent la fenêtre.
    private func barres(_ lecture: VeilleLecture) -> some View {
        VStack(spacing: 2) {
            HStack(alignment: .bottom, spacing: 3) {
                ForEach(Array(lecture.quarts.enumerated()), id: \.element.time) { index, quart in
                    ZStack(alignment: .bottom) {
                        RoundedRectangle(cornerRadius: 3)
                            .fill(Color.white.opacity(0.08))
                        RoundedRectangle(cornerRadius: 3)
                            .fill(Veille.mouille(quart) ? bleuPluie : Color.white.opacity(0.22))
                            .frame(height: max(3, hauteur * min(quart.precipitation * 4 / debitPlein, 1)))
                    }
                    .frame(height: hauteur)
                    .overlay {
                        if index == 0 {
                            RoundedRectangle(cornerRadius: 3)
                                .stroke(Color.white.opacity(0.6), lineWidth: 1)
                        }
                    }
                    .frame(maxWidth: .infinity)
                }
            }
            HStack {
                Text(lecture.quarts.first.map { AgroFormat.time($0.time, in: timeZone) } ?? "")
                Spacer()
                Text(AgroFormat.time(lecture.finFenetre, in: timeZone))
            }
            .font(.system(size: 10).monospacedDigit())
            .foregroundStyle(.secondary)
        }
        .accessibilityLabel(Localized.text("veille.chart"))
    }
}

// MARK: Pluie

/// La pluie qui vient, puis ce qu'il faut emporter, en symboles.
private struct WatchPluie: View {
    let forecast: AgroForecast
    let timeZone: TimeZone

    var body: some View {
        let pluie = Ville.prochainePluie(forecast.hourly)
        let conseils = Ville.conseils(forecast.hourly)
        let seche = pluie.code == "aucune"

        VStack(alignment: .leading, spacing: 4) {
            Intitule(symbole: seche ? "umbrella" : "cloud.rain.fill", texte: Localized.text("rain.title"))
            Text(titre(pluie))
                .font(.headline)
                .foregroundStyle(seche ? Color.primary : bleuPluie)
                .fixedSize(horizontal: false, vertical: true)

            if conseils.isEmpty {
                Text(Localized.text("advice.none"))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            } else {
                VStack(alignment: .leading, spacing: 3) {
                    ForEach(conseils, id: \.self) { conseil in
                        Label(conseil.label, systemImage: conseil.symbolName)
                            .font(.caption2)
                            .symbolRenderingMode(.multicolor)
                            .lineLimit(1)
                            .minimumScaleFactor(0.8)
                    }
                }
                .padding(.top, 1)
            }
        }
        .verre()
        .accessibilityElement(children: .combine)
    }

    private func titre(_ pluie: Pluie) -> String {
        switch pluie {
        case .aucune(let heures): return Localized.text("rain.none", String(heures))
        case .enCours(let fin?): return Localized.text("rain.now", AgroFormat.hour(fin, in: timeZone))
        case .enCours(.none): return Localized.text("rain.nowLasting")
        case .prevue(let debut, _, _): return Localized.text("rain.soon", AgroFormat.hour(debut, in: timeZone))
        }
    }
}

// MARK: Heures

/// Les heures qui viennent, sur la largeur du cadran : rien ne défile. La
/// première colonne est l'instant présent, comme sur l'iPhone.
private struct WatchHeures: View {
    let forecast: AgroForecast
    let timeZone: TimeZone

    private let colonnes = 4

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Intitule(symbole: "clock", texte: Localized.text("hourly.title"))
            HStack(alignment: .top, spacing: 2) {
                ForEach(Array(forecast.hourly.prefix(colonnes).enumerated()), id: \.element.time) { index, heure in
                    colonne(heure, maintenant: index == 0)
                        .frame(maxWidth: .infinity)
                }
            }
        }
        .verre()
    }

    private func colonne(_ heure: HourlySample, maintenant: Bool) -> some View {
        let current = forecast.current
        let code = maintenant ? current.weatherCode : heure.weatherCode
        let isDay = maintenant ? current.isDay : heure.isDay
        let temperature = maintenant ? current.temperature : heure.temperature

        return VStack(spacing: 3) {
            Text(maintenant ? Localized.text("hourly.now") : AgroFormat.hour(heure.time, in: timeZone))
                .font(.system(size: 10, weight: .semibold))
                .foregroundStyle(.secondary)
                .lineLimit(1)
                .minimumScaleFactor(0.7)
            Image(systemName: WeatherCondition.forCode(code).icon.symbolName(isDay: isDay))
                .symbolRenderingMode(.multicolor)
                .font(.system(size: 15))
                .frame(height: 18)
            Text(AgroFormat.temperature(temperature))
                .font(.system(size: 13, weight: .medium).monospacedDigit())
                .lineLimit(1)
                .minimumScaleFactor(0.7)
            Text(heure.precipitationProbability >= 20 ? AgroFormat.percent(heure.precipitationProbability) : " ")
                .font(.system(size: 9).monospacedDigit())
                .foregroundStyle(bleuPluie)
                .lineLimit(1)
        }
    }
}

// MARK: Tuiles

/// Ressenti, vent, UV et humidité : quatre tuiles sur deux colonnes.
private struct WatchTuiles: View {
    let forecast: AgroForecast

    var body: some View {
        let current = forecast.current
        let grille = [GridItem(.flexible(), spacing: 6), GridItem(.flexible(), spacing: 6)]

        LazyVGrid(columns: grille, spacing: 6) {
            Tuile(
                symbole: "thermometer.medium",
                titre: Localized.text("tile.feelsLike"),
                valeur: AgroFormat.temperature(current.apparentTemperature),
                detail: nil
            )
            Tuile(
                symbole: "wind",
                titre: Localized.text("tile.wind"),
                valeur: AgroFormat.unit(current.windSpeed, "km/h", decimals: 0),
                detail: "↗ " + AgroFormat.unit(current.windGusts, "km/h", decimals: 0)
            )
            if let today = forecast.daily.first {
                Tuile(
                    symbole: "sun.max.fill",
                    titre: Localized.text("tile.uv"),
                    valeur: AgroFormat.decimal(today.uvIndexMax, decimals: 0),
                    detail: Ville.niveauUv(today.uvIndexMax).label
                )
            }
            Tuile(
                symbole: "humidity.fill",
                titre: Localized.text("tile.humidity"),
                valeur: AgroFormat.percent(current.relativeHumidity),
                detail: nil
            )
        }
    }
}

private struct Tuile: View {
    let symbole: String
    let titre: String
    let valeur: String
    let detail: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Intitule(symbole: symbole, texte: titre)
                .minimumScaleFactor(0.7)
            Text(valeur)
                .font(.system(size: 17, weight: .medium, design: .rounded).monospacedDigit())
                .lineLimit(1)
                .minimumScaleFactor(0.6)
            Text(detail ?? " ")
                .font(.system(size: 10))
                .foregroundStyle(.secondary)
                .lineLimit(1)
                .minimumScaleFactor(0.7)
        }
        .frame(maxWidth: .infinity, minHeight: 58, alignment: .topLeading)
        .verre()
        .accessibilityElement(children: .combine)
    }
}
