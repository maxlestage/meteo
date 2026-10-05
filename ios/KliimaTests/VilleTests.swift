import XCTest
@testable import Kliima

/// Les règles de la ville : mêmes cas que `rust/klima-core/src/ville.rs`.
final class VilleTests: XCTestCase {

    /// 14 h pile.
    private let t0 = Date(timeIntervalSince1970: TimeInterval(1_790_000_000 - 1_790_000_000 % 3600))

    /// Une heure sèche, douce, sans vent ni soleil fort.
    private func heure(_ decalage: Int) -> HourlySample {
        HourlySample(
            time: t0.addingTimeInterval(Double(decalage) * 3600),
            weatherCode: 3, isDay: true, precipitationProbability: 10,
            temperature: 18, apparentTemperature: 17, relativeHumidity: 60, dewPoint: 10,
            precipitation: 0, windSpeed: 10, windGusts: 20, uvIndex: 1
        )
    }

    private func serie(_ n: Int) -> [HourlySample] { (0..<n).map(heure) }

    /// Une heure modifiée : les échantillons sont immuables.
    private func avec(_ h: HourlySample, probabilite: Double? = nil, pluie: Double? = nil,
                      temperature: Double? = nil, ressenti: Double? = nil,
                      rafales: Double? = nil, uv: Double? = nil) -> HourlySample {
        HourlySample(
            time: h.time, weatherCode: h.weatherCode, isDay: h.isDay,
            precipitationProbability: probabilite ?? h.precipitationProbability,
            temperature: temperature ?? h.temperature,
            apparentTemperature: ressenti ?? h.apparentTemperature,
            relativeHumidity: h.relativeHumidity, dewPoint: h.dewPoint,
            precipitation: pluie ?? h.precipitation, windSpeed: h.windSpeed,
            windGusts: rafales ?? h.windGusts, uvIndex: uv ?? h.uvIndex
        )
    }

    func testUneHeureEstPluvieuseParLEauOuParLeRisque() {
        XCTAssertFalse(Ville.pluvieuse(heure(0)))
        XCTAssertTrue(Ville.pluvieuse(avec(heure(0), pluie: 0.1)))
        XCTAssertTrue(Ville.pluvieuse(avec(heure(0), probabilite: 50)))
        XCTAssertFalse(Ville.pluvieuse(avec(heure(0), probabilite: 49)))
    }

    func testSansPluieOnDitSurCombienDHeures() {
        XCTAssertEqual(Ville.prochainePluie(serie(24)), .aucune(heures: 12))
        XCTAssertEqual(Ville.prochainePluie(serie(5)), .aucune(heures: 5))
        XCTAssertEqual(Ville.prochainePluie([]), .aucune(heures: 0))
    }

    func testLaPluieQuiVientDitSonHeureSonRisqueEtSonCumul() {
        var s = serie(24)
        s[3] = avec(s[3], probabilite: 60, pluie: 0.4)
        s[4] = avec(s[4], probabilite: 80, pluie: 1.2)
        s[6] = avec(s[6], pluie: 3.0) // un second épisode, qui ne compte pas
        XCTAssertEqual(
            Ville.prochainePluie(s),
            .prevue(debut: t0.addingTimeInterval(3 * 3600), probabilite: 80, cumul: 1.6)
        )
    }

    func testLaPluieAuDelaDeDouzeHeuresNeSeDitPas() {
        var s = serie(24)
        s[12] = avec(s[12], pluie: 2)
        XCTAssertEqual(Ville.prochainePluie(s), .aucune(heures: 12))
    }

    func testIlPleutEtCelaCesseALaPremiereHeureSeche() {
        var s = serie(24)
        s[0] = avec(s[0], pluie: 0.8)
        s[1] = avec(s[1], probabilite: 70)
        XCTAssertEqual(Ville.prochainePluie(s), .enCours(fin: t0.addingTimeInterval(2 * 3600)))

        for i in 0..<12 { s[i] = avec(s[i], pluie: 1) }
        XCTAssertEqual(Ville.prochainePluie(s), .enCours(fin: nil))
    }

    func testUneBelleJourneeDouceNeDemandeRien() {
        XCTAssertTrue(Ville.conseils(serie(24)).isEmpty)
        XCTAssertTrue(Ville.conseils([]).isEmpty)
    }

    func testChaqueConseilASaRaison() {
        var s = serie(24)
        s[2] = avec(s[2], probabilite: 55)
        s[5] = avec(s[5], ressenti: 9)
        s[1] = avec(s[1], uv: 6.2)
        s[3] = avec(s[3], temperature: 31)
        s[8] = avec(s[8], temperature: -1)
        s[9] = avec(s[9], rafales: 55)
        XCTAssertEqual(Ville.conseils(s), [.parapluie, .manteau, .cremeSolaire, .hydratation, .gel, .vent])
    }

    func testLesLunettesAvantLaCremeEtJamaisLesDeux() {
        var s = serie(24)
        s[1] = avec(s[1], uv: 3)
        XCTAssertEqual(Ville.conseils(s), [.lunettes])
        s[1] = avec(s[1], uv: 6)
        XCTAssertEqual(Ville.conseils(s), [.cremeSolaire])
    }

    func testCeQuiArriveApresDouzeHeuresNeChangePasLesConseils() {
        var s = serie(24)
        s[12] = avec(s[12], temperature: -5)
        s[13] = avec(s[13], pluie: 4)
        XCTAssertTrue(Ville.conseils(s).isEmpty)
    }

    func testLEchelleUvDeLOmsArronditAvantDeClasser() {
        XCTAssertEqual(Ville.niveauUv(0), .faible)
        XCTAssertEqual(Ville.niveauUv(2.4), .faible)
        XCTAssertEqual(Ville.niveauUv(2.5), .modere)
        XCTAssertEqual(Ville.niveauUv(5), .modere)
        XCTAssertEqual(Ville.niveauUv(6), .eleve)
        XCTAssertEqual(Ville.niveauUv(8), .tresEleve)
        XCTAssertEqual(Ville.niveauUv(10.4), .tresEleve)
        XCTAssertEqual(Ville.niveauUv(11), .extreme)
    }

    func testLHeureDeMaintenantSeTrouveDansLaSerie() {
        let s = serie(4)
        XCTAssertEqual(Ville.heure(contenant: t0.addingTimeInterval(2 * 3600 + 600), dans: s)?.time,
                       t0.addingTimeInterval(2 * 3600))
        XCTAssertEqual(Ville.heure(contenant: t0.addingTimeInterval(-1), dans: s)?.time, t0)
        XCTAssertNil(Ville.heure(contenant: t0, dans: []))
    }

    func testLesCodesSontCeuxDesCatalogues() {
        XCTAssertEqual(Conseil.cremeSolaire.key, "advice.cremeSolaire")
        XCTAssertEqual(NiveauUv.tresEleve.key, "uv.tresEleve")
        XCTAssertEqual(Pluie.enCours(fin: nil).code, "enCours")
    }
}
