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
    ) -> ModelReading {
        ModelReading(
            model: WeatherModel.all[index],
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

    func testFourIndependentServices() {
        XCTAssertEqual(WeatherModel.all.count, 4)
        XCTAssertEqual(Set(WeatherModel.all.map(\.institution)).count, 4)
        XCTAssertEqual(Set(WeatherModel.all.map(\.id)).count, 4)
        XCTAssertEqual(WeatherModel.named("meteofrance_seamless")?.institution, "Météo-France")
        XCTAssertNil(WeatherModel.named("inconnu"))
    }

    /// Décodage des colonnes suffixées par modèle, et rejet d'un modèle vide.
    func testDecodesSuffixedColumns() throws {
        let pad = { (n: Int) in String(format: "%02d", n) }
        let now = Date()
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = .current
        let stamps = (-1...1).map { offset -> String in
            let date = now.addingTimeInterval(Double(offset) * 3600)
            let c = calendar.dateComponents([.year, .month, .day, .hour], from: date)
            return "\(c.year!)-\(pad(c.month!))-\(pad(c.day!))T\(pad(c.hour!)):00"
        }

        let json = """
        {
          "timezone": "\(TimeZone.current.identifier)",
          "hourly": {
            "time": ["\(stamps[0])", "\(stamps[1])", "\(stamps[2])"],
            "temperature_2m_meteofrance_seamless": [10, 18.2, 19],
            "precipitation_meteofrance_seamless": [0, 0, 0],
            "wind_speed_10m_meteofrance_seamless": [8, 12, 14],
            "temperature_2m_ecmwf_ifs025": [10, 18.6, 19],
            "precipitation_ecmwf_ifs025": [0, 0, 0],
            "wind_speed_10m_ecmwf_ifs025": [8, 13, 14],
            "temperature_2m_icon_seamless": [null, null, null],
            "temperature_2m_gfs_seamless": [10, 18.4, 19],
            "precipitation_gfs_seamless": [0, 0, 0],
            "wind_speed_10m_gfs_seamless": [8, 12, 14]
          }
        }
        """.data(using: .utf8)!

        let payload = try JSONDecoder().decode(ModelPayload.self, from: json)
        let readings = AgroWeatherService.readings(from: payload)

        // ICON n'a que des valeurs nulles : il sort du recoupement.
        XCTAssertEqual(readings.count, 3)
        XCTAssertFalse(readings.contains { $0.model.id == "icon_seamless" })

        let result = try XCTUnwrap(ModelConsensus.consensus(readings))
        XCTAssertEqual(result.temperature.min, 18.2)
        XCTAssertEqual(result.temperature.max, 18.6)
        XCTAssertEqual(result.agreement, .forte)
    }
}
