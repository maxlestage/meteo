import SwiftUI

/// La pluie qui vient, et ce qu'il faut emporter.
///
/// La première question qu'on pose à une météo de ville : faut-il un
/// parapluie, et jusqu'à quand. La carte y répond en une phrase, la montre
/// sur douze barres — une par heure, aussi hautes que le risque —, puis dit
/// ce qu'il faut prendre avant de sortir. Miroir de `composants/pluie.rs`.
///
/// Les douze barres tiennent dans la largeur de la carte : rien ne défile.
/// Les conseils vont un par ligne, pour qu'un grand texte ne les coupe pas.
struct PluieCardView: View {
    let hours: [HourlySample]
    let timeZone: TimeZone

    @ScaledMetric(relativeTo: .body) private var hauteurBarres: CGFloat = 44

    private var pluie: Pluie { Ville.prochainePluie(hours) }
    private var conseils: [Conseil] { Ville.conseils(hours) }
    private var douze: [HourlySample] { Array(hours.prefix(VilleSeuils.horizonPluie)) }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            CardLabel(text: Localized.text("rain.title"))

            Text(titre)
                .font(.title2)
                .fixedSize(horizontal: false, vertical: true)

            if let detail {
                Text(detail)
                    .font(.subheadline)
                    .foregroundStyle(Color.encreDouce)
            }

            HStack(alignment: .bottom, spacing: 3) {
                ForEach(douze) { heure in
                    RoundedRectangle(cornerRadius: 3)
                        .fill(Ville.pluvieuse(heure)
                              ? Color(red: 0.498, green: 0.816, blue: 0.961)
                              : Color.filet)
                        .frame(height: max(4, hauteurBarres * min(heure.precipitationProbability, 100) / 100))
                        .frame(maxWidth: .infinity)
                }
            }
            .frame(height: hauteurBarres, alignment: .bottom)
            .padding(.top, 10)
            .accessibilityHidden(true)

            HStack {
                Text(Localized.text("hourly.now"))
                Spacer()
                Text("+6 h")
                Spacer()
                Text("+12 h")
            }
            .font(.caption2)
            .foregroundStyle(Color.encreDouce)
            .lineLimit(1)
            .padding(.top, 4)

            CardLabel(text: Localized.text("advice.title"))
                .padding(.top, 14)

            if conseils.isEmpty {
                Text(Localized.text("advice.none"))
                    .font(.subheadline)
                    .foregroundStyle(Color.encreDouce)
            } else {
                VStack(alignment: .leading, spacing: 6) {
                    ForEach(conseils, id: \.self) { conseil in
                        Label(conseil.label, systemImage: conseil.symbolName)
                            .font(.subheadline)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                }
                .padding(.top, 2)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardBackground()
    }

    private var titre: String {
        switch pluie {
        case .aucune(let heures):
            return Localized.text("rain.none", String(heures))
        case .enCours(let fin?):
            return Localized.text("rain.now", AgroFormat.hour(fin, in: timeZone))
        case .enCours(.none):
            return Localized.text("rain.nowLasting")
        case .prevue(let debut, _, _):
            return Localized.text("rain.soon", AgroFormat.hour(debut, in: timeZone))
        }
    }

    private var detail: String? {
        guard case let .prevue(_, probabilite, cumul) = pluie else { return nil }
        return Localized.text("rain.detail", AgroFormat.percent(probabilite), AgroFormat.unit(cumul, "mm"))
    }
}
