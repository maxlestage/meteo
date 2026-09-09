import XCTest
@testable import Klima

/// Les mêmes cas que la suite du site web (`web/src/domain/weather.test.ts`).
final class WeatherConditionTests: XCTestCase {

    func testClearSky() {
        let condition = WeatherCondition.forCode(0)
        XCTAssertEqual(condition.label, "Ciel dégagé")
        XCTAssertEqual(condition.icon, .clear)
    }

    func testDrizzle() {
        XCTAssertEqual(WeatherCondition.forCode(53).label, "Bruine")
        XCTAssertEqual(WeatherCondition.forCode(53).icon, .drizzle)
    }

    func testRainIntensitiesShareOneIcon() {
        XCTAssertEqual([61, 63, 65].map { WeatherCondition.forCode($0).icon }, [.rain, .rain, .rain])
    }

    func testThunderWithHail() {
        XCTAssertEqual(WeatherCondition.forCode(96).icon, .thunder)
    }

    func testUnknownCodeFallsBack() {
        XCTAssertEqual(WeatherCondition.forCode(42), WeatherCondition(label: "Couvert", icon: .cloudy))
    }

    func testSymbolsFollowDayAndNight() {
        XCTAssertEqual(WeatherCondition.Icon.clear.symbolName(isDay: true), "sun.max.fill")
        XCTAssertEqual(WeatherCondition.Icon.clear.symbolName(isDay: false), "moon.stars.fill")
        // Un temps couvert ne dépend pas de l'heure.
        XCTAssertEqual(
            WeatherCondition.Icon.cloudy.symbolName(isDay: true),
            WeatherCondition.Icon.cloudy.symbolName(isDay: false)
        )
    }
}
