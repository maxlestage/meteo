import XCTest
@testable import Klima

/// Les heures affichées sont celles de la parcelle, pas celles du téléphone.
final class AgroFormatTests: XCTestCase {

    /// 12 mai 2026, 20 h 00 UTC — soit 22 h à Paris et 13 h à Los Angeles.
    private let instant = Date(timeIntervalSince1970: 1_778_616_000)
    private let paris = TimeZone(identifier: "Europe/Paris")!
    private let losAngeles = TimeZone(identifier: "America/Los_Angeles")!

    func testHourUsesTheParcelleTimeZone() {
        XCTAssertEqual(AgroFormat.hour(instant, in: paris), "22 h")
        XCTAssertEqual(AgroFormat.hour(instant, in: losAngeles), "13 h")
    }

    /// Le suffixe « h » est déjà dans le format : rien à rajouter à l'affichage.
    func testHourCarriesItsOwnSuffix() {
        XCTAssertFalse(AgroFormat.hour(instant, in: paris).hasSuffix("h h"))
    }

    func testWeekdayIsCapitalised() {
        XCTAssertEqual(AgroFormat.weekday(instant, in: paris), "Mar.")
        XCTAssertEqual(AgroFormat.weekdayHour(instant, in: paris), "Mar. 22 h")
    }

    func testTimeFallsBackWhenMissing() {
        XCTAssertEqual(AgroFormat.time(instant, in: paris), "22:00")
        XCTAssertEqual(AgroFormat.time(nil, in: paris), "—")
    }
}
