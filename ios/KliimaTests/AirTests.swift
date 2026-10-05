import XCTest
@testable import Kliima

/// L'air de la ville : mêmes cas que `rust/klima-core/src/air.rs`.
final class AirTests: XCTestCase {

    func testLesSixClassesDeLIndiceEuropeen() {
        XCTAssertEqual(Air.qualite(0), .bonne)
        XCTAssertEqual(Air.qualite(20), .bonne)
        XCTAssertEqual(Air.qualite(20.5), .correcte)
        XCTAssertEqual(Air.qualite(40), .correcte)
        XCTAssertEqual(Air.qualite(55), .moyenne)
        XCTAssertEqual(Air.qualite(80), .mediocre)
        XCTAssertEqual(Air.qualite(100), .tresMediocre)
        XCTAssertEqual(Air.qualite(140), .extremementMediocre)
    }

    func testLesNiveauxDePollen() {
        XCTAssertEqual(Air.niveauPollen(0), .faible)
        XCTAssertEqual(Air.niveauPollen(9.9), .faible)
        XCTAssertEqual(Air.niveauPollen(10), .modere)
        XCTAssertEqual(Air.niveauPollen(50), .eleve)
        XCTAssertEqual(Air.niveauPollen(200), .tresEleve)
    }

    func testLePollenDominantEstLePlusPresent() throws {
        let air = AirSample(pollens: [(.bouleau, 12), (.graminees, 64), (.ambroisie, 3)])
        let dominant = try XCTUnwrap(Air.pollenDominant(air))
        XCTAssertEqual(dominant.pollen, .graminees)
        XCTAssertEqual(dominant.grains, 64)
        XCTAssertEqual(dominant.niveau, .eleve)
    }

    func testAEgaliteLePremierDeLaListe() {
        let air = AirSample(pollens: [(.aulne, 20), (.bouleau, 20)])
        XCTAssertEqual(Air.pollenDominant(air)?.pollen, .aulne)
    }

    func testSousUnGrainIlNYARienADire() {
        XCTAssertNil(Air.pollenDominant(AirSample(pollens: [(.bouleau, 0.4)])))
        XCTAssertNil(Air.pollenDominant(AirSample()))
    }

    func testLesCodesSontCeuxDesCatalogues() {
        XCTAssertEqual(QualiteAir.extremementMediocre.key, "air.extremementMediocre")
        XCTAssertEqual(Pollen.graminees.key, "pollen.graminees")
        XCTAssertEqual(Pollen.ambroisie.variable, "ragweed_pollen")
        XCTAssertEqual(NiveauPollen.tresEleve.key, "pollenLevel.tresEleve")
    }
}
