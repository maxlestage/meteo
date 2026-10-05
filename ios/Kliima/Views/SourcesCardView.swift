import SwiftUI

/// Ce que dit chaque source, l'une sous l'autre.
///
/// La tuile « Accord des modèles » résume : combien de sources, quel écart,
/// quelle valeur retenue. Elle ne dit pas *qui* annonce quoi — et c'est
/// pourtant ce qu'on regarde quand Météo-France et le modèle américain ne
/// s'entendent pas sur la pluie de l'après-midi. Cette carte le montre.
///
/// Deux lignes par source plutôt qu'une rangée de colonnes : l'institut et sa
/// température d'abord, le modèle, la pluie et le vent ensuite. Une rangée de
/// cinq colonnes ne tenait pas dès que le texte grandit, et la largeur d'un
/// nom d'institut n'est pas celle d'un autre.
struct SourcesCardView: View {
    let consensus: Consensus

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            CardLabel(text: Localized.text("sources.title"))

            VStack(spacing: 0) {
                ForEach(Array(consensus.readings.enumerated()), id: \.element.source.id) { index, reading in
                    if index > 0 {
                        Divider().overlay(Color.filet)
                    }
                    row(reading)
                }
            }

            Text(resume)
                .font(.footnote)
                .foregroundStyle(Color.encreDouce)
                .fixedSize(horizontal: false, vertical: true)
        }
        .cardBackground()
    }

    private func row(_ reading: SourceReading) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(alignment: .firstTextBaseline) {
                Text(reading.source.institution)
                    .font(.body.weight(.medium))
                    .lineLimit(1)
                    .minimumScaleFactor(0.75)
                Spacer(minLength: 8)
                Text(AgroFormat.temperature(reading.temperature))
                    .font(.title3)
                    .monospacedDigit()
                    .lineLimit(1)
                    .foregroundStyle(ecartee(reading) ? Color(red: 0.937, green: 0.541, blue: 0.353) : Color.encre)
            }

            Text(detail(reading))
                .font(.footnote)
                .foregroundStyle(Color.encreDouce)
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding(.vertical, 8)
        .accessibilityElement(children: .combine)
    }

    /// « AROME / ARPEGE · sec · 12 km/h »
    private func detail(_ reading: SourceReading) -> String {
        let pluie = reading.precipitation >= ConsensusThresholds.rainThreshold
            ? AgroFormat.unit(reading.precipitation, "mm")
            : Localized.text("sources.dry")
        return [reading.source.name, pluie, AgroFormat.unit(reading.windSpeed, "km/h", decimals: 0)]
            .joined(separator: " · ")
    }

    /// Une source qui s'écarte de la valeur retenue de plus que l'accord fort
    /// ne le tolère se signale : c'est elle qu'il faut regarder.
    private func ecartee(_ reading: SourceReading) -> Bool {
        abs(reading.temperature - consensus.temperature.median) > ConsensusThresholds.strongTemperatureSpread
    }

    private var resume: String {
        let detail = Localized.text(
            "consensus.detail",
            String(consensus.readings.count),
            AgroFormat.unit(consensus.temperature.spread, "°C")
        )
        let median = Localized.text("consensus.median", AgroFormat.unit(consensus.temperature.median, "°C"))
        return "\(detail). \(median)."
    }
}
