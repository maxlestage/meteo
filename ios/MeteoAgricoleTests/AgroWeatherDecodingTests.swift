import XCTest
@testable import MeteoAgricole

/// Vérifie le contrat avec Open-Meteo : noms de champs, valeurs nulles et
/// interprétation des horodatages dans le fuseau de la parcelle.
final class AgroWeatherDecodingTests: XCTestCase {
    func testDecodesOpenMeteoPayload() throws {
        let json = """
        {
          "latitude": 48.44, "longitude": 1.48, "elevation": 155.0,
          "timezone": "Europe/Paris", "timezone_abbreviation": "CEST",
          "hourly": {
            "time": ["2026-05-12T00:00", "2026-05-12T01:00"],
            "temperature_2m": [11.4, null],
            "relative_humidity_2m": [92, 94],
            "dew_point_2m": [10.1, 10.4],
            "precipitation": [0.0, 1.8],
            "wind_speed_10m": [7.2, 24.0],
            "wind_gusts_10m": [15.1, 33.0],
            "soil_temperature_6cm": [13.9, 13.6],
            "soil_moisture_3_to_9cm": [0.238, 0.239],
            "et0_fao_evapotranspiration": [0.01, 0.0],
            "vapour_pressure_deficit": [0.11, 0.09]
          },
          "daily": {
            "time": ["2026-05-12", "2026-05-13"],
            "temperature_2m_min": [3.0, 1.2],
            "temperature_2m_max": [19.0, 21.4],
            "precipitation_sum": [1.8, 0.0],
            "precipitation_probability_max": [40, null],
            "et0_fao_evapotranspiration": [3.4, 3.9],
            "wind_gusts_10m_max": [34.0, 28.0]
          }
        }
        """.data(using: .utf8)!

        let payload = try JSONDecoder().decode(ForecastPayload.self, from: json)
        XCTAssertEqual(payload.timezone, "Europe/Paris")
        XCTAssertEqual(payload.elevation, 155.0)

        let zone = try XCTUnwrap(TimeZone(identifier: payload.timezone))
        let hours = payload.hourly.decode(in: zone)
        XCTAssertEqual(hours.count, 2)
        XCTAssertEqual(hours[0].temperature, 11.4)
        XCTAssertEqual(hours[0].relativeHumidity, 92)
        XCTAssertEqual(hours[0].soilMoisture3to9cm, 0.238)
        XCTAssertEqual(hours[0].vapourPressureDeficit, 0.11)
        // Un `null` de l'API se lit 0, sans décaler la série.
        XCTAssertEqual(hours[1].temperature, 0)
        XCTAssertEqual(hours[1].windSpeed, 24.0)

        // L'horodatage est interprété dans le fuseau de la parcelle, pas en UTC.
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = zone
        XCTAssertEqual(calendar.component(.hour, from: hours[0].time), 0)
        XCTAssertEqual(hours[1].time.timeIntervalSince(hours[0].time), 3600)

        let days = payload.daily.decode(in: zone)
        XCTAssertEqual(days.count, 2)
        XCTAssertEqual(days[0].temperatureMin, 3.0)
        XCTAssertEqual(days[0].precipitationSum, 1.8)
        XCTAssertEqual(days[0].et0Sum, 3.4)
        XCTAssertEqual(days[1].precipitationProbabilityMax, 0)
        XCTAssertEqual(calendar.component(.day, from: days[1].date), 13)

        // Le géocodage.
        let geo = try JSONDecoder().decode(GeocodingPayload.self, from: """
        {"results":[{"id":1,"name":"Chartres","latitude":48.44,"longitude":1.48,"admin1":"Eure-et-Loir","country":"France"}]}
        """.data(using: .utf8)!)
        XCTAssertEqual(geo.results?.first?.name, "Chartres")
        XCTAssertEqual(geo.results?.first?.admin1, "Eure-et-Loir")
    }
}
