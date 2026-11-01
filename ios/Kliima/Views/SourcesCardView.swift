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
    /// Ce que l'apprenti a appris des modèles, quand le relais l'a dit.
    var apprentissage: Apprentissage? = nil

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

            if let apprentissage, !apprentissage.notes.isEmpty {
                Divider().overlay(Color.filet)
                VStack(alignment: .leading, spacing: 4) {
                    Label(Localized.text("apprenti.title"), systemImage: "graduationcap")
                        .font(.footnote.weight(.semibold))
                        .foregroundStyle(Color.encre)
                    ForEach(Self.phrases(apprentissage), id: \.self) { phrase in
                        Text(phrase)
                            .font(.footnote)
                            .foregroundStyle(Color.encreDouce)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                }
                .accessibilityElement(children: .combine)
            }
        }
        .cardBackground()
    }

    /// Le nom d'une source d'après son identifiant : l'institut.
    private static func institut(_ id: String) -> String {
        WeatherProviders.openMeteoSources.first { $0.id == id }?.institution ?? id
    }

    /// Qui a jugé, le plus juste, ce qui a été corrigé, et la pluie — comme la
    /// carte des sources du web.
    static func phrases(_ appris: Apprentissage) -> [String] {
        var phrases = [Localized.text("apprenti.juge", appris.nom, String(appris.jours))]
        if let juste = Apprenti.meilleure(appris.notes) {
            phrases.append(Localized.text("apprenti.meilleure", institut(juste.sourceId), AgroFormat.unit(juste.ecart, "°C")))
        }
        let corrigees = appris.notes.filter { $0.correction != 0 }.map { note in
            Localized.text(
                note.correction > 0 ? "apprenti.tropChaud" : "apprenti.tropFroid",
                institut(note.sourceId),
                AgroFormat.unit(abs(note.correction), "°C")
            )
        }
        phrases += corrigees.isEmpty ? [Localized.text("apprenti.aucun")] : corrigees
        if let pluie = appris.notes
            .filter({ $0.pluieN >= ApprentiSeuils.relevesMin })
            .max(by: { ($0.justessePluie ?? 0) < ($1.justessePluie ?? 0) }),
           let justesse = pluie.justessePluie {
            phrases.append(Localized.text("apprenti.pluie", institut(pluie.sourceId), AgroFormat.unit(justesse, "%", decimals: 0)))
        }
        return phrases
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
