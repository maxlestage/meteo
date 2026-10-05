import SwiftUI

/// Le ciel vivant : derrière l'application, le temps qu'il fait dans la ville
/// affichée.
///
/// Miroir de `rust/klima-web/src/composants/ciel.rs`. Le jour, un soleil qui
/// respire ; la nuit, des étoiles et la lune ; des nuages d'autant plus
/// nombreux et sombres que le ciel est couvert ; et ce qui tombe — bruine,
/// pluie, averses, neige —, les éclairs d'un orage, les bancs d'un
/// brouillard. Le tout d'après le code météo du moment : le fond dit la même
/// chose que l'en-tête.
///
/// Un seul `Canvas`, redessiné à trente images par seconde au plus : les
/// positions se calculent depuis l'heure, sans état à tenir. Les couleurs
/// suivent le thème du système — des gouttes bleu ardoise sur un ciel clair,
/// argentées sur un ciel sombre. Pour qui demande moins de mouvement, le ciel
/// est dessiné une fois et ne bouge plus.
struct CielVivant: View {
    let isDay: Bool
    let weatherCode: Int

    @Environment(\.colorScheme) private var scheme
    @Environment(\.accessibilityReduceMotion) private var moinsDeMouvement

    var body: some View {
        TimelineView(.animation(minimumInterval: 1.0 / 30.0, paused: moinsDeMouvement)) { contexte in
            Canvas { g, taille in
                let tableau = Tableau(icone: WeatherCondition.forCode(weatherCode).icon, jour: isDay, sombre: scheme == .dark)
                tableau.dessiner(&g, taille, t: contexte.date.timeIntervalSinceReferenceDate)
            }
        }
        .ignoresSafeArea()
        .allowsHitTesting(false)
        .accessibilityHidden(true)
    }
}

/// Une particule placée une fois pour toutes : position, rythme, taille.
private struct Particule {
    let x: Double
    let y: Double
    let duree: Double
    let phase: Double
    let echelle: Double
}

/// Une suite pseudo-aléatoire fixe : le même ciel d'un rendu à l'autre, les
/// mêmes graines que côté web.
private struct Graine {
    var etat: UInt32

    mutating func suivant() -> Double {
        etat = etat &* 1_664_525 &+ 1_013_904_223
        return Double(etat >> 8) / Double(1 << 24)
    }

    static func particules(_ nombre: Int, graine: UInt32, dureeMin: Double, dureeEcart: Double) -> [Particule] {
        var g = Graine(etat: graine)
        return (0..<nombre).map { _ in
            let x = g.suivant()
            let y = g.suivant()
            let duree = dureeMin + g.suivant() * dureeEcart
            let phase = g.suivant()
            let echelle = 0.6 + g.suivant() * 0.6
            return Particule(x: x, y: y, duree: duree, phase: phase, echelle: echelle)
        }
    }
}

private enum Reserve {
    static let etoiles = Graine.particules(48, graine: 7_731, dureeMin: 2.6, dureeEcart: 2.4)
    static let gouttes = Graine.particules(100, graine: 42_424, dureeMin: 0.55, dureeEcart: 0.35)
    static let flocons = Graine.particules(60, graine: 42_424, dureeMin: 7, dureeEcart: 6)

    /// (haut, largeur relative, durée de traversée, avance)
    static let nuages: [(Double, Double, Double, Double)] = [
        (0.06, 0.58, 90, 0.2), (0.22, 0.44, 70, 0.7), (0.40, 0.64, 120, 0.8),
        (0.58, 0.40, 80, 0.1), (0.74, 0.54, 105, 0.55),
    ]
}

private enum Chute { case rien, bruine, pluie, averse, neige }

private struct Tableau {
    let icone: WeatherCondition.Icon
    let jour: Bool
    let sombre: Bool

    private var chute: Chute {
        switch icone {
        case .drizzle: return .bruine
        case .rain: return .pluie
        case .showers, .thunder: return .averse
        case .snow: return .neige
        default: return .rien
        }
    }

    private var nombreDeNuages: Int {
        switch icone {
        case .clear: return 2
        case .partly: return 3
        default: return 5
        }
    }

    private var nuagesSombres: Bool { icone == .rain || icone == .showers || icone == .thunder }
    private var soleil: Bool { jour && (icone == .clear || icone == .partly || icone == .showers) }
    private var etoiles: Bool { !jour && (icone == .clear || icone == .partly || icone == .cloudy) }

    func dessiner(_ g: inout GraphicsContext, _ taille: CGSize, t: Double) {
        if soleil { dessinerSoleil(&g, taille, t) }
        if etoiles { dessinerEtoiles(&g, taille, t) }
        dessinerNuages(&g, taille, t)
        if icone == .fog { dessinerBrouillard(&g, taille, t) }
        switch chute {
        case .rien: break
        case .neige: dessinerNeige(&g, taille, t)
        default: dessinerPluie(&g, taille, t)
        }
        if icone == .thunder { dessinerEclair(&g, taille, t) }
    }

    // MARK: Soleil, étoiles, lune

    private func dessinerSoleil(_ g: inout GraphicsContext, _ taille: CGSize, _ t: Double) {
        let respire = 1 + 0.07 * sin(t * 2 * .pi / 8)
        let rayon = min(taille.width, taille.height) * 0.62 * respire
        let centre = CGPoint(x: taille.width * 0.92, y: taille.height * 0.04)
        let teinte = sombre ? Color(red: 1, green: 0.886, blue: 0.549) : Color(red: 1, green: 0.78, blue: 0.35)
        let degrade = Gradient(stops: [
            .init(color: teinte.opacity(sombre ? 0.55 : 0.6), location: 0),
            .init(color: teinte.opacity(sombre ? 0.22 : 0.25), location: 0.28),
            .init(color: teinte.opacity(0.06), location: 0.55),
            .init(color: teinte.opacity(0), location: 1),
        ])
        let disque = Path(ellipseIn: CGRect(x: centre.x - rayon, y: centre.y - rayon, width: rayon * 2, height: rayon * 2))
        g.fill(disque, with: .radialGradient(degrade, center: centre, startRadius: 0, endRadius: rayon))
    }

    private func dessinerEtoiles(_ g: inout GraphicsContext, _ taille: CGSize, _ t: Double) {
        let voile = icone == .cloudy ? 0.4 : 1
        let couleur = sombre ? Color.white : Color(red: 0.30, green: 0.33, blue: 0.58)
        for (i, e) in Reserve.etoiles.enumerated() {
            let r = 0.5 + e.echelle * 0.8
            let scintille = i % 3 == 0 ? 0.55 + 0.45 * sin(t * 2 * .pi / e.duree + e.phase * 6.28) : 1
            let point = CGRect(x: e.x * taille.width - r, y: e.y * 0.62 * taille.height - r, width: r * 2, height: r * 2)
            g.fill(Path(ellipseIn: point), with: .color(couleur.opacity(0.75 * voile * scintille)))
        }

        // La lune, à hauteur du nom de la ville, qui se balance doucement.
        var lune = g.resolve(Image(systemName: "moon.fill"))
        lune.shading = .color(sombre ? Color(red: 0.91, green: 0.93, blue: 0.95) : Color(red: 0.36, green: 0.40, blue: 0.62))
        var calque = g
        let centre = CGPoint(x: taille.width * 0.86, y: taille.height * 0.22 + 4 * sin(t * 2 * .pi / 12))
        calque.translateBy(x: centre.x, y: centre.y)
        calque.rotate(by: .degrees(-6 + 5 * sin(t * 2 * .pi / 12)))
        calque.addFilter(.shadow(color: (sombre ? Color.white : Color.indigo).opacity(0.45), radius: 12))
        calque.draw(lune, in: CGRect(x: -17, y: -17, width: 34, height: 34))
    }

    // MARK: Nuages et brouillard

    private func dessinerNuages(_ g: inout GraphicsContext, _ taille: CGSize, _ t: Double) {
        let couleur: Color
        let opacite: Double
        switch (nuagesSombres, sombre) {
        case (true, true): couleur = Color(red: 0.36, green: 0.41, blue: 0.48); opacite = 0.40
        case (true, false): couleur = Color(red: 0.55, green: 0.60, blue: 0.68); opacite = 0.40
        case (false, true): couleur = .white; opacite = 0.17
        case (false, false): couleur = .white; opacite = 0.75
        }
        let degrade = Gradient(stops: [
            .init(color: couleur.opacity(opacite), location: 0),
            .init(color: couleur.opacity(opacite * 0.55), location: 0.6),
            .init(color: couleur.opacity(0), location: 1),
        ])

        for (haut, largeurRelative, duree, avance) in Reserve.nuages.prefix(nombreDeNuages) {
            let largeur = max(taille.width * largeurRelative * 1.6, 300)
            let progres = (t / duree + avance).truncatingRemainder(dividingBy: 1)
            let x = progres * (taille.width + largeur) - largeur
            let y = haut * taille.height
            let k = largeur / 200
            // Trois ellipses aux bords fondus : la forme du dessin web.
            let ellipses: [(Double, Double, Double, Double)] = [(100, 50, 92, 26), (74, 36, 44, 28), (124, 30, 52, 30)]
            for (cx, cy, rx, ry) in ellipses {
                let rect = CGRect(x: x + (cx - rx) * k, y: y + (cy - ry) * k, width: rx * 2 * k, height: ry * 2 * k)
                let centre = CGPoint(x: rect.midX, y: rect.midY)
                g.fill(
                    Path(ellipseIn: rect),
                    with: .radialGradient(degrade, center: centre, startRadius: 0, endRadius: max(rect.width, rect.height) / 2)
                )
            }
        }
    }

    private func dessinerBrouillard(_ g: inout GraphicsContext, _ taille: CGSize, _ t: Double) {
        let couleur = Color.white.opacity(sombre ? 0.16 : 0.55)
        let degrade = Gradient(colors: [couleur, couleur.opacity(0)])
        let bancs: [(Double, Double, Double)] = [(0.30, 38, 0), (0.52, 52, 0.4), (0.72, 44, 0.2)]
        for (haut, duree, phase) in bancs {
            let decalage = sin((t / duree + phase) * 2 * .pi) * taille.width * 0.25
            let rect = CGRect(x: -taille.width * 0.5 + decalage, y: haut * taille.height, width: taille.width * 2, height: taille.height * 0.26)
            g.fill(
                Path(ellipseIn: rect),
                with: .radialGradient(degrade, center: CGPoint(x: rect.midX, y: rect.midY), startRadius: 0, endRadius: rect.width / 2.6)
            )
        }
    }

    // MARK: Ce qui tombe

    private func dessinerPluie(_ g: inout GraphicsContext, _ taille: CGSize, _ t: Double) {
        let reglage: (nombre: Int, longueur: Double, epaisseur: Double, ralenti: Double)
        switch chute {
        case .bruine: reglage = (36, 10, 1.2, 2.0)
        case .averse: reglage = (100, 28, 2.0, 1.0)
        default: reglage = (70, 20, 1.5, 1.4)
        }
        let (nombre, longueur, epaisseur, ralenti) = reglage
        let couleur = sombre ? Color(red: 0.78, green: 0.90, blue: 1) : Color(red: 0.22, green: 0.38, blue: 0.58)
        let opacite = sombre ? 0.6 : 0.42

        for goutte in Reserve.gouttes.prefix(nombre) {
            let duree = goutte.duree * ralenti
            let progres = (t / duree + goutte.phase).truncatingRemainder(dividingBy: 1)
            let y = progres * taille.height * 1.25 - taille.height * 0.12
            let x = (goutte.x * 1.04 - 0.02) * taille.width - progres * taille.height * 0.03
            let l = longueur * goutte.echelle
            var trait = Path()
            trait.move(to: CGPoint(x: x, y: y))
            trait.addLine(to: CGPoint(x: x - l * 0.08, y: y + l))
            g.stroke(
                trait,
                with: .linearGradient(
                    Gradient(colors: [couleur.opacity(0), couleur.opacity(opacite)]),
                    startPoint: CGPoint(x: x, y: y), endPoint: CGPoint(x: x, y: y + l)
                ),
                style: StrokeStyle(lineWidth: epaisseur, lineCap: .round)
            )
        }
    }

    private func dessinerNeige(_ g: inout GraphicsContext, _ taille: CGSize, _ t: Double) {
        for flocon in Reserve.flocons {
            let progres = (t / flocon.duree + flocon.phase).truncatingRemainder(dividingBy: 1)
            let y = progres * taille.height * 1.14 - taille.height * 0.04
            let x = (flocon.x * 1.04 - 0.02) * taille.width + sin(progres * 4 * .pi) * 13
            let r = 3 * flocon.echelle
            let rond = Path(ellipseIn: CGRect(x: x - r, y: y - r, width: r * 2, height: r * 2))
            g.fill(rond, with: .color(.white.opacity(0.88)))
            if !sombre {
                // Sur un ciel clair, un fin liseré garde le flocon visible.
                g.stroke(rond, with: .color(Color(red: 0.45, green: 0.52, blue: 0.65).opacity(0.35)), lineWidth: 0.8)
            }
        }
    }

    /// Un éclair toutes les neuf secondes : deux éclats rapprochés.
    private func dessinerEclair(_ g: inout GraphicsContext, _ taille: CGSize, _ t: Double) {
        let phase = (t / 9).truncatingRemainder(dividingBy: 1)
        let intensite: Double
        switch phase {
        case 0.91..<0.92: intensite = 0.45
        case 0.92..<0.935: intensite = 0.05
        case 0.935..<0.95: intensite = 0.3
        default: intensite = 0
        }
        guard intensite > 0 else { return }
        g.fill(Path(CGRect(origin: .zero, size: taille)), with: .color(Color(red: 0.92, green: 0.94, blue: 1).opacity(intensite)))
    }
}
