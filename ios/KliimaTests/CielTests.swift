import XCTest
@testable import Kliima

/// Le ciel observé : mêmes cas que `rust/klima-core/src/ciel.rs` et
/// `rust/klima-api/src/ciel.rs`.
final class CielTests: XCTestCase {

    private let maintenant = Date(timeIntervalSince1970: 1_791_302_400)

    private func metar(_ station: String, _ latitude: Double, _ longitude: Double, age: Double, _ temps: String) -> Metar {
        Metar(
            station: station, nom: station, latitude: latitude, longitude: longitude,
            time: maintenant.addingTimeInterval(-age * 60), tempsPresent: temps
        )
    }

    func testLeTempsPresentDevientUnCodeDeLOmm() {
        XCTAssertEqual(Ciel.codeDuMetar("-RA"), 61)
        XCTAssertEqual(Ciel.codeDuMetar("RA BR"), 63)
        XCTAssertEqual(Ciel.codeDuMetar("+RA"), 65)
        XCTAssertEqual(Ciel.codeDuMetar("-DZ"), 51)
        XCTAssertEqual(Ciel.codeDuMetar("-SHRA"), 80)
        XCTAssertEqual(Ciel.codeDuMetar("+SHRA"), 82)
        XCTAssertEqual(Ciel.codeDuMetar("SHSN"), 85)
        XCTAssertEqual(Ciel.codeDuMetar("-SN"), 71)
        XCTAssertEqual(Ciel.codeDuMetar("FZRA"), 66)
        XCTAssertEqual(Ciel.codeDuMetar("TSRA"), 95)
        XCTAssertEqual(Ciel.codeDuMetar("+TSGR"), 99)
        XCTAssertEqual(Ciel.codeDuMetar("TS"), 95)
        XCTAssertEqual(Ciel.codeDuMetar("-RASN"), 71, "la neige l'emporte sur la pluie")
    }

    func testCeQuiNeTombePasSurLaVilleNeComptePas() {
        XCTAssertNil(Ciel.codeDuMetar(""))
        XCTAssertNil(Ciel.codeDuMetar("BR"))
        XCTAssertNil(Ciel.codeDuMetar("FG"))
        XCTAssertNil(Ciel.codeDuMetar("VCSH"), "une averse au voisinage")
        XCTAssertNil(Ciel.codeDuMetar("RERA"), "une pluie récente, finie")
        XCTAssertNil(Ciel.codeDuMetar("BLSN"), "de la neige soulevée par le vent")
        XCTAssertEqual(Ciel.codeDuMetar("-DZ -RA"), 61, "le plus marqué des groupes")
    }

    func testNatureEtIntensiteDUnCode() {
        XCTAssertEqual(Ciel.precipitationDuCode(82), VeillePrecipitation(nature: .pluie, intensite: .forte))
        XCTAssertEqual(Ciel.precipitationDuCode(73), VeillePrecipitation(nature: .neige, intensite: .moderee))
        XCTAssertEqual(Ciel.precipitationDuCode(95).nature, .orage)
        XCTAssertEqual(Ciel.quartObserveMm(.faible), CielSeuils.quartFaibleMm)
        XCTAssertEqual(Ciel.quartObserveMm(.forte), CielSeuils.quartFortMm)
        // La force vient du signe : un « +TSRA » est un orage fort.
        let fort = Ciel.tombeDuMetar("+TSRA")
        XCTAssertEqual(fort, CielTombe(code: 95, intensite: .forte))
        XCTAssertEqual(fort?.precipitation.key, "veille.kind.orage")
        XCTAssertEqual(Ciel.tombeDuMetar("-RA")?.intensite, .faible)
        XCTAssertEqual(Ciel.tombeDuMetar("RA")?.intensite, .moderee)
    }

    func testLeBulletinRetenuEstLePlusProcheEtAssezRecent() throws {
        let (lat, lon) = (48.39, -4.4861)
        let bulletins = [
            metar("LFRL", 48.279, -4.439, age: 10, "-RA"),
            metar("LFRB", 48.444, -4.412, age: 10, ""),
            metar("LFRJ", 48.527, -4.138, age: 10, "+RA"),
        ]
        let ciel = try XCTUnwrap(Ciel.plusProche(bulletins, latitude: lat, longitude: lon, maintenant: maintenant))
        XCTAssertEqual(ciel.station, "LFRB")
        XCTAssertEqual(ciel.distanceKm, 8)
        XCTAssertNil(ciel.tombe)

        let vieux = [metar("LFRB", 48.444, -4.412, age: 120, ""), metar("LFRL", 48.279, -4.439, age: 20, "-RA")]
        let autre = try XCTUnwrap(Ciel.plusProche(vieux, latitude: lat, longitude: lon, maintenant: maintenant))
        XCTAssertEqual(autre.station, "LFRL")
        XCTAssertEqual(autre.tombe?.code, 61)
        XCTAssertEqual(autre.precipitation?.key, "veille.kind.pluie.faible")
    }

    func testTropLoinOuTropVieuxLeCielNEstPasObserve() {
        XCTAssertNil(Ciel.plusProche([metar("LFRJ", 48.527, -4.138, age: 10, "-RA")], latitude: 48, longitude: -3, maintenant: maintenant))
        XCTAssertNil(Ciel.plusProche([metar("LFRB", 48.444, -4.412, age: 80, "-RA")], latitude: 48.39, longitude: -4.4861, maintenant: maintenant))
        XCTAssertNil(Ciel.plusProche([metar("LFRB", 48.444, -4.412, age: -20, "-RA")], latitude: 48.39, longitude: -4.4861, maintenant: maintenant))
    }

    func testLaTendanceDesPrevisionnistes() {
        let merignac = "METAR LFBD 061530Z AUTO 04004KT 9999 1900 +TSRA ///CB 21/18 Q1012 TEMPO VRB15G30KT 1200 TSRA"
        XCTAssertEqual(
            Ciel.tendanceDuMetar(merignac),
            .changement(passager: true, tombe: CielTombe(code: 95, intensite: .moderee), sec: false)
        )
        XCTAssertEqual(Ciel.tendanceDuMetar("METAR LFPB 061400Z AUTO 14004KT CAVOK 25/10 Q1015 NOSIG"), .stable)
        XCTAssertEqual(
            Ciel.tendanceDuMetar("METAR LFRB 061400Z 01006KT 9999 -RA BKN008 17/16 Q1014 BECMG NSW"),
            .changement(passager: false, tombe: nil, sec: true)
        )
        XCTAssertEqual(
            Ciel.tendanceDuMetar("METAR LFRS 061400Z 24010KT 9999 FEW020 17/12 Q1014 BECMG 27015KT"),
            .changement(passager: false, tombe: nil, sec: false),
            "du vent seulement : rien ne tombe"
        )
        XCTAssertEqual(Ciel.tendanceDuMetar("METAR KJFK 061451Z 18010KT 10SM -RA OVC020 RMK AO2 TEMPO"), .inconnue)
        XCTAssertEqual(Ciel.tendanceDuMetar("METAR LFPM 061400Z AUTO VRB02KT CAVOK 26/11 Q1015"), .inconnue)
    }

    private func vu(_ tombe: CielTombe?, _ tendance: CielTendance) -> CielObserve {
        CielObserve(
            station: "LFBD", nom: "Bordeaux/Merignac", distanceKm: 8, time: maintenant,
            tombe: tombe, tendance: tendance
        )
    }

    func testCeQuOnVoitTombeAussiLongtempsQueLeBulletinLeDit() {
        let pluie = CielTombe(code: 63, intensite: .moderee)
        XCTAssertEqual(vu(pluie, .stable).tombeJusquA, maintenant.addingTimeInterval(CielSeuils.tendance))
        XCTAssertEqual(vu(pluie, .inconnue).tombeJusquA, maintenant.addingTimeInterval(CielSeuils.validite))
        XCTAssertEqual(vu(pluie, .changement(passager: false, tombe: nil, sec: true)).tombeJusquA, maintenant)
        XCTAssertEqual(
            vu(pluie, .changement(passager: true, tombe: pluie, sec: false)).tombeJusquA,
            maintenant.addingTimeInterval(CielSeuils.validite)
        )
        XCTAssertEqual(
            vu(pluie, .changement(passager: false, tombe: pluie, sec: false)).tombeJusquA,
            maintenant.addingTimeInterval(CielSeuils.tendance)
        )
        XCTAssertNil(vu(nil, .stable).tombeJusquA)
    }

    func testLAnnonceDeLAeroport() throws {
        let orage = CielTombe(code: 95, intensite: .moderee)
        let a = try XCTUnwrap(vu(nil, .changement(passager: true, tombe: orage, sec: false)).annonce)
        XCTAssertTrue(a.passagere)
        XCTAssertEqual(a.precipitation.key, "veille.kind.orage")
        XCTAssertEqual(a.jusquA, maintenant.addingTimeInterval(CielSeuils.tendance))
        XCTAssertNil(vu(orage, .stable).annonce)
        XCTAssertNil(vu(nil, .changement(passager: false, tombe: nil, sec: true)).annonce)
    }

    func testUnNomDAeroportLisible() {
        XCTAssertEqual(Ciel.nomLisible("Paris/Le Bourge Arpt, ID, FR"), "Paris/Le Bourge")
        XCTAssertEqual(Ciel.nomLisible("Brest/Guipavas Intl Arpt, BRE, FR"), "Brest/Guipavas")
        XCTAssertEqual(Ciel.nomLisible("Villacoublay, ID, FR"), "Villacoublay")
    }

    func testLeCadreCouvreLeRayonAToutesLesLatitudes() {
        let (s, o, n, e) = Ciel.cadre(48.39, -4.4861)
        XCTAssertEqual([s, o, n, e], [48.03, -5.03, 48.75, -3.94])
        let (s2, o2, n2, e2) = Ciel.cadre(60, 10)
        XCTAssertGreaterThan(Ciel.distanceKm(60, 10, 60, e2), CielSeuils.rayonKm)
        XCTAssertGreaterThan(Ciel.distanceKm(60, 10, 60, o2), CielSeuils.rayonKm)
        XCTAssertGreaterThan(Ciel.distanceKm(60, 10, s2, 10), CielSeuils.rayonKm)
        XCTAssertGreaterThan(Ciel.distanceKm(60, 10, n2, 10), CielSeuils.rayonKm)
        XCTAssertEqual(Ciel.cadre(89.9, 0).2, 90)
    }

    func testLaDistanceAVolDOiseau() {
        XCTAssertEqual(Ciel.distanceKm(48.8566, 2.3522, 45.764, 4.8357), 392, accuracy: 2)
    }

    // MARK: La réponse de l'Aviation Weather Center

    private let brest = Data("""
    [
      {"icaoId":"LFRJ","obsTime":1791295200,"wxString":null,"lat":48.527,"lon":-4.138,
       "name":"Landivisiau Arpt, BRE, FR",
       "rawOb":"METAR LFRJ 061400Z AUTO 01008KT 9999 ///CB 18/16 Q1013 TEMPO 06015G25KT 2500 TSRA BECMG BKN005"},
      {"icaoId":"LFRL","obsTime":1791295200,"wxString":"-RA","lat":48.279,"lon":-4.439,
       "name":"Lanveoc/Poulmic Arpt, BRE, FR"},
      {"icaoId":"LFRB","obsTime":1791295200,"lat":48.444,"lon":-4.412,
       "name":"Brest/Guipavas Intl Arpt, BRE, FR"},
      {"icaoId":"XXXX","obsTime":1791295200}
    ]
    """.utf8)

    func testLesBulletinsSeLisent() {
        let bulletins = Ciel.decodeMetars(brest)
        XCTAssertEqual(bulletins.count, 3, "sans position, écarté")
        XCTAssertEqual(bulletins[1].station, "LFRL")
        XCTAssertEqual(bulletins[1].nom, "Lanveoc/Poulmic")
        XCTAssertEqual(bulletins[1].tempsPresent, "-RA")
        XCTAssertEqual(bulletins[1].time, Date(timeIntervalSince1970: 1_791_295_200))
        XCTAssertEqual(bulletins[2].tempsPresent, "")
        guard case .changement(passager: true, tombe: .some(_), sec: _) = bulletins[0].tendance else {
            return XCTFail("la tendance se lit dans le bulletin brut")
        }
        XCTAssertEqual(bulletins[2].tendance, .inconnue)
        XCTAssertTrue(Ciel.decodeMetars(Data("pas du json".utf8)).isEmpty)
        XCTAssertTrue(Ciel.decodeMetars(Data("{}".utf8)).isEmpty)
    }

    func testLeCielDeBrestEstCeluiDeGuipavas() throws {
        let bulletins = Ciel.decodeMetars(brest)
        let apres = Date(timeIntervalSince1970: 1_791_295_200 + 600)
        let ciel = try XCTUnwrap(Ciel.plusProche(bulletins, latitude: 48.39, longitude: -4.4861, maintenant: apres))
        XCTAssertEqual(ciel.station, "LFRB")
        XCTAssertNil(ciel.tombe)
        let sud = try XCTUnwrap(Ciel.plusProche(bulletins, latitude: 48.30, longitude: -4.45, maintenant: apres))
        XCTAssertEqual(sud.station, "LFRL")
        XCTAssertEqual(sud.tombe?.code, 61)
    }

    func testLEnsembleTrouveLAeroportDeLaVille() {
        let sources = WeatherProviders.ensemble(
            openMeteo: nil, met: nil, station: nil, aviation: brest,
            point: Parcelle(name: "Lanvéoc", latitude: 48.30, longitude: -4.45),
            maintenant: Date(timeIntervalSince1970: 1_791_295_200 + 600)
        )
        XCTAssertEqual(sources.ciel?.tombe?.code, 61)
    }
}
