import SwiftUI

/// Bandeau horaire sur 24 h : heure, temps, probabilité de pluie, température.
struct HourlyStripView: View {
    let hours: [HourlySample]
    /// Conditions observées, affichées sur la colonne « Maintenant ».
    let current: CurrentSample
    let timeZone: TimeZone

    /// Largeur d'une colonne, qui suit la taille de texte choisie par la
    /// personne.
    ///
    /// Figée à 58 points, elle allait pour le réglage par défaut et pour lui
    /// seul : un cran au-dessus, « Maint. » ne tenait plus sur une ligne,
    /// passait à la ligne, et poussait toute sa colonne d'un cran vers le bas
    /// pendant que les autres restaient en place. `ScaledMetric` fait grandir
    /// la colonne avec le texte, ce qui traite la cause plutôt que le
    /// symptôme.
    @ScaledMetric private var largeurColonne: CGFloat = 58

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
                // Aligné en haut : si une colonne devient malgré tout plus
                // haute que ses voisines, elles gardent la même ligne de
                // départ au lieu de se recentrer contre elle.
                HStack(alignment: .top, spacing: 4) {
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
                .lineLimit(1)
                .minimumScaleFactor(0.7)
                // « Maint. » se lit « maint point » : VoiceOver dit le mot.
                .accessibilityLabel(isFirst ? Localized.text("hourly.now.a11y") : AgroFormat.hour(hour.time, in: timeZone))

            // Sans libellé, VoiceOver lisait le nom anglais du symbole.
            Image(systemName: condition.icon.symbolName(isDay: isDay))
                .symbolRenderingMode(.multicolor)
                .font(.system(size: 20))
                .frame(height: 24)
                .accessibilityLabel(condition.label(isDay: isDay))

            Text(hour.precipitationProbability >= 10
                 ? AgroFormat.percent(hour.precipitationProbability)
                 : " ")
                .font(.caption2.weight(.semibold))
                .foregroundStyle(Color(red: 0.498, green: 0.816, blue: 0.961))
                .lineLimit(1)
                // Un pourcentage nu ne dit pas de quoi il est le risque.
                .accessibilityLabel(Localized.text("rain.chance.a11y", AgroFormat.percent(hour.precipitationProbability)))
                .accessibilityHidden(hour.precipitationProbability < 10)

            Text(AgroFormat.temperature(temperature))
                .font(.title3)
                .lineLimit(1)
                .minimumScaleFactor(0.7)
        }
        .frame(width: largeurColonne)
        .accessibilityElement(children: .combine)
    }
}
