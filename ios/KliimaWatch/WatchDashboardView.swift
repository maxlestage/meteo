import SwiftUI

/// Écran unique de la montre : la température, puis ce qu'on vient chercher
/// au poignet en sortant — la pluie qui vient, ce qu'il faut emporter, le
/// ressenti et le vent.
struct WatchDashboardView: View {
    @StateObject private var viewModel = WatchViewModel()

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 10) {
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
                        header(forecast)
                        pluieRow(forecast)
                        WatchRow(
                            title: Localized.text("tile.feelsLike"),
                            value: AgroFormat.temperature(forecast.current.apparentTemperature),
                            detail: AgroFormat.percent(forecast.current.relativeHumidity)
                        )
                        WatchRow(
                            title: Localized.text("tile.wind"),
                            value: AgroFormat.unit(forecast.current.windSpeed, "km/h", decimals: 0),
                            detail: AgroFormat.unit(forecast.current.windGusts, "km/h", decimals: 0)
                        )
                        if let today = forecast.daily.first {
                            WatchRow(
                                title: Localized.text("tile.uv"),
                                value: Ville.niveauUv(today.uvIndexMax).label,
                                detail: AgroFormat.decimal(today.uvIndexMax, decimals: 0)
                            )
                        }
                    }
                }
                .padding(.horizontal, 4)
            }
            .navigationTitle(viewModel.parcelle.name)
            .navigationBarTitleDisplayMode(.inline)
        }
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
    }

    private func header(_ forecast: AgroForecast) -> some View {
        let condition = WeatherCondition.forCode(forecast.current.weatherCode)

        return HStack(alignment: .center, spacing: 8) {
            Image(systemName: condition.icon.symbolName(isDay: forecast.current.isDay))
                .symbolRenderingMode(.multicolor)
                .font(.system(size: 26))

            VStack(alignment: .leading, spacing: 0) {
                Text(AgroFormat.temperature(forecast.current.temperature))
                    .font(.system(size: 34, weight: .light))
                Text(condition.label)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            if let today = forecast.daily.first {
                VStack(alignment: .trailing, spacing: 1) {
                    Text("↑ \(AgroFormat.temperature(today.temperatureMax))")
                    Text("↓ \(AgroFormat.temperature(today.temperatureMin))")
                }
                .font(.caption2)
                .foregroundStyle(.secondary)
            }
        }
    }

    /// La pluie mérite sa propre carte : c'est la question qui fait sortir
    /// la montre. Puis ce qu'il faut emporter, en une ligne.
    private func pluieRow(_ forecast: AgroForecast) -> some View {
        let zone = viewModel.timeZone
        let pluie = Ville.prochainePluie(forecast.hourly)
        let conseils = Ville.conseils(forecast.hourly)

        let titre: String
        switch pluie {
        case .aucune(let heures): titre = Localized.text("rain.none", String(heures))
        case .enCours(let fin?): titre = Localized.text("rain.now", AgroFormat.hour(fin, in: zone))
        case .enCours(.none): titre = Localized.text("rain.nowLasting")
        case .prevue(let debut, _, _): titre = Localized.text("rain.soon", AgroFormat.hour(debut, in: zone))
        }

        return VStack(alignment: .leading, spacing: 2) {
            Text(Localized.text("rain.title"))
                .font(.system(size: 11, weight: .semibold))
                .textCase(.uppercase)
                .foregroundStyle(.secondary)
            Text(titre)
                .font(.headline)
                .foregroundStyle(pluie.code == "aucune" ? Color.primary : Color(red: 0.498, green: 0.816, blue: 0.961))
                .fixedSize(horizontal: false, vertical: true)
            Text(conseils.isEmpty
                 ? Localized.text("advice.none")
                 : conseils.map(\.label).joined(separator: " · "))
                .font(.caption2)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(Color.white.opacity(0.12), in: RoundedRectangle(cornerRadius: 12))
    }
}

/// Ligne compacte : un intitulé, une valeur, un complément.
private struct WatchRow: View {
    let title: String
    let value: String
    let detail: String

    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            VStack(alignment: .leading, spacing: 1) {
                Text(title)
                    .font(.system(size: 11, weight: .semibold))
                    .textCase(.uppercase)
                    .foregroundStyle(.secondary)
                Text(value)
                    .font(.body)
            }
            Spacer()
            Text(detail)
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 2)
    }
}
