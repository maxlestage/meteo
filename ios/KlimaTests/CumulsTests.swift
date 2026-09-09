import XCTest
@testable import Klima

/// Les mêmes cas que `core/src/cumuls.test.ts`.
final class CumulsTests: XCTestCase {
    private let calendar: Calendar = {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: "Europe/Paris")!
        return calendar
    }()

    private func date(_ iso: String) -> Date {
        let formatter = DateFormatter()
        formatter.calendar = calendar
        formatter.timeZone = calendar.timeZone
        formatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss"
        return formatter.date(from: iso)!
    }

    private func day(
        _ iso: String,
        precipitationSum: Double = 2,
        et0Sum: Double = 3
    ) -> DailySample {
        DailySample(
            date: date("\(iso)T00:00:00"),
            weatherCode: 3,
            temperatureMin: 8,
            temperatureMax: 20,
            precipitationSum: precipitationSum,
            precipitationProbabilityMax: 40,
            et0Sum: et0Sum,
            windGustsMax: 30,
            sunrise: date("\(iso)T06:00:00"),
            sunset: date("\(iso)T21:00:00")
        )
    }

    private var series: [DailySample] {
        [
            day("2026-04-10"),
            day("2026-04-11", precipitationSum: 5, et0Sum: 2),
            day("2026-04-12", precipitationSum: 0, et0Sum: 4),
        ]
    }

    private func accumulate(from iso: String, base: Double = AgroThresholds.gddBase) -> Cumul? {
        Cumuls.accumulate(series, from: date(iso), base: base, calendar: calendar)
    }

    func testAdditionnePluieEt0EtBilan() {
        let cumul = accumulate(from: "2026-04-10T00:00:00")!
        XCTAssertEqual(cumul.days, 3)
        XCTAssertEqual(cumul.precipitation, 7)
        XCTAssertEqual(cumul.evapotranspiration, 9)
        XCTAssertEqual(cumul.balance, -2)
    }

    func testCapitaliseLesDegresJours() {
        // (8 + 20) / 2 = 14 ; base 10 → 4 degrés-jours par journée, trois journées.
        XCTAssertEqual(accumulate(from: "2026-04-10T00:00:00")!.gdd, 12)
    }

    func testDateEnCoursDeSerieNeCompteQueLaSuite() {
        let cumul = accumulate(from: "2026-04-11T00:00:00")!
        XCTAssertEqual(cumul.days, 2)
        XCTAssertEqual(cumul.precipitation, 5)
    }

    func testHeureDeLaDemandeNExclutPasSaJournee() {
        // Un semis noté à 14 h reste un semis du 11.
        let cumul = accumulate(from: "2026-04-11T14:30:00")!
        XCTAssertEqual(cumul.days, 2)
        XCTAssertEqual(cumul.from, date("2026-04-11T00:00:00"))
    }

    func testDemandeAnterieureSeDitIncomplete() {
        let cumul = accumulate(from: "2026-04-05T00:00:00")!
        XCTAssertEqual(cumul.days, 3)
        XCTAssertEqual(cumul.missingDays, 5)
        XCTAssertFalse(cumul.isComplete)
        XCTAssertEqual(cumul.requestedFrom, date("2026-04-05T00:00:00"))
        XCTAssertEqual(cumul.from, date("2026-04-10T00:00:00"))
    }

    func testCouvertureCompleteLeDitAussi() {
        XCTAssertTrue(accumulate(from: "2026-04-10T00:00:00")!.isComplete)
    }

    func testDatePosterieureNeRenvoieRien() {
        // Zéro millimètre et « pas de données » ne veulent pas dire la même chose.
        XCTAssertNil(accumulate(from: "2026-05-01T00:00:00"))
    }

    func testSerieVideNeRenvoieRien() {
        XCTAssertNil(Cumuls.accumulate([], from: date("2026-04-10T00:00:00"), calendar: calendar))
    }

    func testBornesEffectivesEncadrentCeQuiAEteCompte() {
        let cumul = accumulate(from: "2026-04-10T00:00:00")!
        XCTAssertEqual(cumul.from, date("2026-04-10T00:00:00"))
        XCTAssertEqual(cumul.to, date("2026-04-12T00:00:00"))
    }

    func testBaseDifferenteChangeLeResultat() {
        // Base 6 : (8 + 20) / 2 = 14 → 8 par journée.
        XCTAssertEqual(accumulate(from: "2026-04-10T00:00:00", base: 6)!.gdd, 24)
    }
}
