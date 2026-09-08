import SwiftUI

/// Couleur d'un indicateur selon qu'il est favorable, à surveiller ou bloquant.
enum Tone {
    case good, warn, bad, neutral

    var color: Color {
        switch self {
        case .good: return Color("Favorable")
        case .warn: return Color("Vigilance")
        case .bad: return Color("Alerte")
        case .neutral: return Color.accentColor
        }
    }
}

/// Carte d'indicateur du tableau de bord : un titre, une valeur, un commentaire
/// et d'éventuelles étiquettes de conduite de culture.
struct IndicatorCard: View {
    let title: String
    let value: String
    let caption: String
    var tone: Tone = .neutral
    var tags: [String] = []

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title.uppercased())
                .font(.caption2.weight(.semibold))
                .kerning(0.6)
                .foregroundStyle(.secondary)

            Text(value)
                .font(.title2.weight(.semibold))
                .foregroundStyle(tone.color)
                .minimumScaleFactor(0.7)
                .lineLimit(2)

            Text(caption)
                .font(.footnote)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)

            if !tags.isEmpty {
                HStack(spacing: 6) {
                    ForEach(tags, id: \.self) { tag in
                        Text(tag)
                            .font(.caption2)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 3)
                            .background(Color.secondary.opacity(0.12), in: Capsule())
                    }
                }
                .padding(.top, 2)
            }
        }
        .frame(maxWidth: .infinity, minHeight: 120, alignment: .topLeading)
        .padding(14)
        .background(Color(uiColor: .secondarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 14))
        .overlay(alignment: .leading) {
            Rectangle()
                .fill(tone.color)
                .frame(width: 4)
                .clipShape(RoundedRectangle(cornerRadius: 2))
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(title) : \(value). \(caption)")
    }
}

#Preview {
    IndicatorCard(
        title: "Bilan hydrique 7 j",
        value: "-19.9 mm",
        caption: "Irrigation conseillée : 20 mm",
        tone: .bad,
        tags: ["Portance correcte"]
    )
    .padding()
}
