import XCTest
@testable import Kliima

/// Le guetteur : mêmes cas que `rust/klima-core/src/veille.rs`.
final class VeilleTests: XCTestCase {

    /// 16 h 00 pile.
    private let seizeH = Date(timeIntervalSince1970: 1_778_601_600)
    private let quart = VeilleSeuils.quart

    /// Il est 16 h 07 : le quart entamé est celui de 16 h.
    private var maintenant: Date { seizeH.addingTimeInterval(7 * 60) }

    private func serie(_ pluie: [Double]) -> [QuartSample] {
        pluie.enumerated().map { i, mm in
            QuartSample(
                time: seizeH.addingTimeInterval(Double(i) * quart),
                precipitation: mm,
                weatherCode: mm > 0 ? 61 : 3,
                temperature: 16 - Double(i) * 0.25,
                apparentTemperature: 15,
                windGusts: 20,
                isDay: true
            )
        }
    }

    private func avec(_ q: QuartSample, code: Int? = nil, rafales: Double? = nil, decalage: TimeInterval = 0) -> QuartSample {
        QuartSample(
            time: q.time.addingTimeInterval(decalage), precipitation: q.precipitation,
            weatherCode: code ?? q.weatherCode, temperature: q.temperature,
            apparentTemperature: q.apparentTemperature, windGusts: rafales ?? q.windGusts, isDay: q.isDay
        )
    }

    private let faible = VeillePrecipitation(nature: .pluie, intensite: .faible)

    func testUnePluieObserveeMouilleLeQuartEnCours() throws {
        let sec = serie(Array(repeating: 0, count: 10))
        let vu = Veille.observer(sec, maintenant: maintenant, tombe: 63)
        XCTAssertEqual(vu[0].precipitation, 1.0)
        XCTAssertEqual(vu[0].weatherCode, 63)
        XCTAssertEqual(vu[1], sec[1], "la suite reste à la prévision")
        let v = try XCTUnwrap(Veille.veille(vu, maintenant: maintenant))
        XCTAssertEqual(v.immediat, .cesse(fin: seizeH.addingTimeInterval(quart)))

        let mouillee = serie([0.6, 0.6])
        XCTAssertEqual(Veille.observer(mouillee, maintenant: maintenant, tombe: 65), mouillee)
        XCTAssertEqual(Veille.observer(sec, maintenant: maintenant, tombe: nil), sec)
        XCTAssertEqual(Veille.observer(sec, maintenant: maintenant, tombe: 45), sec, "le brouillard ne mouille pas")
    }

    func testRienNeTombe() throws {
        let v = try XCTUnwrap(Veille.veille(serie(Array(repeating: 0, count: 10)), maintenant: maintenant))
        XCTAssertEqual(v.immediat, .sec)
        XCTAssertEqual(v.suite, .sec)
        XCTAssertEqual(v.quarts.count, VeilleSeuils.horizonQuarts)
        XCTAssertEqual(v.quarts[0].time, seizeH)
        XCTAssertEqual(v.finFenetre, seizeH.addingTimeInterval(8 * quart))
        XCTAssertNil(v.rafales)
    }

    func testLaSerieRepartDuQuartEntame() throws {
        let v = try XCTUnwrap(Veille.veille(serie(Array(repeating: 0, count: 10)), maintenant: seizeH.addingTimeInterval(20 * 60)))
        XCTAssertEqual(v.quarts[0].time, seizeH.addingTimeInterval(quart))
    }

    func testCaCommenceAuQuartSuivant() throws {
        let v = try XCTUnwrap(Veille.veille(serie([0, 0.3, 0.5, 0, 0, 0, 0, 0]), maintenant: maintenant))
        XCTAssertEqual(v.immediat, .commence(debut: seizeH.addingTimeInterval(quart), precipitation: faible))
        XCTAssertEqual(v.suite, .episode(
            debut: seizeH.addingTimeInterval(quart),
            fin: seizeH.addingTimeInterval(3 * quart),
            precipitation: faible,
            cumul: 0.8
        ))
    }

    func testCaContinueEtLIntensiteEstLaPlusForte() throws {
        // 0,8 mm en un quart : 3,2 mm/h, modérée.
        let v = try XCTUnwrap(Veille.veille(serie([0.2, 0.8, 0, 0, 0, 0, 0, 0]), maintenant: maintenant))
        XCTAssertEqual(v.immediat, .dure(precipitation: VeillePrecipitation(nature: .pluie, intensite: .moderee)))
        XCTAssertEqual(v.immediat.code, "continue")
        XCTAssertEqual(v.suite, .accalmie(fin: seizeH.addingTimeInterval(2 * quart), reprise: nil))
    }

    func testCaCessePuisReprend() throws {
        let v = try XCTUnwrap(Veille.veille(serie([0.4, 0, 0, 0, 0.2, 0.2, 0, 0]), maintenant: maintenant))
        XCTAssertEqual(v.immediat, .cesse(fin: seizeH.addingTimeInterval(quart)))
        XCTAssertEqual(v.suite, .accalmie(
            fin: seizeH.addingTimeInterval(quart),
            reprise: seizeH.addingTimeInterval(4 * quart)
        ))
    }

    func testUnePluieQuiNeSArretePas() throws {
        let v = try XCTUnwrap(Veille.veille(serie(Array(repeating: 2, count: 8)), maintenant: maintenant))
        XCTAssertEqual(v.suite, .persiste(precipitation: VeillePrecipitation(nature: .pluie, intensite: .forte), cumul: 16))
    }

    func testUneAverseQuiDebordeDeLaFenetre() throws {
        let v = try XCTUnwrap(Veille.veille(serie([0, 0, 0, 0, 0, 0, 0.3, 0.3, 0.3]), maintenant: maintenant))
        XCTAssertEqual(v.immediat, .sec)
        guard case let .episode(debut, fin, _, cumul) = v.suite else {
            return XCTFail("\(v.suite)")
        }
        XCTAssertEqual(debut, seizeH.addingTimeInterval(6 * quart))
        XCTAssertNil(fin, "le quart sec suivant est hors de la fenêtre")
        XCTAssertEqual(cumul, 0.6)
    }

    func testUnDixiemeNeMouillePasEnDessous() throws {
        let v = try XCTUnwrap(Veille.veille(serie([0.09, 0.09, 0, 0]), maintenant: maintenant))
        XCTAssertEqual(v.immediat, .sec)
        XCTAssertEqual(v.quarts.count, 4, "une série courte se lit telle quelle")
        XCTAssertEqual(v.finFenetre, seizeH.addingTimeInterval(4 * quart))
    }

    func testLesBornesDIntensite() {
        let quart = { (mm: Double) in
            QuartSample(time: self.seizeH, precipitation: mm, weatherCode: 61, temperature: 16,
                        apparentTemperature: 15, windGusts: 20, isDay: true)
        }
        XCTAssertEqual(Veille.intensite(quart(0.6)), .faible) // 2,4 mm/h
        XCTAssertEqual(Veille.intensite(quart(0.625)), .moderee) // 2,5 mm/h
        XCTAssertEqual(Veille.intensite(quart(1.9)), .forte) // 7,6 mm/h
    }

    func testLaNeigeEtLOrageSeDisent() throws {
        var quarts = serie([0.3, 0.3, 0.3, 0])
        quarts[0] = avec(quarts[0], code: 73)
        quarts[1] = avec(quarts[1], code: 73)
        let v = try XCTUnwrap(Veille.veille(quarts, maintenant: maintenant))
        XCTAssertEqual(v.immediat, .dure(precipitation: VeillePrecipitation(nature: .neige, intensite: .faible)))

        // Un quart d'orage suffit à faire de l'épisode un orage.
        quarts[2] = avec(quarts[2], code: 95)
        let orage = try XCTUnwrap(Veille.veille(quarts, maintenant: maintenant))
        XCTAssertEqual(orage.suite.code, "accalmie")
        XCTAssertEqual(Veille.precipitation(Array(quarts[0..<3])).nature, .orage)
        XCTAssertEqual(Veille.precipitation(Array(quarts[0..<3])).key, "veille.kind.orage")
        XCTAssertEqual(VeillePrecipitation(nature: .neige, intensite: .forte).key, "veille.kind.neige.forte")
    }

    func testLesRafalesQuiRetournentUnParapluie() throws {
        var quarts = serie(Array(repeating: 0, count: 8))
        quarts[3] = avec(quarts[3], rafales: 62)
        quarts[5] = avec(quarts[5], rafales: 55)
        quarts[6] = avec(quarts[6], rafales: 62)
        let v = try XCTUnwrap(Veille.veille(quarts, maintenant: maintenant))
        // À égalité, le premier quart.
        XCTAssertEqual(v.rafales, VeilleRafale(valeur: 62, quand: seizeH.addingTimeInterval(3 * quart)))
        // Sous le seuil, rien à signaler.
        XCTAssertNil(try XCTUnwrap(Veille.veille(serie(Array(repeating: 0, count: 8)), maintenant: maintenant)).rafales)
    }

    func testOnRelitUneMinuteApresLeQuartSuivant() {
        let attendu = seizeH.addingTimeInterval(quart + 60)
        XCTAssertEqual(Veille.prochaineLecture(maintenant), attendu)
        // Lu pile au début d'un quart : on attend le suivant.
        XCTAssertEqual(Veille.prochaineLecture(seizeH), attendu)
        XCTAssertEqual(Veille.prochaineLecture(seizeH.addingTimeInterval(quart - 0.001)), attendu)
    }

    func testUnTrouDansLaSerieFaitTaireLeGuetteur() {
        // La série commence dans une heure : pas de quart en cours.
        let tard = serie(Array(repeating: 0, count: 8)).map { avec($0, decalage: 3600) }
        XCTAssertNil(Veille.veille(tard, maintenant: maintenant))
        // La série est finie : rien après maintenant.
        XCTAssertNil(Veille.veille(serie(Array(repeating: 0, count: 8)), maintenant: seizeH.addingTimeInterval(3 * 3600)))
        // Un seul quart : pas de quoi couvrir la demi-heure.
        XCTAssertNil(Veille.veille(serie([0]), maintenant: maintenant))
    }

    /// Le contrat avec Open-Meteo : mêmes cas que `klima-api/src/veille.rs`.
    func testLitLaSerieAuQuartDHeure() throws {
        let reponse = Data("""
        {"latitude": 48.86, "longitude": 2.34, "utc_offset_seconds": 7200,
         "timezone": "Europe/Paris",
         "minutely_15": {
           "time": ["2026-05-12T16:00", "2026-05-12T16:15", "2026-05-12T16:30"],
           "precipitation": [0.0, 0.4, null],
           "weather_code": [3, 61, 95],
           "temperature_2m": [16.2, 15.8, 15.1],
           "apparent_temperature": [15.0, 14.1, 13.9],
           "wind_gusts_10m": [22.0, 31.0, 58.0],
           "is_day": [1, 1, 1]
         }}
        """.utf8)
        let quarts = try XCTUnwrap(AgroWeatherService.decodeQuarts(reponse))
        XCTAssertEqual(quarts.count, 3)
        XCTAssertEqual(quarts[1].time.timeIntervalSince(quarts[0].time), 900)
        XCTAssertEqual(quarts[1].precipitation, 0.4)
        XCTAssertEqual(quarts[2].weatherCode, 95)
        XCTAssertEqual(quarts[2].windGusts, 58)
        // Un `null` se lit 0, sans décaler la série.
        XCTAssertEqual(quarts[2].precipitation, 0)
        XCTAssertTrue(quarts[0].isDay)

        // L'heure est celle de Paris : 16 h là-bas, 14 h en temps universel.
        var utc = Calendar(identifier: .gregorian)
        utc.timeZone = TimeZone(identifier: "UTC")!
        XCTAssertEqual(utc.component(.hour, from: quarts[0].time), 14)
    }

    func testUneReponseSansSerieFaitTaireLeGuetteur() {
        XCTAssertNil(AgroWeatherService.decodeQuarts(Data(#"{"latitude": 1}"#.utf8)))
        XCTAssertNil(AgroWeatherService.decodeQuarts(Data(#"{"timezone":"UTC","minutely_15": {"time": []}}"#.utf8)))
        XCTAssertNil(AgroWeatherService.decodeQuarts(Data("pas du json".utf8)))
    }

    func testLesVariablesSontCellesDuCoeur() {
        XCTAssertEqual(
            AgroWeatherService.quartVariables.joined(separator: ","),
            "precipitation,weather_code,temperature_2m,apparent_temperature,wind_gusts_10m,is_day"
        )
    }
}
