import Foundation

/// Le radar : ce qui tombe au-dessus de la ville, et ce qui vient.
///
/// La mosaïque européenne OPERA d'EUMETNET (licence CC BY 4.0) est lue par le
/// relais (`rust/klima-relay/src/radar.rs`), qui en tire, pour la ville, le
/// débit du moment, le déplacement des averses et les deux heures qui
/// viennent (`rust/klima-core/src/radar.rs`). L'iPhone n'en reçoit que ce
/// résumé, sur `/v1/radar` et par le direct ; ce fichier en est la lecture,
/// miroir de `rust/klima-api/src/radar.rs`, avec les mêmes cas de test.
enum RadarSeuils {
    /// À partir de ce débit au-dessus de la ville, il tombe quelque chose
    /// (mm/h) : le seuil d'un quart mouillé du guetteur, ramené à l'heure.
    static let debitMouille = VeilleSeuils.pluieQuartMm * 4
    /// Au-delà de cet âge, l'image ne dit plus ce qui tombe. Le guetteur relit
    /// à chaque quart : une lecture manquée laisse une image d'une vingtaine
    /// de minutes, deux disent que le relais ne répond plus.
    static let ageMax: TimeInterval = 30 * 60
}

/// Ce que le radar dit pour une ville.
struct RadarPrevision: Equatable, Sendable {
    /// L'heure de l'image.
    let image: Date
    /// Le débit au-dessus de la ville (mm/h) ; `nil` si aucun radar ne la voit.
    let maintenant: Double?
    /// Les quarts d'heure qui viennent : début et débit prévu (mm/h).
    let quarts: [(debut: Date, debit: Double)]
    /// Vitesse (km/h) et direction où va la pluie (degrés, 0 = nord).
    let deplacement: (vitesse: Double, cap: Double)?

    static func == (a: RadarPrevision, b: RadarPrevision) -> Bool {
        a.image == b.image && a.maintenant == b.maintenant
            && a.quarts.map(\.debut) == b.quarts.map(\.debut)
            && a.quarts.map(\.debit) == b.quarts.map(\.debit)
            && a.deplacement?.vitesse == b.deplacement?.vitesse
            && a.deplacement?.cap == b.deplacement?.cap
    }

    /// Vrai tant que l'image n'est pas plus vieille que `RadarSeuils.ageMax`
    /// à `instant`. Gardée en mémoire quand le relais ne répond plus, une
    /// image de midi disait encore à 15 h ce qui tombait, taisait l'aéroport
    /// et montrait où allait une averse partie depuis. Miroir de
    /// `Prevision::frais`.
    func frais(_ instant: Date) -> Bool {
        instant.timeIntervalSince(image) <= RadarSeuils.ageMax
    }

    /// Vrai quand un radar voit la ville sur une image fraîche : il fait alors
    /// foi pour ce qui tombe. Miroir de `Prevision::voit_la_ville`.
    func voitLaVille(a instant: Date) -> Bool {
        maintenant != nil && frais(instant)
    }
}

enum Radar {

    /// Lit la réponse du relais ; `nil` si ce n'en est pas une.
    static func decode(_ data: Data) -> RadarPrevision? {
        guard
            let objet = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
            let image = (objet["image"] as? NSNumber)?.doubleValue,
            let brut = objet["quarts"] as? [[NSNumber]]
        else { return nil }
        let quarts: [(debut: Date, debit: Double)] = brut.compactMap { paire -> (debut: Date, debit: Double)? in
            guard paire.count == 2 else { return nil }
            return (debut: Date(timeIntervalSince1970: paire[0].doubleValue), debit: paire[1].doubleValue)
        }
        var deplacement: (vitesse: Double, cap: Double)?
        if let vitesse = (objet["vitesse"] as? NSNumber)?.doubleValue,
           let cap = (objet["cap"] as? NSNumber)?.doubleValue {
            deplacement = (vitesse: vitesse, cap: cap)
        }
        return RadarPrevision(
            image: Date(timeIntervalSince1970: image),
            maintenant: (objet["maintenant"] as? NSNumber)?.doubleValue,
            quarts: quarts,
            deplacement: deplacement
        )
    }

    /// Le code météo d'un débit (mm/h) : pluie faible, modérée ou forte.
    static func codeDuDebit(_ mmH: Double) -> Int {
        if mmH >= VeilleSeuils.forteMmH { return 65 }
        if mmH >= VeilleSeuils.modereeMmH { return 63 }
        return 61
    }

    /// Ce que le radar voit tomber sur la ville, en code de l'OMM.
    static func tombe(_ maintenant: Double?) -> Int? {
        guard let maintenant, maintenant >= RadarSeuils.debitMouille else { return nil }
        return codeDuDebit(maintenant)
    }

    /// La direction où va la pluie, en clé de catalogue : `dir.n`, `dir.ne`…
    static func direction(_ cap: Double) -> String {
        let rose = ["dir.n", "dir.ne", "dir.e", "dir.se", "dir.s", "dir.so", "dir.o", "dir.no"]
        let degres = cap.truncatingRemainder(dividingBy: 360)
        let positif = degres < 0 ? degres + 360 : degres
        return rose[Int((positif + 22.5) / 45) % 8]
    }
}
