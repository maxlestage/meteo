import Foundation

/// L'apprenti : chaque modèle noté sur ce qui s'est vraiment passé.
///
/// Miroir de `rust/klima-core/src/apprenti.rs`, avec les mêmes cas de test.
/// Toute règle ajoutée d'un côté se porte de l'autre.
///
/// Le relais compare, sur les sept derniers jours, ce que chaque modèle avait
/// annoncé la veille à ce que l'aéroport le plus proche a relevé
/// (`/v1/apprenti`). L'apprenti en tire un biais de température par modèle,
/// et le retranche avant le vote : le vote reste un vote, entre des modèles
/// redressés. Rien sous `ApprentiSeuils.relevesMin` heures comparées ; un
/// biais retenu tant que les comparaisons sont peu nombreuses ; jamais plus
/// de `correctionMax`. La pluie n'est pas corrigée : elle se vote, et ce qui
/// tombe se voit. Sa justesse se montre, elle ne pèse pas.
enum ApprentiSeuils {
    /// Heures comparées en dessous desquelles on ne corrige rien.
    static let relevesMin = 24
    /// Le biais appris est multiplié par `n / (n + retenue)`.
    static let retenue = 24.0
    /// La plus grande correction de température permise (°C).
    static let correctionMax = 3.0
    /// Un relevé ne compte pour une heure qu'à moins de tant de secondes d'elle.
    static let toleranceS: TimeInterval = 600
    /// Un modèle « mouille » une heure à partir de ce cumul (mm).
    static let pluieMm = VilleSeuils.pluieMm
}

/// Ce que l'aéroport a relevé.
struct ApprentiReleve: Equatable {
    let time: Date
    let temperature: Double?
    let mouille: Bool
}

/// Ce qu'un modèle avait annoncé la veille pour une heure.
struct ApprentiAnnonce: Equatable {
    let time: Date
    let temperature: Double?
    let precipitation: Double?
}

/// La note d'une source.
struct ApprentiNote: Equatable {
    let sourceId: String
    /// Heures où la température a pu être comparée.
    let n: Int
    /// Moyenne de (annoncé − relevé), °C. Positif : le modèle voit trop chaud.
    let biais: Double
    /// Moyenne de |annoncé − relevé|, °C.
    let ecart: Double
    let pluieN: Int
    let pluieJustes: Int

    /// Ce qu'on retranche à la température de ce modèle avant le vote (°C).
    var correction: Double {
        guard n >= ApprentiSeuils.relevesMin, biais.isFinite else { return 0 }
        let retenu = biais * Double(n) / (Double(n) + ApprentiSeuils.retenue)
        let borne = min(max(retenu, -ApprentiSeuils.correctionMax), ApprentiSeuils.correctionMax)
        return (borne * 10).rounded() / 10
    }

    /// La part des heures où il a dit juste sur la pluie (0–100).
    var justessePluie: Double? {
        guard pluieN > 0 else { return nil }
        return (Double(pluieJustes) / Double(pluieN) * 100).rounded()
    }
}

/// Ce que l'apprenti a appris pour une ville.
struct Apprentissage: Equatable, Codable {
    let station: String
    let nom: String
    let jours: Int
    let notes: [ApprentiNote]

    enum CodingKeys: String, CodingKey { case station, nom, jours, notes }

    init(station: String, nom: String, jours: Int, notes: [ApprentiNote]) {
        self.station = station
        self.nom = nom
        self.jours = jours
        self.notes = notes
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        station = try c.decode(String.self, forKey: .station)
        nom = try c.decodeIfPresent(String.self, forKey: .nom) ?? ""
        jours = try c.decodeIfPresent(Int.self, forKey: .jours) ?? 7
        notes = try c.decode([ApprentiNote].self, forKey: .notes)
    }
}

extension ApprentiNote: Codable {
    enum CodingKeys: String, CodingKey {
        case sourceId = "id", n, biais, ecart, pluieN = "pluie_n", pluieJustes = "pluie_justes"
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        sourceId = try c.decode(String.self, forKey: .sourceId)
        n = try c.decode(Int.self, forKey: .n)
        biais = try c.decode(Double.self, forKey: .biais)
        ecart = try c.decode(Double.self, forKey: .ecart)
        pluieN = try c.decodeIfPresent(Int.self, forKey: .pluieN) ?? 0
        pluieJustes = try c.decodeIfPresent(Int.self, forKey: .pluieJustes) ?? 0
    }
}

enum Apprenti {

    private static func releve(_ releves: [ApprentiReleve], pour heure: Date) -> ApprentiReleve? {
        releves
            .filter { abs($0.time.timeIntervalSince(heure)) <= ApprentiSeuils.toleranceS }
            .min { abs($0.time.timeIntervalSince(heure)) < abs($1.time.timeIntervalSince(heure)) }
    }

    /// Note une source. Seules les heures déjà passées comptent.
    static func noter(_ sourceId: String, annonces: [ApprentiAnnonce], releves: [ApprentiReleve], maintenant: Date) -> ApprentiNote {
        var (n, somme, absolue) = (0, 0.0, 0.0)
        var (pluieN, pluieJustes) = (0, 0)
        for annonce in annonces where annonce.time <= maintenant {
            guard let vu = releve(releves, pour: annonce.time) else { continue }
            if let prevu = annonce.temperature, let mesure = vu.temperature, prevu.isFinite, mesure.isFinite {
                n += 1
                somme += prevu - mesure
                absolue += abs(prevu - mesure)
            }
            if let cumul = annonce.precipitation, cumul.isFinite {
                pluieN += 1
                if (cumul >= ApprentiSeuils.pluieMm) == vu.mouille { pluieJustes += 1 }
            }
        }
        func moyenne(_ x: Double) -> Double { n == 0 ? 0 : (x / Double(n) * 100).rounded() / 100 }
        return ApprentiNote(sourceId: sourceId, n: n, biais: moyenne(somme), ecart: moyenne(absolue),
                            pluieN: pluieN, pluieJustes: pluieJustes)
    }

    /// Redresse les séries avant le vote. Une source sans note, ou trop peu
    /// notée, reste telle quelle.
    static func corriger(_ series: [SerieSource], _ notes: [ApprentiNote]) -> [SerieSource] {
        series.map { serie in
            guard let note = notes.first(where: { $0.sourceId == serie.sourceId }) else { return serie }
            let c = note.correction
            guard c != 0 else { return serie }
            var redressee = serie
            redressee.heures = serie.heures.map { heure in
                var h = heure
                h.temperature = heure.temperature.map { $0 - c }
                h.ressenti = heure.ressenti.map { $0 - c }
                return h
            }
            redressee.jours = serie.jours.map { jour in
                var j = jour
                j.minimum = jour.minimum.map { $0 - c }
                j.maximum = jour.maximum.map { $0 - c }
                return j
            }
            return redressee
        }
    }

    /// La source la plus juste en température, parmi celles assez notées.
    static func meilleure(_ notes: [ApprentiNote]) -> ApprentiNote? {
        notes.filter { $0.n >= ApprentiSeuils.relevesMin }.min { $0.ecart < $1.ecart }
    }

    /// La réponse de `/v1/apprenti`. `nil` sans juge — aucun aéroport assez
    /// proche — ou si la réponse n'en est pas une.
    static func decode(_ data: Data) -> Apprentissage? {
        guard let appris = try? JSONDecoder().decode(Apprentissage.self, from: data) else { return nil }
        return appris
    }
}
