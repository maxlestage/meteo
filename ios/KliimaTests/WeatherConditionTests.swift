import XCTest
@testable import Kliima

/// Les mêmes cas que la suite du site web (`web/src/domain/weather.test.ts`).
final class WeatherConditionTests: XCTestCase {

    func testClearSky() {
        let condition = WeatherCondition.forCode(0)
        XCTAssertEqual(condition.labelKey, "wmo.clearSky")
        XCTAssertEqual(condition.icon, .clear)
    }

    func testDrizzle() {
        XCTAssertEqual(WeatherCondition.forCode(53).labelKey, "wmo.drizzle")
        XCTAssertEqual(WeatherCondition.forCode(53).icon, .drizzle)
    }

    func testRainIntensitiesShareOneIcon() {
        XCTAssertEqual([61, 63, 65].map { WeatherCondition.forCode($0).icon }, [.rain, .rain, .rain])
    }

    func testThunderWithHail() {
        XCTAssertEqual(WeatherCondition.forCode(96).icon, .thunder)
    }

    func testUnknownCodeFallsBack() {
        XCTAssertEqual(
            WeatherCondition.forCode(42),
            WeatherCondition(labelKey: "wmo.overcast", icon: .cloudy)
        )
    }

    /// Mêmes cas que `ciel_couvert.rs` (klima-web) : le fond est bleu pour
    /// un ciel dégagé ou peu nuageux, gris dès qu'il est couvert.
    func testUnCielCouvertEstGris() {
        XCTAssertEqual([0, 1, 2].map { WeatherCondition.forCode($0).icon.cielBleu }, [true, true, true])
        // Couvert, brouillard, pluie, neige, orage : gris.
        XCTAssertEqual([3, 45, 61, 71, 95].map { WeatherCondition.forCode($0).icon.cielBleu },
                       [false, false, false, false, false])
        // Un code inconnu retombe sur « couvert », gris lui aussi.
        XCTAssertFalse(WeatherCondition.forCode(4).icon.cielBleu)
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

    func testLabelsFollowDayAndNight() {
        // Un ciel clair, la nuit, est une nuit claire.
        XCTAssertEqual(WeatherCondition.forCode(0).label(isDay: false), Localized.text("wmo.clearNight"))
        XCTAssertEqual(WeatherCondition.forCode(1).label(isDay: false), Localized.text("wmo.mainlyClearNight"))
        XCTAssertEqual(WeatherCondition.forCode(0).label(isDay: true), WeatherCondition.forCode(0).label)
        // La pluie ne dépend pas de l'heure.
        XCTAssertEqual(WeatherCondition.forCode(63).label(isDay: false), WeatherCondition.forCode(63).label)
    }
}
