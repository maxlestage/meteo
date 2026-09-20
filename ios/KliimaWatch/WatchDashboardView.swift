import SwiftUI

/// Écran unique de la montre : la température, puis les trois réponses qu'on
/// vient chercher au champ — peut-on traiter, le sol porte-t-il, gèlera-t-il.
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

                    if let forecast = viewModel.forecast, let summary = viewModel.summary {
                        header(forecast)
                        sprayRow(summary)
                        WatchRow(
                            title: Localized.text("tile.wind"),
                            value: AgroFormat.unit(forecast.current.windSpeed, "km/h", decimals: 0),
                            detail: AgroFormat.unit(forecast.current.windGusts, "km/h", decimals: 0)
                        )
                        WatchRow(
                            title: Localized.text("tile.soil"),
                            value: summary.soil.state.label,
                            detail: AgroFormat.percent(summary.soil.moisture * 100)
                        )
                        WatchRow(
                            title: Localized.text("tile.frost"),
                            value: summary.frost.severity.label,
                            detail: AgroFormat.unit(summary.frost.minTemperature, "°C")
                        )
                        WatchRow(
                            title: Localized.text("tile.water"),
                            value: AgroFormat.signedUnit(summary.water.balance, "mm"),
                            detail: summary.water.status.label
                        )
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

    /// La fenêtre de traitement mérite sa propre carte : c'est la question qui
    /// fait sortir la montre.
    private func sprayRow(_ summary: AgroSummary) -> some View {
        let zone = viewModel.timeZone

        return VStack(alignment: .leading, spacing: 2) {
            Text(Localized.text("spray.title"))
                .font(.system(size: 11, weight: .semibold))
                .textCase(.uppercase)
                .foregroundStyle(.secondary)

            if let spray = summary.nextSpray {
                Text("\(AgroFormat.hour(spray.start, in: zone)) → \(AgroFormat.hour(spray.end, in: zone))")
                    .font(.title3)
                    .foregroundStyle(spray.score >= 80 ? Color.green : Color.orange)
                Text(Localized.text("spray.score", String(spray.score)))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            } else {
                Text(Localized.text("spray.none"))
                    .font(.headline)
                    .foregroundStyle(.orange)
            }
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
