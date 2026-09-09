import XCTest
@testable import Klima

/// Décodage de chaque fournisseur. Les mêmes cas que
/// `core/src/providers/providers.test.ts`, pour que les deux plateformes lisent
/// les mêmes réponses de la même façon.
final class ProvidersTests: XCTestCase {

    private func data(_ json: String) -> Data { json.data(using: .utf8)! }

    /// Trois heures autour de maintenant, au format d'Open-Meteo.
    private func hours() -> [String] {
        let pad = { (n: Int) in String(format: "%02d", n) }
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = .current
        return (-1...1).map { offset in
            let date = Date().addingTimeInterval(Double(offset) * 3600)
            let c = calendar.dateComponents([.year, .month, .day, .hour], from: date)
            return "\(c.year!)-\(pad(c.month!))-\(pad(c.day!))T\(pad(c.hour!)):00"
        }
    }

    // MARK: Open-Meteo

    func testOpenMeteoReadsSuffixedColumns() throws {
        let t = hours()
        let payload = try JSONDecoder().decode(ModelPayload.self, from: data("""
        {
          "timezone": "\(TimeZone.current.identifier)",
          "hourly": {
            "time": ["\(t[0])", "\(t[1])", "\(t[2])"],
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
        """))

        let readings = WeatherProviders.openMeteoReadings(from: payload)
        // ICON n'a que des valeurs nulles : il sort du recoupement.
        XCTAssertEqual(readings.count, 3)
        XCTAssertFalse(readings.contains { $0.source.id == "icon_seamless" })
        XCTAssertTrue(readings.allSatisfy { $0.source.provider == "open-meteo" })
        XCTAssertEqual(readings.first?.temperature, 18.2)
    }

    // MARK: MET Norway

    /// Les horodatages sont en ISO 8601 : sans stratégie de décodage, toute la
    /// réponse est perdue. Ce test garde cette porte fermée.
    func testMetNorwayDecodesIsoTimestamps() throws {
        let now = ISO8601DateFormatter().string(from: Date())
        let later = ISO8601DateFormatter().string(from: Date().addingTimeInterval(6 * 3600))

        let payload = try MetPayload.decode(data("""
        {
          "properties": {
            "timeseries": [
              {
                "time": "\(later)",
                "data": { "instant": { "details": { "air_temperature": 30, "wind_speed": 1 } } }
              },
              {
                "time": "\(now)",
                "data": {
                  "instant": { "details": { "air_temperature": 18.3, "wind_speed": 5 } },
                  "next_1_hours": { "details": { "precipitation_amount": 0.4 } }
                }
              }
            ]
          }
        }
        """))

        let readings = WeatherProviders.metNorwayReadings(from: payload)
        XCTAssertEqual(readings.count, 1)
        XCTAssertEqual(readings[0].temperature, 18.3)
        XCTAssertEqual(readings[0].precipitation, 0.4)
        // 5 m/s = 18 km/h
        XCTAssertEqual(readings[0].windSpeed, 18, accuracy: 0.0001)
        XCTAssertEqual(readings[0].source.institution, "MET Norway")
    }

    func testMetNorwayWithoutTemperature() throws {
        let payload = try MetPayload.decode(data("""
        {"properties": {"timeseries": []}}
        """))
        XCTAssertTrue(WeatherProviders.metNorwayReadings(from: payload).isEmpty)
    }

    // MARK: Bright Sky

    func testBrightSkyReadsTheStation() throws {
        let payload = try JSONDecoder().decode(BrightSkyPayload.self, from: data("""
        {"weather": {"temperature": 17.8, "precipitation": 0.2, "wind_speed": 14}}
        """))
        let readings = WeatherProviders.brightSkyReadings(from: payload)
        XCTAssertEqual(readings.count, 1)
        XCTAssertEqual(readings[0].temperature, 17.8)
        XCTAssertEqual(readings[0].source.name, "Observation DWD")
    }

    func testBrightSkyWithoutStation() throws {
        let payload = try JSONDecoder().decode(BrightSkyPayload.self, from: data("""
        {"weather": {"temperature": null}}
        """))
        XCTAssertTrue(WeatherProviders.brightSkyReadings(from: payload).isEmpty)
    }

    // MARK: Registre

    func testSourcesAreDistinctAndAttributed() {
        let sources = WeatherProviders.allSources
        XCTAssertEqual(sources.count, 6)
        XCTAssertEqual(Set(sources.map(\.id)).count, 6)
        XCTAssertEqual(Set(sources.map(\.provider)).count, 3)
        XCTAssertTrue(sources.allSatisfy { !$0.attribution.isEmpty })
        XCTAssertEqual(WeatherProviders.source("met-no-locationforecast")?.country, "NO")
        XCTAssertNil(WeatherProviders.source("inconnu"))
    }

    func testAttributionsAreListedOncePerLicence() {
        let readings = [
            SourceReading(source: WeatherProviders.openMeteoSources[0], temperature: 18,
                          precipitation: 0, windSpeed: 12),
            SourceReading(source: WeatherProviders.openMeteoSources[1], temperature: 18.5,
                          precipitation: 0, windSpeed: 12),
            SourceReading(source: WeatherProviders.metNorwaySource, temperature: 18.2,
                          precipitation: 0, windSpeed: 12),
        ]
        let attributions = WeatherProviders.attributions(for: readings)
        XCTAssertEqual(attributions.count, 2)
        XCTAssertTrue(attributions.contains { $0.contains("MET Norway") })
    }

    /// Le recoupement retient combien de fournisseurs ont parlé.
    func testConsensusCountsProviders() throws {
        let readings = [
            SourceReading(source: WeatherProviders.openMeteoSources[0], temperature: 18,
                          precipitation: 0, windSpeed: 12),
            SourceReading(source: WeatherProviders.metNorwaySource, temperature: 18.4,
                          precipitation: 0, windSpeed: 12),
        ]
        let result = try XCTUnwrap(ModelConsensus.consensus(readings, answered: 2, queried: 3))
        XCTAssertEqual(result.providersAnswered, 2)
        XCTAssertEqual(result.providersQueried, 3)
        XCTAssertEqual(result.agreement, .forte)
    }
}
