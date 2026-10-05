import XCTest
@testable import Kliima

/// Vérifie le contrat avec Open-Meteo : noms de champs, valeurs nulles et
/// interprétation des horodatages dans le fuseau de la ville.
final class AgroWeatherDecodingTests: XCTestCase {
    func testDecodesOpenMeteoPayload() throws {
        let json = """
        {
          "latitude": 48.44, "longitude": 1.48, "elevation": 155.0,
          "timezone": "Europe/Paris", "timezone_abbreviation": "CEST",
          "current": {
            "time": "2026-05-12T01:00",
            "temperature_2m": 11.2,
            "apparent_temperature": 10.4,
            "relative_humidity_2m": 93,
            "weather_code": 53,
            "is_day": 0,
            "wind_speed_10m": 12.0,
            "wind_gusts_10m": 25.0,
            "pressure_msl": 1012.4
          },
          "hourly": {
            "time": ["2026-05-12T00:00", "2026-05-12T01:00"],
            "temperature_2m": [11.4, null],
            "apparent_temperature": [10.2, 9.8],
            "relative_humidity_2m": [92, 94],
            "dew_point_2m": [10.1, 10.4],
            "precipitation": [0.0, 1.8],
            "wind_speed_10m": [7.2, 24.0],
            "wind_gusts_10m": [15.1, 33.0],
            "uv_index": [0.0, 0.4],
            "weather_code": [51, 61],
            "is_day": [0, 0],
            "precipitation_probability": [35, 80]
          },
          "daily": {
            "time": ["2026-05-12", "2026-05-13"],
            "temperature_2m_min": [3.0, 1.2],
            "temperature_2m_max": [19.0, 21.4],
            "precipitation_sum": [1.8, 0.0],
            "precipitation_probability_max": [40, null],
            "wind_gusts_10m_max": [34.0, 28.0],
            "uv_index_max": [5.6, 6.2],
            "weather_code": [61, 3],
            "sunrise": ["2026-05-12T06:32", "2026-05-13T06:31"],
            "sunset": ["2026-05-12T21:24", null]
          }
        }
        """.data(using: .utf8)!

        let payload = try JSONDecoder().decode(ForecastPayload.self, from: json)
        XCTAssertEqual(payload.timezone, "Europe/Paris")
        XCTAssertEqual(payload.elevation, 155.0)

        let current = payload.current.decode(in: TimeZone(identifier: payload.timezone) ?? .current)
        XCTAssertEqual(current.temperature, 11.2)
        XCTAssertEqual(current.apparentTemperature, 10.4)
        XCTAssertEqual(current.weatherCode, 53)
        XCTAssertFalse(current.isDay)
        XCTAssertEqual(current.windGusts, 25.0)
        XCTAssertEqual(current.pressure, 1012.4)

        let zone = try XCTUnwrap(TimeZone(identifier: payload.timezone))
        let hours = payload.hourly.decode(in: zone)
        XCTAssertEqual(hours.count, 2)
        XCTAssertEqual(hours[0].temperature, 11.4)
        XCTAssertEqual(hours[0].relativeHumidity, 92)
        XCTAssertEqual(hours[0].apparentTemperature, 10.2)
        XCTAssertEqual(hours[1].uvIndex, 0.4)
        XCTAssertEqual(hours[0].weatherCode, 51)
        XCTAssertFalse(hours[0].isDay)
        XCTAssertEqual(hours[0].precipitationProbability, 35)
        // Un `null` de l'API se lit 0, sans décaler la série.
        XCTAssertEqual(hours[1].temperature, 0)
        XCTAssertEqual(hours[1].windSpeed, 24.0)

        // L'horodatage est interprété dans le fuseau de la ville, pas en UTC.
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = zone
        XCTAssertEqual(calendar.component(.hour, from: hours[0].time), 0)
        XCTAssertEqual(hours[1].time.timeIntervalSince(hours[0].time), 3600)

        let days = payload.daily.decode(in: zone)
        XCTAssertEqual(days.count, 2)
        XCTAssertEqual(days[0].temperatureMin, 3.0)
        XCTAssertEqual(days[0].precipitationSum, 1.8)
        XCTAssertEqual(days[0].uvIndexMax, 5.6)
        XCTAssertEqual(days[1].precipitationProbabilityMax, 0)
        XCTAssertEqual(days[0].weatherCode, 61)
        XCTAssertEqual(calendar.component(.hour, from: try XCTUnwrap(days[0].sunrise)), 6)
        XCTAssertEqual(calendar.component(.minute, from: try XCTUnwrap(days[0].sunset)), 24)
        // Un lever ou coucher absent reste absent, sans décaler la série.
        XCTAssertNil(days[1].sunset)
        XCTAssertEqual(calendar.component(.day, from: days[1].date), 13)

        // Le géocodage.
        let geo = try JSONDecoder().decode(GeocodingPayload.self, from: """
        {"results":[{"id":1,"name":"Chartres","latitude":48.44,"longitude":1.48,"admin1":"Eure-et-Loir","country":"France"}]}
        """.data(using: .utf8)!)
        XCTAssertEqual(geo.results?.first?.name, "Chartres")
        XCTAssertEqual(geo.results?.first?.admin1, "Eure-et-Loir")
    }

    /// L'API renvoie la journée depuis minuit : les séries doivent repartir de
    /// l'heure en cours pour que « maintenant » soit le premier élément.
    func testHourlySeriesStartsAtTheCurrentHour() throws {
        let midnight = Date(timeIntervalSince1970: 1_778_544_000)
        let hours = (0..<24).map { index in
            HourlySample(
                time: midnight.addingTimeInterval(Double(index) * 3600),
                weatherCode: 3,
                isDay: true,
                precipitationProbability: 10,
                temperature: 15,
                apparentTemperature: 14,
                relativeHumidity: 70,
                dewPoint: 9,
                precipitation: 0,
                windSpeed: 8,
                windGusts: 14,
                uvIndex: 1
            )
        }

        // Il est 9 h 40 : la série doit commencer à 9 h.
        let now = midnight.addingTimeInterval(9 * 3600 + 40 * 60)
        let trimmed = AgroWeatherService.fromCurrentHour(hours, now: now)
        XCTAssertEqual(trimmed.count, 15)
        XCTAssertEqual(trimmed.first?.time, midnight.addingTimeInterval(9 * 3600))

        // Si l'heure courante sort de la série, on ne renvoie pas du vide.
        let later = midnight.addingTimeInterval(48 * 3600)
        XCTAssertEqual(AgroWeatherService.fromCurrentHour(hours, now: later).count, hours.count)
    }
}

/// Le contrat avec le service de qualité de l'air : mêmes cas que
/// `klima-api/src/air.rs`.
final class AirDecodingTests: XCTestCase {

    func testLitLIndiceLesPolluantsEtLesPollensPresents() throws {
        let reponse = Data("""
        {"current":{"time":"2026-05-12T09:00","european_aqi":38,"pm2_5":9.4,"pm10":17.2,
         "nitrogen_dioxide":21.0,"ozone":64.0,"alder_pollen":0.0,"birch_pollen":12.5,
         "grass_pollen":41.0,"mugwort_pollen":null,"olive_pollen":0.2,"ragweed_pollen":0.0}}
        """.utf8)
        let air = try XCTUnwrap(AgroWeatherService.decodeAir(reponse))
        XCTAssertEqual(air.europeanAqi, 38)
        XCTAssertEqual(air.pm25, 9.4)
        XCTAssertEqual(air.ozone, 64)
        XCTAssertEqual(air.pollens.count, 5, "l'armoise à null est écartée")
        XCTAssertTrue(air.pollens.contains { $0.0 == .graminees && $0.1 == 41 })
    }

    func testHorsDEuropePasDePollensMaisUnIndice() throws {
        let air = try XCTUnwrap(AgroWeatherService.decodeAir(Data(#"{"current":{"european_aqi":12,"pm2_5":3.1}}"#.utf8)))
        XCTAssertEqual(air.europeanAqi, 12)
        XCTAssertTrue(air.pollens.isEmpty)
    }

    func testUneReponseSansMesureNEstPasUnAirPur() {
        XCTAssertNil(AgroWeatherService.decodeAir(Data(#"{"latitude":1}"#.utf8)))
        XCTAssertNil(AgroWeatherService.decodeAir(Data("pas du json".utf8)))
    }

    func testLesVariablesSontCellesDuCoeur() {
        XCTAssertEqual(AgroWeatherService.airVariables, [
            "european_aqi", "pm2_5", "pm10", "nitrogen_dioxide", "ozone",
            "alder_pollen", "birch_pollen", "grass_pollen", "mugwort_pollen",
            "olive_pollen", "ragweed_pollen",
        ])
    }
}
