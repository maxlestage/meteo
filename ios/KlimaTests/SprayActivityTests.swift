import XCTest
@testable import Klima

/// L'activité en direct transporte son état encodé par le système. Un encodage
/// qui casse ne se voit pas à la compilation : l'activité échoue en silence.
final class SprayActivityTests: XCTestCase {

    private let reference = Date(timeIntervalSince1970: 1_778_616_000)

    private func roundTrip(_ state: SprayActivityAttributes.ContentState) throws
        -> SprayActivityAttributes.ContentState {
        let data = try JSONEncoder().encode(state)
        return try JSONDecoder().decode(SprayActivityAttributes.ContentState.self, from: data)
    }

    func testStateSurvivesEncoding() throws {
        let state = SprayActivityAttributes.ContentState(
            verdict: .acceptable,
            score: 72,
            windSpeed: 12.4,
            windGusts: 24.1,
            blocker: .windTooStrong(wind: 24, limit: 19),
            updatedAt: reference
        )
        XCTAssertEqual(try roundTrip(state), state)
    }

    /// Chaque motif de blocage doit survivre au trajet, valeurs comprises.
    func testEveryBlockerKindSurvivesEncoding() throws {
        let blockers: [SprayBlocker] = [
            .windTooStrong(wind: 24, limit: 19),
            .windTooWeak,
            .gusts(31),
            .rain(1.4),
            .tooHot(28),
            .tooCold(3),
            .dryAir(35),
            .vapourPressureDeficit(1.6),
        ]

        for blocker in blockers {
            let state = SprayActivityAttributes.ContentState(
                verdict: .defavorable,
                score: 20,
                windSpeed: 8,
                windGusts: 14,
                blocker: blocker,
                updatedAt: reference
            )
            XCTAssertEqual(try roundTrip(state).blocker, blocker, "\(blocker)")
        }
    }

    func testStateWithoutBlocker() throws {
        let state = SprayActivityAttributes.ContentState(
            verdict: .favorable,
            score: 100,
            windSpeed: 8,
            windGusts: 14,
            blocker: nil,
            updatedAt: reference
        )
        XCTAssertNil(try roundTrip(state).blocker)
        XCTAssertFalse(state.isBlocked)
    }

    func testBlockedStateIsFlagged() {
        let state = SprayActivityAttributes.ContentState(
            verdict: .defavorable, score: 20, windSpeed: 40, windGusts: 60,
            blocker: .gusts(60), updatedAt: reference
        )
        XCTAssertTrue(state.isBlocked)
    }

    /// L'activité affiche les heures de la parcelle, pas celles du téléphone.
    func testAttributesResolveTheParcelleTimeZone() {
        let attributes = SprayActivityAttributes(
            parcelleName: "Chartres",
            windowStart: reference,
            windowEnd: reference.addingTimeInterval(7200),
            timeZoneIdentifier: "Europe/Paris"
        )
        XCTAssertEqual(attributes.timeZone.identifier, "Europe/Paris")
        XCTAssertEqual(AgroFormat.hour(attributes.windowStart, in: attributes.timeZone,
                                       locale: Locale(identifier: "fr_FR")), "22 h")
    }

    /// Un fuseau inconnu ne doit pas faire perdre l'affichage.
    func testUnknownTimeZoneFallsBack() {
        let attributes = SprayActivityAttributes(
            parcelleName: "Chartres", windowStart: reference, windowEnd: reference,
            timeZoneIdentifier: "Mars/Olympus"
        )
        XCTAssertEqual(attributes.timeZone, .current)
    }
}

/// Le widget d'écran d'accueil et la complication lisent la parcelle que
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

    /// Une parcelle sans région ni pays se relit telle quelle.
    func testMinimalParcelleSurvives() throws {
        let parcelle = Parcelle(name: "Ma parcelle", latitude: 43.6, longitude: 1.44)
        SharedStore.save(parcelle)
        let reloaded = try XCTUnwrap(SharedStore.loadParcelle())
        XCTAssertEqual(reloaded, parcelle)
        XCTAssertNil(reloaded.admin)
    }
}
