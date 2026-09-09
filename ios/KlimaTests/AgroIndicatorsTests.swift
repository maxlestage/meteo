import XCTest
@testable import Klima

/// Les mêmes cas que la suite du site web (`web/src/domain/agro.test.ts`),
/// pour garantir que les deux plateformes donnent le même conseil.
final class AgroIndicatorsTests: XCTestCase {

    private let reference = Date(timeIntervalSince1970: 1_778_558_400) // 12 mai 2026, 06 h UTC

    private func hour(
        index: Int = 0,
        temperature: Double = 18,
        relativeHumidity: Double = 65,
        dewPoint: Double = 11,
        precipitation: Double = 0,
        windSpeed: Double = 8,
        windGusts: Double = 14,
        soilTemperature: Double = 15,
        soilMoisture: Double = 0.24,
        vpd: Double = 0.7
    ) -> HourlySample {
        HourlySample(
            time: reference.addingTimeInterval(Double(index) * 3600),
            weatherCode: 3,
            isDay: true,
            precipitationProbability: 20,
            temperature: temperature,
            relativeHumidity: relativeHumidity,
            dewPoint: dewPoint,
            precipitation: precipitation,
            windSpeed: windSpeed,
            windGusts: windGusts,
            soilTemperature6cm: soilTemperature,
            soilMoisture3to9cm: soilMoisture,
            et0: 0.2,
            vapourPressureDeficit: vpd
        )
    }

    private func day(
        temperatureMin: Double = 10,
        temperatureMax: Double = 22,
        precipitationSum: Double = 2,
        et0Sum: Double = 3.5
    ) -> DailySample {
        DailySample(
            date: reference,
            weatherCode: 3,
            temperatureMin: temperatureMin,
            temperatureMax: temperatureMax,
            precipitationSum: precipitationSum,
            precipitationProbabilityMax: 30,
            et0Sum: et0Sum,
            windGustsMax: 25,
            sunrise: reference.addingTimeInterval(-3600),
            sunset: reference.addingTimeInterval(12 * 3600)
        )
    }

    // MARK: Degrés-jours

    func testGddAboveBase() {
        XCTAssertEqual(AgroIndicators.growingDegreeDays(temperatureMin: 10, temperatureMax: 22), 6)
    }

    func testColdDayCapitalizesNothing() {
        XCTAssertEqual(AgroIndicators.growingDegreeDays(temperatureMin: 2, temperatureMax: 8), 0)
    }

    func testGddCeiling() {
        XCTAssertEqual(AgroIndicators.growingDegreeDays(temperatureMin: 30, temperatureMax: 42), 20)
    }

    func testCumulativeGdd() {
        let days = [day(), day(temperatureMin: 14, temperatureMax: 26)]
        XCTAssertEqual(AgroIndicators.cumulativeGdd(days), 16)
    }

    // MARK: Bilan hydrique

    func testDeficitTriggersIrrigationAdvice() {
        let days = Array(repeating: day(precipitationSum: 0, et0Sum: 4), count: 7)
        let result = AgroIndicators.waterBalance(days)
        XCTAssertEqual(result.evapotranspiration, 28)
        XCTAssertEqual(result.balance, -28)
        XCTAssertEqual(result.status, .deficit)
        XCTAssertEqual(result.irrigationAdvice, 28)
    }

    func testSurplusNeedsNoIrrigation() {
        let result = AgroIndicators.waterBalance([day(precipitationSum: 30, et0Sum: 3)])
        XCTAssertEqual(result.status, .excedent)
        XCTAssertEqual(result.irrigationAdvice, 0)
    }

    func testBalancedWeek() {
        XCTAssertEqual(AgroIndicators.waterBalance([day(precipitationSum: 4, et0Sum: 3.5)]).status, .equilibre)
    }

    // MARK: Pulvérisation

    func testIdealSprayHour() throws {
        let window = try XCTUnwrap(AgroIndicators.evaluateSprayHour([hour()], at: 0))
        XCTAssertEqual(window.score, 100)
        XCTAssertEqual(window.verdict, .favorable)
        XCTAssertTrue(window.blockers.isEmpty)
    }

    func testWindAboveLegalLimitIsDisqualifying() throws {
        let window = try XCTUnwrap(AgroIndicators.evaluateSprayHour([hour(windSpeed: 24)], at: 0))
        XCTAssertEqual(window.verdict, .defavorable)
        XCTAssertEqual(window.blockers.first, .windTooStrong(wind: 24, limit: 19))
    }

    func testRainInTheNextHourIsDisqualifying() throws {
        let hours = [hour(), hour(index: 1, precipitation: 1.4)]
        let window = try XCTUnwrap(AgroIndicators.evaluateSprayHour(hours, at: 0))
        XCTAssertEqual(window.verdict, .defavorable)
        XCTAssertTrue(window.blockers.contains(.rain(1.4)))
    }

    func testStillAirWarnsAboutThermalInversion() throws {
        let window = try XCTUnwrap(AgroIndicators.evaluateSprayHour([hour(windSpeed: 1)], at: 0))
        XCTAssertEqual(window.verdict, .acceptable)
        XCTAssertTrue(window.blockers.contains(.windTooWeak))
    }

    func testScoreNeverGoesNegative() throws {
        let sample = hour(temperature: 32, precipitation: 5, windSpeed: 40, windGusts: 60)
        let window = try XCTUnwrap(AgroIndicators.evaluateSprayHour([sample], at: 0))
        XCTAssertEqual(window.score, 0)
    }

    func testOutOfRangeIndexReturnsNil() {
        XCTAssertNil(AgroIndicators.evaluateSprayHour([hour()], at: 5))
    }

    func testLastHourOfSeriesIsStillEvaluated() {
        XCTAssertEqual(AgroIndicators.sprayWindows([hour(), hour(index: 1)]).count, 2)
    }

    // MARK: Prochaine fenêtre

    func testFirstTwoHourRunIsSelected() throws {
        let hours = [hour(windSpeed: 30), hour(index: 1), hour(index: 2), hour(index: 3)]
        let opportunity = try XCTUnwrap(AgroIndicators.nextSprayOpportunity(AgroIndicators.sprayWindows(hours)))
        XCTAssertEqual(opportunity.start, reference.addingTimeInterval(3600))
        XCTAssertEqual(opportunity.end, reference.addingTimeInterval(3 * 3600))
        XCTAssertEqual(opportunity.score, 100)
    }

    func testNoWindowWhenEverythingIsUnfavourable() {
        let hours = [hour(windSpeed: 45), hour(index: 1, windSpeed: 45)]
        XCTAssertNil(AgroIndicators.nextSprayOpportunity(AgroIndicators.sprayWindows(hours)))
    }

    // MARK: Gel

    func testMildNight() {
        XCTAssertEqual(AgroIndicators.frostRisk(minTemperature: 6, minDewPoint: 3).severity, .aucun)
    }

    func testHoarFrost() {
        let risk = AgroIndicators.frostRisk(minTemperature: -0.5, minDewPoint: -2)
        XCTAssertEqual(risk.severity, .faible)
        XCTAssertTrue(risk.hoarFrost)
    }

    func testSevereFrost() {
        XCTAssertEqual(AgroIndicators.frostRisk(minTemperature: -6, minDewPoint: -8).severity, .severe)
    }

    // MARK: Maladies

    func testHighDiseasePressure() {
        let hours = (0..<14).map { hour(index: $0, temperature: 16, relativeHumidity: 95) }
        let pressure = AgroIndicators.diseasePressure(hours)
        XCTAssertEqual(pressure.leafWetnessHours, 14)
        XCTAssertEqual(pressure.level, .elevee)
    }

    func testHumidButTooColdCountsAsNoWetness() {
        let hours = (0..<14).map { hour(index: $0, temperature: 3, relativeHumidity: 97) }
        XCTAssertEqual(AgroIndicators.diseasePressure(hours).level, .faible)
    }

    // MARK: Sol

    func testSaturatedSoilBlocksTraffic() {
        let soil = AgroIndicators.soilCondition([hour(soilMoisture: 0.41)])
        XCTAssertEqual(soil.state, .sature)
        XCTAssertFalse(soil.trafficable)
        XCTAssertFalse(soil.sowable)
    }

    func testDrainedWarmSoilAllowsSowing() {
        let soil = AgroIndicators.soilCondition([hour(soilTemperature: 12, soilMoisture: 0.22)])
        XCTAssertEqual(soil.state, .ressuye)
        XCTAssertTrue(soil.sowable)
    }

    func testColdSoilPreventsSowing() {
        XCTAssertFalse(AgroIndicators.soilCondition([hour(soilTemperature: 5)]).sowable)
    }

    func testEmptySeries() {
        XCTAssertFalse(AgroIndicators.soilCondition([]).trafficable)
    }

    // MARK: Synthèse

    func testSummaryAssemblesEveryIndicator() {
        let hours = (0..<24).map { hour(index: $0) }
        let days = Array(repeating: day(), count: 7)
        let summary = AgroIndicators.summarize(hours: hours, days: days)
        XCTAssertEqual(summary.gdd, 42)
        XCTAssertEqual(summary.soil.state, .ressuye)
        XCTAssertNotNil(summary.nextSpray)
        XCTAssertEqual(summary.frost.severity, .aucun)
    }
}
