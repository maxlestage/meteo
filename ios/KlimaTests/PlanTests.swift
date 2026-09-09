import XCTest
@testable import Klima

/// Les mêmes cas que `core/src/plan.test.ts`. Quand un test change d'un côté,
/// il change de l'autre.
final class PlanTests: XCTestCase {
    func testLibreDonneLaJourneeEntiere() {
        // La règle du modèle : on facture l'échelle et l'anticipation, jamais
        // la réponse du jour. Sept jours de prévision des deux côtés.
        XCTAssertEqual(Plan.libre.limits.jours, Plan.pro.limits.jours)
        XCTAssertEqual(Plan.libre.limits.parcelles, 1)
    }

    func testLibreNOuvreAucuneFonctionPayante() {
        for feature in Feature.allCases {
            XCTAssertFalse(Plan.libre.allows(feature), "\(feature) devrait être fermée")
        }
    }

    func testProLesOuvreToutes() {
        for feature in Feature.allCases {
            XCTAssertTrue(Plan.pro.allows(feature), "\(feature) devrait être ouverte")
        }
    }

    func testDeuxiemeParcelleEstLaLimiteDuLibre() {
        XCTAssertTrue(Plan.libre.canAddParcelle(current: 0))
        XCTAssertFalse(Plan.libre.canAddParcelle(current: 1))
    }

    func testProNAPasDePlafond() {
        XCTAssertTrue(Plan.pro.canAddParcelle(current: 0))
        XCTAssertTrue(Plan.pro.canAddParcelle(current: 500))
        XCTAssertNil(Plan.pro.limits.parcelles)
    }

    func testResiliationVerrouilleSansPerdre() {
        let parcelles = ["Chartres", "Reims", "Toulouse"]
        let acces = Plan.libre.partition(parcelles)

        XCTAssertEqual(acces.readable, ["Chartres"])
        XCTAssertEqual(acces.locked, ["Reims", "Toulouse"])
        // Rien n'a disparu : le réabonnement les rend telles quelles.
        XCTAssertEqual(acces.readable + acces.locked, parcelles)
    }

    func testAuRetourToutRedevientLisible() {
        let parcelles = ["Chartres", "Reims", "Toulouse"]
        let acces = Plan.pro.partition(parcelles)
        XCTAssertEqual(acces.readable, parcelles)
        XCTAssertTrue(acces.locked.isEmpty)
    }

    func testMotifDeBlocageEstUneCle() {
        for feature in Feature.allCases {
            let key = feature.upgradeReasonKey
            XCTAssertEqual(key, "plan.reason.\(feature.rawValue)")
            XCTAssertFalse(key.contains(" "))
        }
    }

    func testChaquePalierAsesLimites() {
        for plan in Plan.allCases {
            XCTAssertGreaterThan(plan.limits.jours, 0)
            if let parcelles = plan.limits.parcelles {
                XCTAssertGreaterThan(parcelles, 0)
            }
        }
    }
}
