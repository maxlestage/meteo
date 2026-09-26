import SwiftUI

/// Bandeau horaire sur 24 h : heure, temps, probabilité de pluie, température.
struct HourlyStripView: View {
    let hours: [HourlySample]
    /// Conditions observées, affichées sur la colonne « Maintenant ».
    let current: CurrentSample
    let timeZone: TimeZone

    /// Autant de colonnes que la largeur en accepte, jamais plus étroites que
    /// ce qu'une température lisible demande.
    private static let colonnes = [GridItem(.adaptive(minimum: 52), spacing: 4)]

    /// Ce qu'on montre sans rien demander : deux rangées sur un iPhone.
    ///
    /// Les vingt-quatre heures repliées en grille tenaient dans la carte, mais
    /// la carte tenait tout l'écran — la recherche de commune et la prévision à
    /// sept jours passaient sous la ligne de flottaison. Une demi-journée
    /// répond à la question qu'on se pose en ouvrant l'application ; le reste
    /// se déplie.
    private static let apercu = 12

    @State private var deplie = false

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            CardLabel(text: Localized.text("hourly.title"))

            // Les vingt-quatre heures tiennent dans la carte, en autant de
            // rangées qu'il faut.
            //
            // C'était un défilement horizontal sans indicateur : on voyait six
            // heures et il fallait deviner que les autres existaient. Une
            // grille qui se replie les montre toutes, et ne demande que le
            // geste qu'on fait déjà pour lire l'écran.
            LazyVGrid(columns: Self.colonnes, alignment: .leading, spacing: 14) {
                ForEach(Array(visibles.enumerated()), id: \.element.id) { index, hour in
                    column(for: hour, isFirst: index == 0)
                }
            }

            if hours.count > Self.apercu {
                Button {
                    withAnimation(.snappy) { deplie.toggle() }
                } label: {
                    Label(
                        Localized.text(deplie ? "hourly.fold" : "hourly.unfold"),
                        systemImage: deplie ? "chevron.up" : "chevron.down"
                    )
                    .font(.footnote.weight(.semibold))
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.top, 2)
            }
        }
        .cardBackground()
    }

    /// Les heures affichées : un aperçu, ou les vingt-quatre.
    private var visibles: [HourlySample] {
        Array(hours.prefix(deplie ? 24 : Self.apercu))
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
        .frame(maxWidth: .infinity)
        .accessibilityElement(children: .combine)
    }
}
