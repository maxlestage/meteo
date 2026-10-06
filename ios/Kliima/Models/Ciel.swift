import Foundation

/// Le ciel observé : ce qui tombe **vraiment**, d'après l'aéroport le plus
/// proche.
///
/// Miroir de `rust/klima-core/src/ciel.rs`, avec les mêmes cas de test. Toute
/// règle ajoutée d'un côté se porte de l'autre.
///
/// Toutes les autres sources sont des prévisions : une averse qu'aucun modèle
/// n'a vue venir n'existe pour aucun. Les aéroports, eux, regardent : leur
/// bulletin (METAR) dit le temps présent — `-RA`, `+SHRA`, `TS`. L'Aviation
/// Weather Center de la NOAA les publie tous, dans le domaine public.
///
/// La règle est à sens unique : une pluie observée s'ajoute à la prévision, un
/// ciel sec observé n'en retire rien.
enum CielSeuils {
    /// Au-delà, l'aéroport parle d'un autre ciel (km).
    static let rayonKm = 30.0
    /// Au-delà, le bulletin parle d'un autre moment (s).
    static let ageMax: TimeInterval = 75 * 60
    /// Un bulletin daté d'un peu après « maintenant » (s).
    static let avanceMax: TimeInterval = 10 * 60
    /// Ce qu'un quart observé mouillé compte, selon l'intensité (mm).
    static let quartFaibleMm = 0.2
    static let quartModereMm = 1.0
    static let quartFortMm = 2.5
    /// Ce qu'un bulletin dit voir vaut jusqu'au suivant : une demi-heure.
    static let validite: TimeInterval = 30 * 60
    /// La tendance d'un METAR (`NOSIG`, `TEMPO`, `BECMG`) vaut deux heures.
    static let tendance: TimeInterval = 2 * 3600
}

/// Un bulletin d'aéroport, déjà lu.
struct Metar: Equatable, Sendable {
    /// Indicatif OACI, `LFPG`.
    let station: String
    /// Nom lisible, débarrassé de son suffixe.
    let nom: String
    let latitude: Double
    let longitude: Double
    /// Heure de l'observation.
    let time: Date
    /// Le temps présent, brut : `-RA BR`, `+TSRA`… Vide si rien à signaler.
    let tempsPresent: String
    /// La tendance des prévisionnistes pour les deux heures qui viennent.
    var tendance: CielTendance = .inconnue
}

/// Ce qui tombe, vu ou annoncé : un code de l'OMM et sa force telle que le
/// bulletin la donne (`-` faible, rien modérée, `+` forte). `Tombe` côté Rust.
struct CielTombe: Equatable, Comparable, Sendable {
    let code: Int
    let intensite: VeilleIntensite

    /// Nature d'après le code, force d'après le bulletin.
    var precipitation: VeillePrecipitation {
        VeillePrecipitation(nature: Ciel.precipitationDuCode(code).nature, intensite: intensite)
    }

    static func < (a: CielTombe, b: CielTombe) -> Bool {
        a.code != b.code ? a.code < b.code : a.intensite < b.intensite
    }
}

/// Ce que les prévisionnistes de l'aéroport annoncent pour deux heures.
enum CielTendance: Equatable, Sendable {
    /// Le bulletin n'en dit rien.
    case inconnue
    /// `NOSIG` : rien de notable ne changera.
    case stable
    /// `TEMPO` (passager) ou `BECMG` ; `sec` : le groupe dit `NSW`.
    case changement(passager: Bool, tombe: CielTombe?, sec: Bool)
}

/// Ce que l'aéroport annonce de tomber d'ici la fin de sa tendance.
struct CielAnnonce: Equatable, Sendable {
    let passagere: Bool
    let precipitation: VeillePrecipitation
    let jusquA: Date
}

/// Le ciel observé près de la ville.
struct CielObserve: Equatable, Sendable {
    let station: String
    let nom: String
    /// Distance à la ville, arrondie au kilomètre.
    let distanceKm: Double
    let time: Date
    /// Ce qui tombe ; `nil` si rien ne tombe.
    let tombe: CielTombe?
    var tendance: CielTendance = .inconnue

    /// Ce qui tombe, pour le dire.
    var precipitation: VeillePrecipitation? { tombe?.precipitation }

    /// Jusqu'à quand ce qu'on voit tombera encore, d'après le bulletin :
    /// deux heures sous `NOSIG` ou un `BECMG` qui fait tomber, le quart en
    /// cours seulement sous `BECMG NSW`, sinon jusqu'au bulletin suivant.
    var tombeJusquA: Date? {
        guard tombe != nil else { return nil }
        switch tendance {
        case .stable:
            return time.addingTimeInterval(CielSeuils.tendance)
        case .changement(passager: false, tombe: _, sec: true):
            return time
        case .changement(passager: false, tombe: .some(_), sec: _):
            return time.addingTimeInterval(CielSeuils.tendance)
        default:
            return time.addingTimeInterval(CielSeuils.validite)
        }
    }

    /// Ce que l'aéroport annonce de tomber d'ici deux heures, s'il l'annonce.
    var annonce: CielAnnonce? {
        guard case let .changement(passager, tombe?, _) = tendance else { return nil }
        return CielAnnonce(
            passagere: passager, precipitation: tombe.precipitation,
            jusquA: time.addingTimeInterval(CielSeuils.tendance)
        )
    }
}

enum Ciel {

    /// Distance à vol d'oiseau (km), sur une Terre ronde.
    static func distanceKm(_ lat1: Double, _ lon1: Double, _ lat2: Double, _ lon2: Double) -> Double {
        let rayonTerre: Double = 6371
        let parDegre: Double = Double.pi / 180
        let p1: Double = lat1 * parDegre
        let p2: Double = lat2 * parDegre
        let sinLat: Double = sin((lat2 - lat1) * parDegre / 2)
        let sinLon: Double = sin((lon2 - lon1) * parDegre / 2)
        let a: Double = sinLat * sinLat + cos(p1) * cos(p2) * sinLon * sinLon
        return 2 * rayonTerre * asin(a.squareRoot())
    }

    /// Le cadre où chercher les aéroports : sud, ouest, nord, est, arrondis au
    /// centième, un peu plus large que le rayon.
    static func cadre(_ latitude: Double, _ longitude: Double) -> (Double, Double, Double, Double) {
        let margeKm: Double = CielSeuils.rayonKm + 10
        let dlat: Double = margeKm / 111.32
        let cosLat: Double = max(cos(latitude * Double.pi / 180), 0.05)
        let dlon: Double = min(margeKm / (111.32 * cosLat), 180)
        let c: (Double) -> Double = { x in (x * 100).rounded() / 100 }
        return (
            c(max(latitude - dlat, -90)),
            c(max(longitude - dlon, -180)),
            c(min(latitude + dlat, 90)),
            c(min(longitude + dlon, 180))
        )
    }

    /// Le code météo de l'OMM d'un temps présent METAR, `nil` si rien ne tombe.
    /// Chaque groupe se lit à part ; on garde le plus marqué.
    static func codeDuMetar(_ tempsPresent: String) -> Int? {
        tombeDuMetar(tempsPresent)?.code
    }

    /// Ce qui tombe d'après un temps présent METAR, avec sa force.
    static func tombeDuMetar(_ tempsPresent: String) -> CielTombe? {
        tempsPresent.split(separator: " ").compactMap { tombeDuGroupe(String($0)) }.max()
    }

    private static func tombeDuGroupe(_ groupe: String) -> CielTombe? {
        let force: Int
        let reste: Substring
        switch groupe.first {
        case "-": force = 0; reste = groupe.dropFirst()
        case "+": force = 2; reste = groupe.dropFirst()
        default: force = 1; reste = Substring(groupe)
        }
        guard let code = codeDuGroupe(reste, force: force) else { return nil }
        let intensites: [VeilleIntensite] = [.faible, .moderee, .forte]
        return CielTombe(code: code, intensite: intensites[force])
    }

    /// La tendance d'un bulletin brut : `NOSIG`, ou le premier groupe `TEMPO`
    /// ou `BECMG` et ce qu'il fait tomber. Les remarques (`RMK`) ne comptent
    /// pas.
    static func tendanceDuMetar(_ brut: String) -> CielTendance {
        var groupes: [String] = []
        for morceau in brut.split(separator: " ") {
            if morceau == "RMK" { break }
            groupes.append(String(morceau))
        }
        if groupes.contains("NOSIG") { return .stable }
        guard let debut = groupes.firstIndex(where: { $0 == "TEMPO" || $0 == "BECMG" }) else { return .inconnue }
        let suite: [String] = Array(groupes[(debut + 1)...].prefix { $0 != "TEMPO" && $0 != "BECMG" })
        let tombe: CielTombe? = suite.compactMap { tombeDuGroupe($0) }.max()
        return .changement(passager: groupes[debut] == "TEMPO", tombe: tombe, sec: suite.contains("NSW"))
    }

    private static func codeDuGroupe(_ reste: Substring, force: Int) -> Int? {
        if reste.hasPrefix("VC") || reste.hasPrefix("RE") { return nil }
        // Chasse-neige, sable soulevé : ça vole, ça ne tombe pas.
        if reste.hasPrefix("BL") || reste.hasPrefix("DR") { return nil }
        let a: (String) -> Bool = { motif in reste.contains(motif) }
        let selon: ([Int]) -> Int = { codes in codes[force] }

        if a("TS") {
            if a("GR") || a("GS") { return force == 2 ? 99 : 96 }
            return 95
        }
        if a("SH") {
            if a("SN") { return force == 2 ? 86 : 85 }
            if a("RA") || a("GR") || a("GS") || a("UP") { return selon([80, 81, 82]) }
        }
        if a("FZ") {
            if a("RA") || a("UP") { return force == 2 ? 67 : 66 }
            if a("DZ") { return force == 2 ? 57 : 56 }
        }
        if a("SN") { return selon([71, 73, 75]) }
        if a("SG") || a("PL") || a("GR") || a("GS") || a("IC") { return 77 }
        if a("RA") || a("UP") { return selon([61, 63, 65]) }
        if a("DZ") { return selon([51, 53, 55]) }
        return nil
    }

    /// Nature et intensité d'un code météo mouillé.
    static func precipitationDuCode(_ code: Int) -> VeillePrecipitation {
        let nature: VeilleNature
        switch code {
        case 95, 96, 99: nature = .orage
        case 71...77, 85, 86: nature = .neige
        default: nature = .pluie
        }
        let intensite: VeilleIntensite
        switch code {
        case 53, 63, 73, 81: intensite = .moderee
        case 55, 57, 65, 67, 75, 82, 86, 99: intensite = .forte
        default: intensite = .faible
        }
        return VeillePrecipitation(nature: nature, intensite: intensite)
    }

    /// Ce qu'un quart observé mouillé compte (mm), selon la force observée.
    static func quartObserveMm(_ intensite: VeilleIntensite) -> Double {
        switch intensite {
        case .faible: return CielSeuils.quartFaibleMm
        case .moderee: return CielSeuils.quartModereMm
        case .forte: return CielSeuils.quartFortMm
        }
    }

    /// Le bulletin qui parle de la ville : le plus proche dans le rayon, parmi
    /// ceux qui parlent de maintenant.
    static func plusProche(_ metars: [Metar], latitude: Double, longitude: Double, maintenant: Date) -> CielObserve? {
        var retenu: Metar?
        var distanceRetenue = Double.infinity
        for metar in metars {
            let age = maintenant.timeIntervalSince(metar.time)
            guard age <= CielSeuils.ageMax, -age <= CielSeuils.avanceMax else { continue }
            let distance: Double = distanceKm(latitude, longitude, metar.latitude, metar.longitude)
            guard distance <= CielSeuils.rayonKm else { continue }
            // À distance égale, le plus récent.
            let plusPres = distance < distanceRetenue
            let aussiPresPlusRecent = distance == distanceRetenue && metar.time > (retenu?.time ?? .distantPast)
            if plusPres || aussiPresPlusRecent {
                retenu = metar
                distanceRetenue = distance
            }
        }
        guard let retenu else { return nil }
        return CielObserve(
            station: retenu.station, nom: retenu.nom, distanceKm: distanceRetenue.rounded(),
            time: retenu.time, tombe: tombeDuMetar(retenu.tempsPresent), tendance: retenu.tendance
        )
    }

    /// Le nom lisible d'un aéroport : `Paris/Le Bourge Arpt, ID, FR` →
    /// `Paris/Le Bourge`.
    static func nomLisible(_ brut: String) -> String {
        var nom = (brut.split(separator: ",", omittingEmptySubsequences: false).first.map(String.init) ?? "")
            .trimmingCharacters(in: .whitespaces)
        for suffixe in [" Intl Arpt", " Arpt", " Airport", " Aprt", " Intl", " AB", " Afb"] where nom.hasSuffix(suffixe) {
            nom = String(nom.dropLast(suffixe.count)).trimmingCharacters(in: .whitespaces)
            break
        }
        return nom
    }

    /// Lit la réponse de l'Aviation Weather Center : un tableau, un bulletin
    /// par aéroport. Sans position ou sans heure, un bulletin est écarté.
    static func decodeMetars(_ data: Data) -> [Metar] {
        guard let bulletins = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else { return [] }
        return bulletins.compactMap { b in
            guard
                let station = b["icaoId"] as? String,
                let latitude = (b["lat"] as? NSNumber)?.doubleValue,
                let longitude = (b["lon"] as? NSNumber)?.doubleValue,
                let heure = (b["obsTime"] as? NSNumber)?.doubleValue
            else { return nil }
            return Metar(
                station: station,
                nom: nomLisible(b["name"] as? String ?? ""),
                latitude: latitude,
                longitude: longitude,
                time: Date(timeIntervalSince1970: heure),
                tempsPresent: b["wxString"] as? String ?? "",
                tendance: tendanceDuMetar(b["rawOb"] as? String ?? "")
            )
        }
    }
}
