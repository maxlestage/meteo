import XCTest
@testable import Kliima

/// Les mêmes cas que `rust/klima-core/src/consensus.rs` : les deux
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

    /// Miroir de `une_moitie_negative_s_arrondit_vers_le_haut`.
    func testUneMoitieNegativeSArronditVersLeHaut() throws {
        let gel = try XCTUnwrap(ModelConsensus.consensus([reading(0, -2), reading(1, -2.5)]))
        XCTAssertEqual(gel.temperature.median, -2.2)
        XCTAssertEqual(gel.temperature.min, -2.5)
        XCTAssertEqual(gel.temperature.max, -2)
        XCTAssertEqual(gel.temperature.spread, 0.5)
        let doux = try XCTUnwrap(ModelConsensus.consensus([reading(0, 2), reading(1, 2.5)]))
        XCTAssertEqual(doux.temperature.median, 2.3)
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

    func testEveryoneAgreesOnRain() throws {
        let result = try XCTUnwrap(ModelConsensus.consensus([
            reading(0, 14, precipitation: 2.1),
            reading(1, 14.5, precipitation: 1.8),
            reading(2, 14.2, precipitation: 3),
        ]))
        XCTAssertTrue(result.agreeOnRain)
        XCTAssertEqual(result.agreement, .forte)
    }

    func testWindIsCrossCheckedLikeTheRest() throws {
        let result = try XCTUnwrap(ModelConsensus.consensus([
            reading(0, 18, windSpeed: 12),
            reading(1, 18, windSpeed: 24),
        ]))
        XCTAssertEqual(result.windSpeed.median, 18)
        XCTAssertEqual(result.windSpeed.spread, 12)
    }

    /// Mêmes cas que `les_seuils_de_l_accord_sont_inclusifs` côté Rust.
    func testAgreementThresholdsAreInclusive() throws {
        let strong = try XCTUnwrap(ModelConsensus.consensus([reading(0, 18), reading(1, 19.5)]))
        XCTAssertEqual(strong.temperature.spread, 1.5)
        XCTAssertEqual(strong.agreement, .forte)
        let moderate = try XCTUnwrap(ModelConsensus.consensus([reading(0, 18), reading(1, 21)]))
        XCTAssertEqual(moderate.temperature.spread, 3)
        XCTAssertEqual(moderate.agreement, .moyenne)
        // 0,1 mm pile, c'est de la pluie : face à un modèle sec, désaccord.
        let rain = try XCTUnwrap(ModelConsensus.consensus([
            reading(0, 18, precipitation: 0),
            reading(1, 18, precipitation: 0.1),
        ]))
        XCTAssertFalse(rain.agreeOnRain)
        XCTAssertEqual(rain.agreement, .moyenne)
    }

    func testProviderCountsAreKept() throws {
        let counted = try XCTUnwrap(ModelConsensus.consensus(
            [reading(0, 18), reading(1, 18.4), reading(2, 18.6)], answered: 2, queried: 3
        ))
        XCTAssertEqual(counted.readings.count, 3)
        XCTAssertEqual(counted.providersAnswered, 2)
        XCTAssertEqual(counted.providersQueried, 3)
        // Sans compte, les fournisseurs sont ceux des relevés : un seul ici.
        let uncounted = try XCTUnwrap(ModelConsensus.consensus([reading(0, 18), reading(1, 18.4)]))
        XCTAssertEqual(uncounted.providersAnswered, 1)
        XCTAssertEqual(uncounted.providersQueried, 1)
    }

    /// Sept modèles, tous d'instituts différents : les mêmes que
    /// `OPEN_METEO_SOURCES` côté Rust.
    func testSevenIndependentModelsAtOpenMeteo() {
        let sources = WeatherProviders.openMeteoSources
        XCTAssertEqual(sources.count, 7)
        XCTAssertEqual(Set(sources.map(\.institution)).count, 7)
        XCTAssertEqual(Set(sources.map(\.id)).count, 7)
        XCTAssertEqual(WeatherProviders.source("meteofrance_seamless")?.institution, "Météo-France")
    }

}
