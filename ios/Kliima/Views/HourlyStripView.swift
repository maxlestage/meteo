import SwiftUI

/// Bandeau horaire sur 24 h : heure, temps, probabilité de pluie, température.
struct HourlyStripView: View {
    let hours: [HourlySample]
    /// Conditions observées, affichées sur la colonne « Maintenant ».
    let current: CurrentSample
    let timeZone: TimeZone

    /// Largeur d'une colonne. Vingt-quatre d'entre elles font environ quinze
    /// cents points : c'est ce qu'on fait glisser.
    private static let largeurColonne: CGFloat = 58

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            CardLabel(text: Localized.text("hourly.title"))

            // Un bandeau qui glisse, et qui le montre.
            //
            // Il a été une grille repliable entre-temps : tout voir d'un coup
            // évitait de faire glisser, mais la carte prenait la moitié de
            // l'écran et la dernière rangée finissait ébréchée, deux colonnes
            // seules sous cinq. Maxime Nathan Lestage a tranché pour le
            // bandeau.
            //
            // Une chose ne revient pas : l'indicateur caché. La première
            // version défilait sans rien dire, on voyait six heures et il
            // fallait deviner que les autres existaient. Il est visible
            // désormais, et le rembourrage latéral laisse une colonne entamée
            // au bord — deux façons de dire « ça continue ».
            ScrollView(.horizontal) {
                HStack(spacing: 4) {
                    ForEach(Array(hours.prefix(24).enumerated()), id: \.element.id) { index, hour in
                        column(for: hour, isFirst: index == 0)
                    }
                }
                .padding(.horizontal, 2)
            }
            .scrollIndicators(.visible)
            .scrollBounceBehavior(.basedOnSize)
        }
        .cardBackground()
    }

    private func column(for hour: HourlySample, isFirst: Bool) -> some View {
        // La première colonne montre le relevé courant, pas la prévision de
        // l'heure déjà entamée.
        let code = isFirst ? current.weatherCode : hour.weatherCode
        let condition = WeatherCondition.forCode(code)
        let isDay = isFirst ? current.isDay : hour.isDay
        let temperature = isFirst ? current.temperature : hour.temperature

        return VStack(spacing: 7) {
            Text(isFirst ? Localized.text("hourly.now") : AgroFormat.hour(hour.time, in: timeZone))
                .font(.subheadline.weight(.semibold))

            Image(systemName: condition.icon.symbolName(isDay: isDay))
                .symbolRenderingMode(.multicolor)
                .font(.system(size: 20))
                .frame(height: 24)

            Text(hour.precipitationProbability >= 10
                 ? AgroFormat.percent(hour.precipitationProbability)
                 : " ")
                .font(.caption2.weight(.semibold))
                .foregroundStyle(Color(red: 0.498, green: 0.816, blue: 0.961))

            Text(AgroFormat.temperature(temperature))
                .font(.title3)
        }
        .frame(width: Self.largeurColonne)
        .accessibilityElement(children: .combine)
    }
}
