import SwiftUI

/// En-tête : commune, température, temps et amplitude du jour.
///
/// Comme l'en-tête web : un grand pictogramme animé — gouttes qui tombent,
/// rayons qui pulsent —, et une température qui défile quand elle change.
struct HeroView: View {
    let forecast: AgroForecast

    @State private var flotte = false
    @Environment(\.accessibilityReduceMotion) private var moinsDeMouvement

    var body: some View {
        let condition = WeatherCondition.forCode(forecast.current.weatherCode)

        VStack(spacing: 2) {
            Text(forecast.parcelle.name)
                .font(.system(size: 34, weight: .regular))
                .lineLimit(1)
                .minimumScaleFactor(0.6)

            Image(systemName: condition.icon.symbolName(isDay: forecast.current.isDay))
                .symbolRenderingMode(.multicolor)
                .font(.system(size: 62))
                .symbolEffect(.variableColor.iterative.reversing, options: .repeating, isActive: !moinsDeMouvement)
                .offset(y: flotte ? -3 : 3)
                .shadow(color: .black.opacity(0.15), radius: 12, y: 6)
                .padding(.top, 6)
                .accessibilityHidden(true)
                .onAppear {
                    guard !moinsDeMouvement else { return }
                    withAnimation(.easeInOut(duration: 3).repeatForever(autoreverses: true)) { flotte = true }
                }

            Text(AgroFormat.temperature(forecast.current.temperature))
                .font(.system(size: 96, weight: .thin))
                .contentTransition(.numericText(value: forecast.current.temperature))
                .animation(.snappy, value: forecast.current.temperature)
                .padding(.top, -6)

            Text(condition.label)
                .font(.title3)
                .foregroundStyle(Color.encreDouce)

            if let today = forecast.daily.first {
                Text("↑ \(AgroFormat.temperature(today.temperatureMax))   ↓ \(AgroFormat.temperature(today.temperatureMin))")
                    .font(.title3)
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.top, 8)
        .accessibilityElement(children: .combine)
    }
}
