import SwiftUI

/// Le guetteur : la demi-heure en cours et les deux heures à venir, dites
/// comme un message qu'on reçoit.
///
/// Miroir de `rust/klima-ui/src/composants/guetteur.rs`. Deux bulles — la
/// demi-heure, puis les deux heures —, les huit quarts sur une grille qui
/// tient dans la largeur (rien ne défile), et l'heure de la lecture. La vue
/// se redessine toutes les trente secondes : le quart en cours avance même
/// sans nouvelle lecture ; c'est le modèle de vue qui relit, à chaque quart.
struct GuetteurCardView: View {
    let quarts: [QuartSample]?
    let luA: Date?
    let enLecture: Bool
    let timeZone: TimeZone

    @ScaledMetric(relativeTo: .body) private var hauteurBarres: CGFloat = 48

    /// Le débit qui remplit une barre (mm/h).
    private let debitPlein = 8.0
    private let bleuPluie = Color.bleuPluie

    var body: some View {
        TimelineView(.periodic(from: .now, by: 30)) { contexte in
            VStack(alignment: .leading, spacing: 10) {
                tete

                if let lecture = quarts.flatMap({ Veille.veille($0, maintenant: contexte.date) }) {
                    bulle(Localized.text("veille.now"), immediat(lecture))
                    bulle(Localized.text("veille.next"), suite(lecture))
                    grille(lecture)
                } else {
                    Text(Localized.text(enLecture ? "veille.loading" : "veille.unavailable"))
                        .font(.subheadline)
                        .foregroundStyle(Color.encreDouce)
                        .padding(12)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color.filet.opacity(0.6), in: bulleForme)
                }

                if let luA, quarts != nil {
                    Text(Localized.text(
                        "veille.checked",
                        AgroFormat.time(luA, in: timeZone),
                        AgroFormat.time(Veille.prochaineLecture(luA), in: timeZone)
                    ))
                    .font(.caption)
                    .foregroundStyle(Color.encreDouce)
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .cardBackground()
        .accessibilityElement(children: .combine)
    }

    // MARK: Morceaux

    private var tete: some View {
        HStack(spacing: 12) {
            Image(systemName: "dot.radiowaves.left.and.right")
                .font(.title3)
                .foregroundStyle(bleuPluie)
                .frame(width: 40, height: 40)
                .background(Color.filet, in: Circle())
                .symbolEffect(.variableColor.iterative, isActive: enLecture)
                .accessibilityHidden(true)

            VStack(alignment: .leading, spacing: 2) {
                Text(Localized.text("veille.title"))
                    .font(.headline)
                Text(Localized.text("veille.subtitle"))
                    .font(.caption)
                    .foregroundStyle(Color.encreDouce)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Spacer(minLength: 0)

            Circle()
                .fill(Color(red: 0.494, green: 0.816, blue: 0.478))
                .frame(width: 9, height: 9)
                .opacity(enLecture ? 0.4 : 1)
                .animation(.easeInOut(duration: 0.6).repeatForever(), value: enLecture)
                .accessibilityHidden(true)
        }
    }

    private var bulleForme: UnevenRoundedRectangle {
        UnevenRoundedRectangle(
            topLeadingRadius: 4, bottomLeadingRadius: 16, bottomTrailingRadius: 16, topTrailingRadius: 16
        )
    }

    private func bulle(_ quand: String, _ texte: String) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(quand.uppercased())
                .font(.caption2.weight(.semibold))
                .kerning(0.6)
                .foregroundStyle(bleuPluie)
            Text(texte)
                .font(.subheadline)
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding(.horizontal, 13)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.filet.opacity(0.6), in: bulleForme)
    }

    /// Huit colonnes, une par quart ; une heure sur deux est écrite.
    private func grille(_ lecture: VeilleLecture) -> some View {
        HStack(alignment: .bottom, spacing: 4) {
            ForEach(Array(lecture.quarts.enumerated()), id: \.element.time) { index, quart in
                VStack(spacing: 4) {
                    ZStack(alignment: .bottom) {
                        RoundedRectangle(cornerRadius: 4)
                            .fill(Color.filet.opacity(0.5))
                        RoundedRectangle(cornerRadius: 4)
                            .fill(Veille.mouille(quart) ? bleuPluie : Color.filet)
                            .frame(height: max(4, hauteurBarres * min(quart.precipitation * 4 / debitPlein, 1)))
                            .pousse(index)
                    }
                    .frame(height: hauteurBarres)
                    .overlay {
                        if index == 0 {
                            RoundedRectangle(cornerRadius: 4)
                                .stroke(Color.encre.opacity(0.55), lineWidth: 1.5)
                        }
                    }

                    Text(index.isMultiple(of: 2) ? AgroFormat.time(quart.time, in: timeZone) : " ")
                        .font(.caption2.monospacedDigit())
                        .foregroundStyle(Color.encreDouce)
                        .lineLimit(1)
                        .minimumScaleFactor(0.7)
                }
                .frame(maxWidth: .infinity)
            }
        }
        .padding(.top, 4)
        .accessibilityLabel(Localized.text("veille.chart"))
    }

    // MARK: Phrases

    private func heure(_ date: Date) -> String { AgroFormat.time(date, in: timeZone) }

    private func immediat(_ lecture: VeilleLecture) -> String {
        switch lecture.immediat {
        case .sec:
            return Localized.text("veille.now.dry")
        case let .commence(debut, precipitation):
            return Localized.text("veille.now.starts", heure(debut), precipitation.label)
        case let .dure(precipitation):
            return Localized.text("veille.now.continues", precipitation.label)
        case let .cesse(fin):
            return Localized.text("veille.now.stops", heure(fin))
        }
    }

    private func suite(_ lecture: VeilleLecture) -> String {
        let fin = heure(lecture.finFenetre)
        var phrases: [String] = []

        switch lecture.suite {
        case .sec:
            phrases.append(Localized.text("veille.next.dry", fin))
        case let .episode(debut, arret?, precipitation, cumul):
            phrases.append(Localized.text(
                "veille.next.episode", heure(debut), heure(arret), precipitation.label, AgroFormat.unit(cumul, "mm")
            ))
        case let .episode(debut, .none, precipitation, _):
            phrases.append(Localized.text("veille.next.episodeOpen", heure(debut), fin, precipitation.label))
        case let .persiste(precipitation, cumul):
            phrases.append(Localized.text("veille.next.persists", fin, precipitation.label, AgroFormat.unit(cumul, "mm")))
        case let .accalmie(arret, .none):
            phrases.append(Localized.text("veille.next.lull", heure(arret), fin))
        case let .accalmie(arret, reprise?):
            phrases.append(Localized.text("veille.next.lullReturn", heure(arret), heure(reprise)))
        }

        if let rafales = lecture.rafales {
            phrases.append(Localized.text(
                "veille.gusts", AgroFormat.unit(rafales.valeur, "km/h", decimals: 0), heure(rafales.quand)
            ))
        }
        if let dernier = lecture.quarts.last {
            phrases.append(Localized.text(
                "veille.temperature",
                AgroFormat.temperature(dernier.temperature),
                heure(dernier.time),
                AgroFormat.temperature(dernier.apparentTemperature)
            ))
        }
        return phrases.joined(separator: " ")
    }
}
