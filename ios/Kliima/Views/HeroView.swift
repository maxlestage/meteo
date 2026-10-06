import SwiftUI

/// En-tête : commune, température, temps et amplitude du jour.
///
/// Comme l'en-tête web : un grand pictogramme animé — gouttes qui tombent,
/// rayons qui pulsent —, et une température qui défile quand elle change.
struct HeroView: View {
    let forecast: AgroForecast

    /// Le fuseau de la ville : les heures sont les siennes.
    private var zone: TimeZone { TimeZone(identifier: forecast.timezone) ?? .current }

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

            // La prévision n'est pas celle d'un modèle : toutes les sources
            // l'ont faite, et on le dit.
            if !forecast.sources.isEmpty {
                Label {
                    Text(Localized.text("forecast.blended", String(forecast.sources.count)))
                } icon: {
                    Circle()
                        .fill(Color(red: 0.494, green: 0.816, blue: 0.478))
                        .frame(width: 7, height: 7)
                }
                .font(.footnote.weight(.semibold))
                .padding(.horizontal, 12)
                .padding(.vertical, 5)
                .background(.ultraThinMaterial.opacity(0.6), in: Capsule())
                .overlay(Capsule().strokeBorder(Color.filet, lineWidth: 1))
                .padding(.top, 8)
                .transition(.opacity.combined(with: .scale(scale: 0.9)))
            }

            // Ce qu'on voit tomber à l'aéroport le plus proche : une
            // observation, pas une prévision — et sa source.
            if let vu = VeilleTextes.vu(forecast.ciel, in: zone) {
                VStack(spacing: 3) {
                    Label(vu, systemImage: "drop.fill")
                        .font(.subheadline.weight(.semibold))
                        .symbolEffect(.pulse, options: .repeating, isActive: !moinsDeMouvement)
                        .multilineTextAlignment(.center)
                    Text(Localized.text("ciel.credit"))
                        .font(.caption2)
                        .foregroundStyle(Color.encreDouce)
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 9)
                .background(Color.bleuPluie.opacity(0.16), in: RoundedRectangle(cornerRadius: 16))
                .overlay(RoundedRectangle(cornerRadius: 16).strokeBorder(Color.bleuPluie.opacity(0.45), lineWidth: 1))
                .padding(.top, 10)
                .padding(.horizontal, 8)
                .transition(.opacity.combined(with: .scale(scale: 0.95)))
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.top, 8)
        .accessibilityElement(children: .combine)
    }
}
