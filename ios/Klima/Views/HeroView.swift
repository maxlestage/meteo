import SwiftUI

/// En-tête : commune, température, temps et amplitude du jour.
struct HeroView: View {
    let forecast: AgroForecast

    var body: some View {
        let condition = WeatherCondition.forCode(forecast.current.weatherCode)

        VStack(spacing: 2) {
            Text(forecast.parcelle.name)
                .font(.system(size: 34, weight: .regular))
                .lineLimit(1)
                .minimumScaleFactor(0.6)

            Text("\(Int(forecast.current.temperature.rounded()))°")
                .font(.system(size: 96, weight: .thin))
                .padding(.top, -6)

            Text(condition.label)
                .font(.title3)
                .foregroundStyle(.white.opacity(0.62))

            if let today = forecast.daily.first {
                Text("↑ \(Int(today.temperatureMax.rounded()))°   ↓ \(Int(today.temperatureMin.rounded()))°")
                    .font(.title3)
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.top, 8)
        .accessibilityElement(children: .combine)
    }
}
