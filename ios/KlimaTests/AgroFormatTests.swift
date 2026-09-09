import XCTest
@testable import Klima

/// Les heures affichées sont celles de la parcelle, pas celles du téléphone ;
/// les nombres suivent la langue de l'appareil.
final class AgroFormatTests: XCTestCase {

    /// 12 mai 2026, 20 h 00 UTC — soit 22 h à Paris et 13 h à Los Angeles.
    private let instant = Date(timeIntervalSince1970: 1_778_616_000)
    private let paris = TimeZone(identifier: "Europe/Paris")!
    private let losAngeles = TimeZone(identifier: "America/Los_Angeles")!
    private let french = Locale(identifier: "fr_FR")
    private let english = Locale(identifier: "en_US")

    func testHourUsesTheParcelleTimeZone() {
        XCTAssertEqual(AgroFormat.hour(instant, in: paris, locale: french), "22 h")
        XCTAssertEqual(AgroFormat.hour(instant, in: losAngeles, locale: french), "13 h")
    }

    /// Le gabarit laisse la locale choisir l'horloge : 24 h en français,
    /// 12 h en anglais américain.
    func testHourFollowsTheLanguageClock() {
        let american = AgroFormat.hour(instant, in: paris, locale: english)
        XCTAssertTrue(american.contains("10"), american)
        XCTAssertTrue(american.uppercased().contains("PM"), american)
    }

    func testTimeFallsBackWhenMissing() {
        XCTAssertEqual(AgroFormat.time(instant, in: paris, locale: french), "22:00")
        XCTAssertEqual(AgroFormat.time(nil, in: paris), "—")
    }

    func testWeekdayIsCapitalised() {
        XCTAssertEqual(AgroFormat.weekday(instant, in: paris, locale: french), "Mar.")
        XCTAssertEqual(AgroFormat.weekdayHour(instant, in: paris, locale: french), "Mar. 22 h")
    }

    func testNumbersFollowTheLanguage() {
        XCTAssertEqual(AgroFormat.unit(2.1, "mm", locale: french), "2,1\u{00a0}mm")
        XCTAssertEqual(AgroFormat.unit(2.1, "mm", locale: english), "2.1\u{00a0}mm")
        XCTAssertEqual(AgroFormat.decimal(4.8, locale: Locale(identifier: "es_ES")), "4,8")
    }

    func testBalanceCarriesItsSign() {
        XCTAssertEqual(AgroFormat.signedUnit(2.7, "mm", locale: french), "+2,7\u{00a0}mm")
        // Un bilan nul n'est pas « positif » : pas de signe.
        XCTAssertFalse(AgroFormat.signedUnit(0, "mm", locale: french).hasPrefix("+"))
    }

    func testTemperaturesAreRounded() {
        XCTAssertEqual(AgroFormat.temperature(16.4), "16°")
        XCTAssertEqual(AgroFormat.temperature(-1.4), "-1°")
    }
}
