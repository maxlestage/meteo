import XCTest
@testable import Kliima

/// Le radar : mêmes cas que `rust/klima-api/src/radar.rs` et
/// `rust/klima-core/src/radar.rs`.
final class RadarTests: XCTestCase {

    func testLaPrevisionDuRadarSeLit() throws {
        let fil = Data(#"{"cap":75.0,"image":1791313200,"maintenant":0.8,"quarts":[[1791313200,0.8],[1791314100,1.4]],"vitesse":42.0}"#.utf8)
        let p = try XCTUnwrap(Radar.decode(fil))
        XCTAssertEqual(p.image, Date(timeIntervalSince1970: 1_791_313_200))
        XCTAssertEqual(p.maintenant, 0.8)
        XCTAssertEqual(p.quarts.map(\.debit), [0.8, 1.4])
        XCTAssertEqual(p.quarts[1].debut, Date(timeIntervalSince1970: 1_791_314_100))
        XCTAssertEqual(p.deplacement?.vitesse, 42)
        XCTAssertEqual(p.deplacement?.cap, 75)
        XCTAssertTrue(p.voitLaVille)

        let aveugle = try XCTUnwrap(Radar.decode(Data(#"{"image":1791313200,"maintenant":null,"quarts":[]}"#.utf8)))
        XCTAssertNil(aveugle.maintenant)
        XCTAssertFalse(aveugle.voitLaVille)
        XCTAssertNil(aveugle.deplacement)
        XCTAssertNil(Radar.decode(Data("pas du json".utf8)))
    }

    func testCeQuiTombeEtOuCaVa() {
        XCTAssertEqual(Radar.tombe(1.2), 61)
        XCTAssertNil(Radar.tombe(0.2), "trop peu pour mouiller un quart")
        XCTAssertNil(Radar.tombe(nil))
        XCTAssertEqual(Radar.codeDuDebit(0.8), 61)
        XCTAssertEqual(Radar.codeDuDebit(4), 63)
        XCTAssertEqual(Radar.codeDuDebit(12), 65)
        XCTAssertEqual(Radar.direction(0), "dir.n")
        XCTAssertEqual(Radar.direction(350), "dir.n")
        XCTAssertEqual(Radar.direction(75), "dir.e")
        XCTAssertEqual(Radar.direction(225), "dir.so")
        XCTAssertEqual(Radar.direction(-45), "dir.no")
    }

    func testLAdresseDuRadarChezLeRelais() {
        let url = RadarRelais.adresse(
            relais: URL(string: "https://relais.klima/")!,
            Parcelle(name: "Brest", latitude: 48.39, longitude: -4.4861)
        )
        XCTAssertEqual(url?.absoluteString, "https://relais.klima/v1/radar?lat=48.3900&lon=-4.4861")
    }
}
