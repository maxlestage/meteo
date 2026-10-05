import XCTest
@testable import Kliima

/// Le widget d'écran d'accueil et la complication lisent la ville que
/// l'application a enregistrée : ce trajet doit tenir.
final class SharedStoreTests: XCTestCase {

    func testParcelleSurvivesTheSharedStore() throws {
        let parcelle = Parcelle(
            name: "Reims",
            latitude: 49.2583,
            longitude: 4.0317,
            admin: "Grand Est",
            country: "France"
        )
        SharedStore.save(parcelle)
        XCTAssertEqual(SharedStore.loadParcelle(), parcelle)
    }

    /// Une ville sans région ni pays se relit telle quelle.
    func testMinimalParcelleSurvives() throws {
        let parcelle = Parcelle(name: "Ma ville", latitude: 43.6, longitude: 1.44)
        SharedStore.save(parcelle)
        let reloaded = try XCTUnwrap(SharedStore.loadParcelle())
        XCTAssertEqual(reloaded, parcelle)
        XCTAssertNil(reloaded.admin)
    }
}
