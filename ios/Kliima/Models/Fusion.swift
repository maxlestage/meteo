import Foundation

/// La prévision recoupée : tous les fournisseurs, une seule météo.
///
/// Miroir de `rust/klima-core/src/fusion.rs`, avec les mêmes cas de test.
/// Toute règle ajoutée d'un côté se porte de l'autre.
///
/// Pour chaque heure, chaque source vote, et Kliima ‣ retient la médiane de ce
/// qui se mesure — température, ressenti, pluie, vent, rafales —, tire le
/// risque de pluie de la part des sources qui mouillent (mêlée à moitié au
/// risque publié), et le temps qu'il fait de la majorité. Ce qu'une seule
/// source fournit — humidité, point de rosée, UV, jour ou nuit, lever et
/// coucher — reste celui de la prévision de base ; une heure sans voix garde
/// ses valeurs de base.
enum FusionSeuils {
    /// Une source mouille une heure à partir de ce cumul (mm).
    static let pluieMm = VilleSeuils.pluieMm
    /// Part du risque publié dans le risque recoupé.
    static let poidsRisquePublie = 0.5
    /// Les codes météo à partir desquels il tombe quelque chose.
    static let codeMouille = 51
}

/// Ce qu'une source annonce pour une heure. Tout est facultatif.
struct HeureSource: Equatable, Sendable {
    let time: Date
    var temperature: Double?
    var ressenti: Double?
    var precipitation: Double?
    var probabilite: Double?
    var vent: Double?
    var rafales: Double?
    var code: Int?
}

/// Ce qu'une source annonce pour une journée.
struct JourSource: Equatable, Sendable {
    let date: Date
    var minimum: Double?
    var maximum: Double?
    var cumul: Double?
    var probabilite: Double?
    var rafales: Double?
    var code: Int?
}

/// Une source et sa série.
struct SerieSource: Equatable, Sendable {
    let sourceId: String
    var heures: [HeureSource]
    var jours: [JourSource] = []
}

/// Ce que la fusion a produit.
struct Recoupement: Equatable {
    let heures: [HourlySample]
    let jours: [DailySample]
    let courant: CurrentSample
    /// Les sources qui ont voté, dans l'ordre où on les a reçues.
    let sources: [String]
}

enum Fusion {

    /// La médiane ; pour un nombre pair, la moyenne des deux du milieu.
    static func mediane(_ valeurs: [Double]) -> Double? {
        let v = valeurs.filter(\.isFinite).sorted()
        guard !v.isEmpty else { return nil }
        let milieu = v.count / 2
        return v.count % 2 == 0 ? (v[milieu - 1] + v[milieu]) / 2 : v[milieu]
    }

    /// Le risque recoupé (%), `nil` si aucune source n'a dit sa pluie.
    static func risque(_ cumuls: [Double], _ publies: [Double]) -> Double? {
        guard !cumuls.isEmpty else { return nil }
        let part = Double(cumuls.filter { $0 >= FusionSeuils.pluieMm }.count) / Double(cumuls.count) * 100
        let melange: Double
        if let publie = mediane(publies) {
            melange = FusionSeuils.poidsRisquePublie * publie + (1 - FusionSeuils.poidsRisquePublie) * part
        } else {
            melange = part
        }
        return melange.rounded()
    }

    /// La majorité mouillée ou sèche, puis le code le plus cité de ce camp ; à
    /// égalité, le plus marqué.
    static func codeMajoritaire(_ codes: [Int], _ cumuls: [Double]) -> Int? {
        guard !codes.isEmpty else { return nil }
        let mouilles: Bool
        if cumuls.isEmpty {
            mouilles = codes.filter { $0 >= FusionSeuils.codeMouille }.count * 2 >= codes.count
        } else {
            mouilles = cumuls.filter { $0 >= FusionSeuils.pluieMm }.count * 2 >= cumuls.count
        }
        let camp = codes.filter { ($0 >= FusionSeuils.codeMouille) == mouilles }
        let candidats = camp.isEmpty ? codes : camp
        var comptes: [Int: Int] = [:]
        for code in candidats { comptes[code, default: 0] += 1 }
        return comptes.max { a, b in a.value != b.value ? a.value < b.value : a.key < b.key }?.key
    }

    /// Recoupe la prévision de base avec toutes les séries reçues.
    /// `observation` : la température d'une station proche, qui ne vote que
    /// pour l'instant présent.
    static func recouper(
        heures base: [HourlySample],
        jours baseJours: [DailySample],
        courant baseCourant: CurrentSample,
        series: [SerieSource],
        observation: Double?
    ) -> Recoupement {
        let index: [[Date: HeureSource]] = series.map { serie in
            Dictionary(serie.heures.map { ($0.time, $0) }, uniquingKeysWith: { premier, _ in premier })
        }
        let indexJours: [[Date: JourSource]] = series.map { serie in
            Dictionary(serie.jours.map { ($0.date, $0) }, uniquingKeysWith: { premier, _ in premier })
        }

        let heures = base.map { heure in
            fusionnerHeure(heure, index.compactMap { $0[heure.time] })
        }
        let jours = baseJours.map { jour in
            fusionnerJour(jour, indexJours.compactMap { $0[jour.date] })
        }
        let courant = fusionnerCourant(baseCourant, base.first, heures.first, index, observation)
        let sources = series
            .filter { $0.heures.contains { $0.temperature != nil } }
            .map(\.sourceId)
        return Recoupement(heures: heures, jours: jours, courant: courant, sources: sources)
    }

    private static func fusionnerHeure(_ base: HourlySample, _ voix: [HeureSource]) -> HourlySample {
        guard !voix.isEmpty else { return base }
        let temperature = mediane(voix.compactMap(\.temperature)) ?? base.temperature
        let ressenti = mediane(voix.compactMap(\.ressenti))
            ?? base.apparentTemperature + (temperature - base.temperature)
        let cumuls = voix.compactMap(\.precipitation).filter(\.isFinite)
        let publies = voix.compactMap(\.probabilite).filter(\.isFinite)
        let codes = voix.compactMap(\.code)

        return HourlySample(
            time: base.time,
            weatherCode: codeMajoritaire(codes, cumuls) ?? base.weatherCode,
            isDay: base.isDay,
            precipitationProbability: risque(cumuls, publies) ?? base.precipitationProbability,
            temperature: temperature,
            apparentTemperature: ressenti,
            relativeHumidity: base.relativeHumidity,
            dewPoint: base.dewPoint,
            precipitation: mediane(cumuls) ?? base.precipitation,
            windSpeed: mediane(voix.compactMap(\.vent)) ?? base.windSpeed,
            windGusts: mediane(voix.compactMap(\.rafales)) ?? base.windGusts,
            uvIndex: base.uvIndex
        )
    }

    private static func fusionnerJour(_ base: DailySample, _ voix: [JourSource]) -> DailySample {
        guard !voix.isEmpty else { return base }
        let cumuls = voix.compactMap(\.cumul).filter(\.isFinite)
        let publies = voix.compactMap(\.probabilite).filter(\.isFinite)
        return DailySample(
            date: base.date,
            weatherCode: codeMajoritaire(voix.compactMap(\.code), cumuls) ?? base.weatherCode,
            temperatureMin: mediane(voix.compactMap(\.minimum)) ?? base.temperatureMin,
            temperatureMax: mediane(voix.compactMap(\.maximum)) ?? base.temperatureMax,
            precipitationSum: mediane(cumuls) ?? base.precipitationSum,
            precipitationProbabilityMax: risque(cumuls, publies) ?? base.precipitationProbabilityMax,
            windGustsMax: mediane(voix.compactMap(\.rafales)) ?? base.windGustsMax,
            uvIndexMax: base.uvIndexMax,
            sunrise: base.sunrise,
            sunset: base.sunset
        )
    }

    /// L'instant présent : la base décalée d'autant que l'heure en cours l'a été
    /// par la fusion, puis la médiane avec la station s'il y en a une.
    private static func fusionnerCourant(
        _ base: CurrentSample,
        _ baseHeure: HourlySample?,
        _ heure: HourlySample?,
        _ index: [[Date: HeureSource]],
        _ observation: Double?
    ) -> CurrentSample {
        guard let baseHeure, let heure else { return base }
        let aVote = index.contains { $0[heure.time]?.temperature != nil }
        let mesure = observation.flatMap { $0.isFinite ? $0 : nil }
        guard aVote || mesure != nil else { return base }

        let prevue = base.temperature + (heure.temperature - baseHeure.temperature)
        let temperature = mesure.flatMap { mediane([prevue, $0]) } ?? prevue
        let vent = max(base.windSpeed + (heure.windSpeed - baseHeure.windSpeed), 0)
        let rafales = max(base.windGusts + (heure.windGusts - baseHeure.windGusts), vent)
        return CurrentSample(
            time: base.time,
            temperature: temperature,
            apparentTemperature: base.apparentTemperature + (temperature - base.temperature),
            weatherCode: aVote ? heure.weatherCode : base.weatherCode,
            isDay: base.isDay,
            relativeHumidity: base.relativeHumidity,
            windSpeed: vent,
            windGusts: rafales,
            pressure: base.pressure
        )
    }
}
