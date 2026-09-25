import XCTest
@testable import Kliima

/// Les mêmes cas que `core/src/position.test.ts`. Les valeurs attendues ont été
/// relevées sur l'implémentation TypeScript : c'est ce qui garantit qu'une
/// position donne la même cellule des deux côtés.
final class PositionTests: XCTestCase {

    // MARK: Quand demander la position

    func testLocatesOnlyWhenNothingWasChosen() {
        XCTAssertTrue(Position.locatesOnStart(.defaut))
        XCTAssertFalse(Position.locatesOnStart(.memoire))
        XCTAssertFalse(Position.locatesOnStart(.adresse))
    }

    // MARK: La parcelle d'une position

    func testParcelleCarriesTheNameTheInterfaceGives() {
        let parcelle = Position.parcelle(named: "Ma position", latitude: 48.4468, longitude: 1.4892)
        XCTAssertEqual(parcelle.name, "Ma position")
    }

    /// Le point de la règle : la parcelle voyage jusqu'au widget et jusqu'à
    /// l'adresse partagée, elle n'a pas à porter la position exacte.
    func testParcelleIsRoundedToTheCell() {
        let parcelle = Position.parcelle(
            named: "x", latitude: 48.44681234, longitude: 1.48923456
        )
        XCTAssertEqual(parcelle.latitude, 48.44, accuracy: 1e-9)
        XCTAssertEqual(parcelle.longitude, 1.48, accuracy: 1e-9)
    }

    func testSnapMatchesTheSharedCore() {
        // Relevé sur l'implémentation TypeScript, un point par hémisphère.
        let cas: [(Double, Double, Double, Double)] = [
            (48.44681234, 1.48923456, 48.44, 1.48),
            (-33.8688, 151.2093, -33.86, 151.2),
            (0.0001, -0.0001, 0, 0),
            (64.1466, -21.9426, 64.14, -21.94),
        ]
        for (latitude, longitude, attenduLat, attenduLon) in cas {
            let parcelle = Position.parcelle(named: "x", latitude: latitude, longitude: longitude)
            XCTAssertEqual(parcelle.latitude, attenduLat, accuracy: 1e-9,
                           "latitude de \(latitude)")
            XCTAssertEqual(parcelle.longitude, attenduLon, accuracy: 1e-9,
                           "longitude de \(longitude)")
        }
    }

    func testNeverMovesMoreThanHalfACell() {
        for point in [48.4468, -33.8688, 0.0001, 64.1466, -21.9426, 151.2093] {
            XCTAssertLessThanOrEqual(
                abs(Position.snap(point) - point), Position.cellDegrees / 2 + 1e-9,
                "\(point) déplacé de plus d'une demi-maille"
            )
        }
    }

    func testTwoPositionsInTheSameCellGiveTheSameParcelle() {
        let a = Position.parcelle(named: "x", latitude: 48.4468, longitude: 1.4892)
        let b = Position.parcelle(named: "x", latitude: 48.4490, longitude: 1.4870)
        XCTAssertEqual(a.latitude, b.latitude, accuracy: 1e-9)
        XCTAssertEqual(a.longitude, b.longitude, accuracy: 1e-9)
    }
}
