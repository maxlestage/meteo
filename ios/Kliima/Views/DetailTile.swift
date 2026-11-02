import SwiftUI

/// Tuile de détail, dans l'esprit des cartes « Vent » ou « Indice UV ».
struct DetailTile: View {
    let label: String
    let value: String
    var caption: String?
    /// Jauge optionnelle : position 0–1 sur une échelle colorée.
    var gauge: (position: Double, colors: [Color])?

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            CardLabel(text: label)

            Text(value)
                .font(.system(size: 27, weight: .regular))
                .lineLimit(2)
                .minimumScaleFactor(0.7)

            if let gauge {
                gaugeBar(gauge)
            }

            if let caption {
                Spacer(minLength: 6)
                Text(caption)
                    .font(.footnote)
                    .foregroundStyle(Color.encreDouce)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .frame(maxWidth: .infinity, minHeight: 132, alignment: .topLeading)
        .cardBackground()
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(label) : \(value). \(caption ?? "")")
    }

    private func gaugeBar(_ gauge: (position: Double, colors: [Color])) -> some View {
        GeometryReader { geometry in
            // Le point voyage à l'intérieur de la piste : centré sur la
            // position, il débordait de sa moitié à chaque bout — toutes
            // les nuits sous un indice UV nul, il sortait à gauche du rail.
            let course = max(geometry.size.width - 9, 0)
            let position = min(max(gauge.position, 0), 1)

            ZStack(alignment: .leading) {
                Capsule()
                    .fill(LinearGradient(colors: gauge.colors, startPoint: .leading, endPoint: .trailing))
                    .frame(height: 5)
                Circle()
                    .fill(Color.encre)
                    .frame(width: 9, height: 9)
                    .offset(x: course * position)
            }
            .frame(maxHeight: .infinity)
        }
        .frame(height: 12)
        .padding(.top, 2)
    }
}
