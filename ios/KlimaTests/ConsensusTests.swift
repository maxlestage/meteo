import XCTest
@testable import Klima

/// Les mêmes cas que la suite du web (`core/src/consensus.test.ts`) : les deux
/// plateformes doivent juger l'accord entre modèles de la même façon.
final class ConsensusTests: XCTestCase {

    private func reading(
        _ index: Int,
        _ temperature: Double,
        precipitation: Double = 0,
        windSpeed: Double = 10
    ) -> SourceReading {
        SourceReading(
            source: WeatherProviders.openMeteoSources[index],
            temperature: temperature,
            precipitation: precipitation,
            windSpeed: windSpeed
        )
    }

    func testASingleModelIsNoConsensus() {
        XCTAssertNil(ModelConsensus.consensus([reading(0, 18)]))
        XCTAssertNil(ModelConsensus.consensus([]))
    }

    func testMedianIgnoresAnOutlier() throws {
        let result = try XCTUnwrap(ModelConsensus.consensus([
            reading(0, 18), reading(1, 18.5), reading(2, 19), reading(3, 25),
        ]))
        // (18,5 + 19) / 2 = 18,75, arrondi à la décimale comme le reste du domaine.
        XCTAssertEqual(result.temperature.median, 18.8)
        XCTAssertEqual(result.temperature.min, 18)
        XCTAssertEqual(result.temperature.max, 25)
        XCTAssertEqual(result.temperature.spread, 7)
    }

    func testMedianOfAnOddCount() throws {
        let result = try XCTUnwrap(ModelConsensus.consensus([
            reading(0, 12), reading(1, 18), reading(2, 15),
        ]))
        XCTAssertEqual(result.temperature.median, 15)
    }

    func testTightModelsAgreeStrongly() throws {
        let result = try XCTUnwrap(ModelConsensus.consensus([
            reading(0, 18), reading(1, 18.4), reading(2, 19),
        ]))
        XCTAssertEqual(result.temperature.spread, 1)
        XCTAssertTrue(result.agreeOnRain)
        XCTAssertEqual(result.agreement, .forte)
    }

    func testTwoDegreesApartIsModerate() throws {
        let result = try XCTUnwrap(ModelConsensus.consensus([reading(0, 17), reading(1, 19)]))
        XCTAssertEqual(result.agreement, .moyenne)
    }

    func testMoreThanThreeDegreesIsWeak() throws {
        let result = try XCTUnwrap(ModelConsensus.consensus([reading(0, 15), reading(1, 19.5)]))
        XCTAssertEqual(result.agreement, .faible)
    }

    func testRainDisagreementCapsTheAgreement() throws {
        let result = try XCTUnwrap(ModelConsensus.consensus([
            reading(0, 18, precipitation: 0),
            reading(1, 18.2, precipitation: 1.4),
        ]))
        XCTAssertLessThan(result.temperature.spread, 1.5)
        XCTAssertFalse(result.agreeOnRain)
        XCTAssertEqual(result.agreement, .moyenne)
    }

    func testATraceOfRainIsNotADisagreement() throws {
        let result = try XCTUnwrap(ModelConsensus.consensus([
            reading(0, 18, precipitation: 0),
            reading(1, 18.2, precipitation: 0.05),
        ]))
        XCTAssertTrue(result.agreeOnRain)
        XCTAssertEqual(result.agreement, .forte)
    }

    func testFourIndependentModelsAtOpenMeteo() {
        let sources = WeatherProviders.openMeteoSources
        XCTAssertEqual(sources.count, 4)
        XCTAssertEqual(Set(sources.map(\.institution)).count, 4)
        XCTAssertEqual(Set(sources.map(\.id)).count, 4)
        XCTAssertEqual(WeatherProviders.source("meteofrance_seamless")?.institution, "Météo-France")
    }

}
