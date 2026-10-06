import XCTest
@testable import Kliima

/// Le direct : mêmes cas que `rust/klima-ui/src/crochets/direct.rs`.
@MainActor
final class DirectRelaisTests: XCTestCase {

    func testLAdresseDuDirectPasseEnWebSocket() {
        XCTAssertEqual(
            DirectRelais.adresse(relais: URL(string: "https://klima.example")!)?.absoluteString,
            "wss://klima.example/v1/direct"
        )
        XCTAssertEqual(
            DirectRelais.adresse(relais: URL(string: "http://localhost:8787/")!)?.absoluteString,
            "ws://localhost:8787/v1/direct"
        )
        XCTAssertNil(DirectRelais.adresse(relais: URL(string: "ftp://klima.example")!))
    }

    func testLAbonnementPorteLePointEtLesJours() {
        let paris = Parcelle(name: "Paris", latitude: 48.8566, longitude: 2.3522)
        XCTAssertEqual(DirectRelais.abonnement(paris, jours: 7), #"{"latitude":48.8566,"longitude":2.3522,"jours":7}"#)
    }

    func testUnMessageDuRelaisSeLitAvecSonPoint() throws {
        let message = try XCTUnwrap(DirectRelais.lire(
            #"{"sujet":"quarts","latitude":48.8566,"longitude":2.3522,"corps":{"utc_offset_seconds":7200}}"#
        ))
        XCTAssertEqual(message.sujet, "quarts")
        XCTAssertEqual(message.latitude, 48.8566)
        XCTAssertEqual(message.longitude, 2.3522)
        let corps = try XCTUnwrap(JSONSerialization.jsonObject(with: message.corps) as? [String: Any])
        XCTAssertEqual(corps["utc_offset_seconds"] as? Int, 7200)

        XCTAssertNil(DirectRelais.lire("pas du json"))
        XCTAssertNil(DirectRelais.lire(#"{"sujet":"base","corps":{}}"#), "sans point, écarté")
    }

    func testLaReconnexionAttendDePlusEnPlusSansDepasserLaMinute() {
        XCTAssertEqual(DirectRelais.attente(essai: 1), 2)
        XCTAssertEqual(DirectRelais.attente(essai: 3), 8)
        XCTAssertEqual(DirectRelais.attente(essai: 10), 60)
    }

    /// Les corps poussés se lisent comme des réponses de requête.
    func testLesCorpsPoussesFontLaMemePrevisionRecoupee() throws {
        let base = Data("""
        {"latitude":48.86,"longitude":2.35,"elevation":35,"timezone":"Europe/Paris","utc_offset_seconds":7200,
         "current":{"time":"2026-05-12T12:20","temperature_2m":18.4,"apparent_temperature":17.4,"relative_humidity_2m":60,
           "weather_code":1,"is_day":1,"wind_speed_10m":10,"wind_gusts_10m":20,"pressure_msl":1015},
         "hourly":{"time":["2026-05-12T12:00"],"temperature_2m":[18],"apparent_temperature":[17],"weather_code":[1],
           "is_day":[1],"precipitation_probability":[10],"relative_humidity_2m":[60],"dew_point_2m":[10],
           "precipitation":[0],"wind_speed_10m":[10],"wind_gusts_10m":[20],"uv_index":[4]},
         "daily":{"time":["2026-05-12"],"weather_code":[1],"sunrise":["2026-05-12T06:20"],"sunset":["2026-05-12T21:20"],
           "temperature_2m_min":[10],"temperature_2m_max":[20],"precipitation_sum":[0],
           "precipitation_probability_max":[5],"wind_gusts_10m_max":[30],"uv_index_max":[5]}}
        """.utf8)
        let ensemble = Data("""
        {"timezone":"Europe/Paris","hourly":{"time":["2026-05-12T12:00"],
          "temperature_2m_meteofrance_seamless":[20],"temperature_2m_ecmwf_ifs025":[20]}}
        """.utf8)
        let paris = Parcelle(name: "Paris", latitude: 48.8566, longitude: 2.3522)
        let prevision = try AgroWeatherService.decodeBase(base, parcelle: paris)
        let sources = WeatherProviders.ensemble(openMeteo: ensemble, met: nil, station: nil)
        let recoupee = AgroWeatherService.recouper(prevision, avec: sources)
        XCTAssertEqual(recoupee.hourly[0].temperature, 20)
        XCTAssertEqual(recoupee.current.temperature, 20.4, accuracy: 1e-9)
        XCTAssertEqual(recoupee.sources, ["meteofrance_seamless", "ecmwf_ifs025"])
    }
}
