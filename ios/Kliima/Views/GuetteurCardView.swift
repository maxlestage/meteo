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

    private func immediat(_ lecture: VeilleLecture) -> String {
        VeilleTextes.immediat(lecture, in: timeZone)
    }

    private func suite(_ lecture: VeilleLecture) -> String {
        VeilleTextes.suite(lecture, in: timeZone).joined(separator: " ")
    }
}
