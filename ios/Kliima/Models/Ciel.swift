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
}

/// Le ciel observé près de la ville.
struct CielObserve: Equatable, Sendable {
    let station: String
    let nom: String
    /// Distance à la ville, arrondie au kilomètre.
    let distanceKm: Double
    let time: Date
    /// Ce qui tombe, en code météo de l'OMM ; `nil` si rien ne tombe.
    let tombe: Int?

    /// Ce qui tombe, pour le dire.
    var precipitation: VeillePrecipitation? { tombe.map(Ciel.precipitationDuCode) }
}

enum Ciel {

    /// Distance à vol d'oiseau (km), sur une Terre ronde.
    static func distanceKm(_ lat1: Double, _ lon1: Double, _ lat2: Double, _ lon2: Double) -> Double {
        let rayonTerre = 6371.0
        let radians = { (degres: Double) in degres * .pi / 180 }
        let (p1, p2) = (radians(lat1), radians(lat2))
        let dp = radians(lat2 - lat1)
        let dl = radians(lon2 - lon1)
        let a = pow(sin(dp / 2), 2) + cos(p1) * cos(p2) * pow(sin(dl / 2), 2)
        return 2 * rayonTerre * asin(a.squareRoot())
    }

    /// Le cadre où chercher les aéroports : sud, ouest, nord, est, arrondis au
    /// centième, un peu plus large que le rayon.
    static func cadre(_ latitude: Double, _ longitude: Double) -> (Double, Double, Double, Double) {
        let margeKm = CielSeuils.rayonKm + 10
        let dlat = margeKm / 111.32
        let dlon = min(margeKm / (111.32 * max(cos(latitude * .pi / 180), 0.05)), 180)
        let c = { (x: Double) in (x * 100).rounded() / 100 }
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
        tempsPresent.split(separator: " ").compactMap { codeDuGroupe(String($0)) }.max()
    }

    private static func codeDuGroupe(_ groupe: String) -> Int? {
        if groupe.hasPrefix("VC") || groupe.hasPrefix("RE") { return nil }
        let force: Int
        let reste: Substring
        switch groupe.first {
        case "-": force = 0; reste = groupe.dropFirst()
        case "+": force = 2; reste = groupe.dropFirst()
        default: force = 1; reste = Substring(groupe)
        }
        // Chasse-neige, sable soulevé : ça vole, ça ne tombe pas.
        if reste.hasPrefix("BL") || reste.hasPrefix("DR") { return nil }
        let a = { (motif: String) in reste.contains(motif) }
        let selon = { (codes: [Int]) in codes[force] }

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

    /// Ce qu'un quart observé mouillé compte (mm), selon le code.
    static func quartObserveMm(_ code: Int) -> Double {
        switch precipitationDuCode(code).intensite {
        case .faible: return CielSeuils.quartFaibleMm
        case .moderee: return CielSeuils.quartModereMm
        case .forte: return CielSeuils.quartFortMm
        }
    }

    /// Le bulletin qui parle de la ville : le plus proche dans le rayon, parmi
    /// ceux qui parlent de maintenant.
    static func plusProche(_ metars: [Metar], latitude: Double, longitude: Double, maintenant: Date) -> CielObserve? {
        metars
            .filter { $0.time <= maintenant.addingTimeInterval(CielSeuils.avanceMax)
                && maintenant.timeIntervalSince($0.time) <= CielSeuils.ageMax }
            .map { (distanceKm(latitude, longitude, $0.latitude, $0.longitude), $0) }
            .filter { $0.0 <= CielSeuils.rayonKm }
            // À distance égale, le plus récent.
            .min { a, b in a.0 != b.0 ? a.0 < b.0 : a.1.time > b.1.time }
            .map { paire in
                let (d, m) = paire
                return CielObserve(
                    station: m.station, nom: m.nom, distanceKm: d.rounded(),
                    time: m.time, tombe: codeDuMetar(m.tempsPresent)
                )
            }
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
                tempsPresent: b["wxString"] as? String ?? ""
            )
        }
    }
}
