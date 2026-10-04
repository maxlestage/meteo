import XCTest
@testable import Kliima

/// L'île dynamique tenue à l'heure par le relais.
///
/// L'envoi lui-même demande Apple et un appareil. Ce qui se teste ici, ce sont
/// les deux bouts du contrat : ce que l'iPhone envoie au relais, et ce que le
/// relais pousse en retour — qu'ActivityKit décode avec un `JSONDecoder` par
/// défaut, sans qu'on puisse rien y régler.
final class IlesRelaisTests: XCTestCase {

    func testLeJetonPartEnHexadecimalEtLaParcelleArrondieALaMaille() throws {
        let corps = IlesRelais.corps(jeton: Data([0x0a, 0xbc, 0xff]), latitude: 48.4567, longitude: 1.4891)
        let lu = try XCTUnwrap(JSONSerialization.jsonObject(with: corps) as? [String: Any])
        XCTAssertEqual(lu["jeton"] as? String, "0abcff")
        XCTAssertEqual(lu["latitude"] as? Double, Position.snap(48.4567))
        XCTAssertEqual(lu["longitude"] as? Double, Position.snap(1.4891))
        XCTAssertNotEqual(lu["latitude"] as? Double, 48.4567)
        XCTAssertNil(lu["name"], "le nom de la parcelle ne quitte pas l'appareil")
    }

    /// Le `content-state` tel que le relais le produit (`klima-relay/src/iles.rs`) :
    /// mêmes clés, dates en secondes depuis le 1er janvier 2001.
    func testCeQuePousseLeRelaisSeDecodeTelQuel() throws {
        let pousse = #"""
        {"temperature":22.0,"apparentTemperature":22.0,"weatherCode":3,"isDay":true,
         "windSpeed":14.0,"temperatureMin":11.0,"temperatureMax":23.0,
         "updatedAt":781272300.0,
         "next":{"start":781275600.0,"temperature":23.0,"weatherCode":61,"isDay":true,
                 "precipitationProbability":70.0,"windSpeed":18.0}}
        """#
        let etat = try JSONDecoder().decode(WeatherActivityAttributes.ContentState.self,
                                            from: Data(pousse.utf8))
        XCTAssertEqual(etat.temperature, 22)
        XCTAssertEqual(etat.next?.weatherCode, 61)
        XCTAssertEqual(etat.next?.start, Date(timeIntervalSinceReferenceDate: 781_275_600))
        XCTAssertEqual(etat.updatedAt, Date(timeIntervalSinceReferenceDate: 781_272_300))

        // Passé l'heure, l'île montre la suivante sans rien attendre.
        XCTAssertEqual(etat.now(stale: true).temperature, 23)
        XCTAssertNil(etat.upcoming(stale: true))
    }

    func testEnFinDeSerieLeRelaisOmetLHeureSuivante() throws {
        let pousse = #"""
        {"temperature":18.0,"apparentTemperature":18.0,"weatherCode":1,"isDay":false,
         "windSpeed":6.0,"temperatureMin":11.0,"temperatureMax":23.0,"updatedAt":781290000.0}
        """#
        let etat = try JSONDecoder().decode(WeatherActivityAttributes.ContentState.self,
                                            from: Data(pousse.utf8))
        XCTAssertNil(etat.next)
        XCTAssertEqual(etat.now(stale: true).temperature, 18)
    }

    func testUneActiviteDUneVersionPrecedenteSeRelitSansCoordonnees() throws {
        let ancienne = #"{"parcelleName":"Chartres","timeZoneIdentifier":"Europe/Paris"}"#
        let lu = try JSONDecoder().decode(WeatherActivityAttributes.self, from: Data(ancienne.utf8))
        XCTAssertNil(lu.latitude)
        XCTAssertEqual(lu.parcelleName, "Chartres")
    }
}
