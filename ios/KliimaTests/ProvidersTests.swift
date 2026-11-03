import XCTest
@testable import Kliima

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
        XCTAssertEqual(sources.count, 9)
        XCTAssertEqual(Set(sources.map(\.id)).count, 9)
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

    /// Open-Meteo en panne : MET Norway fait seul la prévision de base, à
    /// l'heure, sans inventer ni UV ni rafales quand la réponse n'en a pas.
    func testMetNorwayFaitLaPrevisionDeSecours() throws {
        let f = ISO8601DateFormatter()
        let debut = Date(timeIntervalSince1970: (Date().timeIntervalSince1970 / 3600).rounded(.down) * 3600)
        func heure(_ i: Double, _ t: Double, _ pluie: Double, _ symbole: String) -> String {
            """
            {"time": "\(f.string(from: debut.addingTimeInterval(i * 3600)))",
             "data": {"instant": {"details": {"air_temperature": \(t), "wind_speed": 5, "relative_humidity": 80, "air_pressure_at_sea_level": 1012}},
                      "next_1_hours": {"summary": {"symbol_code": "\(symbole)"}, "details": {"precipitation_amount": \(pluie)}}}}
            """
        }
        let payload = try MetPayload.decode(data("""
        {"properties": {"timeseries": [
          {"time": "\(f.string(from: debut.addingTimeInterval(-3600)))", "data": {"instant": {"details": {"air_temperature": 1}}}},
          \(heure(0, 8, 0, "clearsky_night")), \(heure(1, 9, 1.2, "rain")), \(heure(2, 11, 0, "fair_day"))
        ]}}
        """))
        let prevision = try XCTUnwrap(AgroWeatherService.baseMetNorway(payload, parcelle: .paris))
        XCTAssertEqual(prevision.hourly.count, 3, "l'heure passée est écartée")
        XCTAssertEqual(prevision.current.temperature, 8)
        XCTAssertFalse(prevision.current.isDay)
        XCTAssertEqual(prevision.current.weatherCode, 0)
        XCTAssertEqual(prevision.current.pressure, 1012)
        XCTAssertEqual(prevision.hourly[1].weatherCode, 63)
        XCTAssertEqual(prevision.hourly[1].precipitationProbability, 70)
        XCTAssertEqual(prevision.hourly[0].windSpeed, 18, accuracy: 0.0001)
        XCTAssertEqual(prevision.hourly[0].windGusts, prevision.hourly[0].windSpeed)
        // MET Norway ne donne pas l'UV : inconnu, pas « 0 · faible ».
        XCTAssertNil(prevision.hourly[0].uvIndex)
        XCTAssertNil(prevision.daily.first?.uvIndexMax)
        // 8 °C et 18 km/h : le vent refroidit.
        XCTAssertLessThan(prevision.current.apparentTemperature, 8)
        XCTAssertFalse(prevision.daily.isEmpty)
        // Aujourd'hui se résume depuis maintenant, avec celles de ses heures
        // qu'on a — quelle que soit l'heure où le test tourne.
        var calendrier = Calendar(identifier: .gregorian)
        calendrier.timeZone = .current
        let aujourdhui = prevision.hourly.filter { calendrier.isDate($0.time, inSameDayAs: debut) }
        XCTAssertEqual(prevision.daily.first?.temperatureMax, aujourdhui.map(\.temperature).max())
        XCTAssertNil(AgroWeatherService.baseMetNorway(try MetPayload.decode(data("{}")), parcelle: .paris))
    }

    /// MET Norway sans pression : inconnue, pas « 0 hPa ».
    func testLaPrevisionDeSecoursSansPressionNeDitPasZero() throws {
        let f = ISO8601DateFormatter()
        let debut = Date(timeIntervalSince1970: (Date().timeIntervalSince1970 / 3600).rounded(.down) * 3600)
        let payload = try MetPayload.decode(data("""
        {"properties": {"timeseries": [
          {"time": "\(f.string(from: debut))",
           "data": {"instant": {"details": {"air_temperature": 8, "wind_speed": 5, "relative_humidity": 80}},
                    "next_1_hours": {"summary": {"symbol_code": "cloudy"}, "details": {"precipitation_amount": 0}}}}
        ]}}
        """))
        let prevision = try XCTUnwrap(AgroWeatherService.baseMetNorway(payload, parcelle: .paris))
        XCTAssertNil(prevision.current.pressure)
    }

    /// La forme complète de MET Norway donne rafales, probabilité de pluie
    /// et point de rosée : la prévision de secours les lit, sans les deviner.
    func testLaPrevisionDeSecoursLitLaFormeComplete() throws {
        let f = ISO8601DateFormatter()
        let debut = Date(timeIntervalSince1970: (Date().timeIntervalSince1970 / 3600).rounded(.down) * 3600)
        let payload = try MetPayload.decode(data("""
        {"properties": {"timeseries": [
          {"time": "\(f.string(from: debut))",
           "data": {"instant": {"details": {"air_temperature": 12, "wind_speed": 5, "wind_speed_of_gust": 12,
                                            "relative_humidity": 80, "dew_point_temperature": 8.4}},
                    "next_1_hours": {"summary": {"symbol_code": "rain"},
                                     "details": {"precipitation_amount": 0.4, "probability_of_precipitation": 42.5}}}},
          {"time": "\(f.string(from: debut.addingTimeInterval(3600)))",
           "data": {"instant": {"details": {"air_temperature": 12, "wind_speed": 5, "wind_speed_of_gust": 3}},
                    "next_1_hours": {"summary": {"symbol_code": "cloudy"}, "details": {"precipitation_amount": 0}}}}
        ]}}
        """))
        let prevision = try XCTUnwrap(AgroWeatherService.baseMetNorway(payload, parcelle: .paris))
        XCTAssertEqual(prevision.hourly[0].windGusts, 43.2, accuracy: 0.0001)
        XCTAssertEqual(prevision.current.windGusts, 43.2, accuracy: 0.0001)
        XCTAssertEqual(prevision.hourly[0].precipitationProbability, 42.5)
        XCTAssertEqual(prevision.hourly[0].dewPoint, 8.4)
        // Une rafale annoncée sous le vent moyen ne le fait pas baisser ;
        // sans probabilité, le risque vient du cumul.
        XCTAssertEqual(prevision.hourly[1].windGusts, prevision.hourly[1].windSpeed)
        XCTAssertEqual(prevision.hourly[1].precipitationProbability, 5)
        XCTAssertEqual(prevision.daily.first?.windGustsMax ?? 0, 43.2, accuracy: 0.0001)
    }

    /// La prévision de secours date à l'heure de la ville, pas du lecteur :
    /// le fuseau vient de la dernière réponse d'Open-Meteo gardée.
    func testLaPrevisionDeSecoursEstALHeureDeLaVille() throws {
        let tokyo = try XCTUnwrap(TimeZone(identifier: "Asia/Tokyo"))
        XCTAssertEqual(AgroWeatherService.fuseauGarde(data(#"{"timezone": "Asia/Tokyo", "elevation": 40}"#)), tokyo)
        XCTAssertNil(AgroWeatherService.fuseauGarde(data("{}")))
        XCTAssertNil(AgroWeatherService.fuseauGarde(data(#"{"timezone": "Nulle/Part"}"#)))

        let f = ISO8601DateFormatter()
        let debut = Date(timeIntervalSince1970: (Date().timeIntervalSince1970 / 3600).rounded(.down) * 3600)
        let heures = (0..<60).map { i in
            """
            {"time": "\(f.string(from: debut.addingTimeInterval(Double(i) * 3600)))",
             "data": {"instant": {"details": {"air_temperature": 12, "wind_speed": 2, "relative_humidity": 70}},
                      "next_1_hours": {"summary": {"symbol_code": "cloudy"}, "details": {"precipitation_amount": 0}}}}
            """
        }.joined(separator: ",")
        let payload = try MetPayload.decode(data(#"{"properties": {"timeseries": ["# + heures + "]}}"))
        let prevision = try XCTUnwrap(AgroWeatherService.baseMetNorway(payload, parcelle: .paris, zone: tokyo))
        XCTAssertEqual(prevision.timezone, "Asia/Tokyo")
        var calendrier = Calendar(identifier: .gregorian)
        calendrier.timeZone = tokyo
        XCTAssertGreaterThanOrEqual(prevision.daily.count, 2, "soixante heures couvrent aujourd'hui et demain")
        for jour in prevision.daily {
            XCTAssertEqual(jour.date, calendrier.startOfDay(for: jour.date), "les jours commencent à minuit à Tokyo")
        }
    }

    /// La série horaire de MET Norway s'arrête en cours de journée : ce
    /// dernier morceau ne se résume pas en jour entier. Aujourd'hui, lui, se
    /// résume depuis maintenant.
    func testLaPrevisionDeSecoursNeResumePasUnJourEntame() throws {
        let paris = try XCTUnwrap(TimeZone(identifier: "Europe/Paris"))
        var calendrier = Calendar(identifier: .gregorian)
        calendrier.timeZone = paris
        let f = ISO8601DateFormatter()
        let debut = Date(timeIntervalSince1970: (Date().timeIntervalSince1970 / 3600).rounded(.down) * 3600)
        let demain = try XCTUnwrap(calendrier.date(byAdding: .day, value: 1, to: calendrier.startOfDay(for: debut)))
        let apresDemain = try XCTUnwrap(calendrier.date(byAdding: .day, value: 1, to: demain))
        // De maintenant à après-demain 05:00 : demain est entier, après-demain
        // n'a que sa nuit — froide et sèche.
        let fin = apresDemain.addingTimeInterval(6 * 3600)
        let n = Int(fin.timeIntervalSince(debut) / 3600)
        let heures = (0..<n).map { i -> String in
            let t = debut.addingTimeInterval(Double(i) * 3600)
            let nuitDApresDemain = t >= apresDemain
            return """
            {"time": "\(f.string(from: t))",
             "data": {"instant": {"details": {"air_temperature": \(nuitDApresDemain ? 2 : 12), "wind_speed": 2, "relative_humidity": 70}},
                      "next_1_hours": {"summary": {"symbol_code": "cloudy"}, "details": {"precipitation_amount": 0}}}}
            """
        }.joined(separator: ",")
        let payload = try MetPayload.decode(data(#"{"properties": {"timeseries": ["# + heures + "]}}"))
        let prevision = try XCTUnwrap(AgroWeatherService.baseMetNorway(payload, parcelle: .paris, zone: paris))
        XCTAssertEqual(prevision.daily.map(\.date), [calendrier.startOfDay(for: debut), demain],
                       "aujourd'hui et demain ; pas la nuit d'après-demain")
        XCTAssertEqual(prevision.daily.map(\.temperatureMin).min(), 12, "les 2 °C de la nuit entamée ne font pas un jour")
        XCTAssertEqual(prevision.hourly.last?.temperature, 2, "les heures, elles, restent")
    }
}
