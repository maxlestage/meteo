import XCTest
@testable import Kliima

/// Les mêmes cas que `rust/klima-core/src/alerts.rs`. Quand un test change
/// d'un côté, il change de l'autre.
final class AlertsTests: XCTestCase {
    /// Calendrier fixe : les tests ne dépendent pas du fuseau de la machine.
    private let calendar: Calendar = {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: "Europe/Paris")!
        return calendar
    }()

    private func date(_ iso: String) -> Date {
        let formatter = DateFormatter()
        formatter.calendar = calendar
        formatter.timeZone = calendar.timeZone
        formatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss"
        return formatter.date(from: iso)!
    }

    private var now: Date { date("2026-04-15T09:00:00") }

    private func options() -> AlertOptions {
        AlertOptions(now: now, calendar: calendar)
    }

    private func hour(_ offset: Int, code: Int = 3, probability: Double = 10, temperature: Double = 14,
                      precipitation: Double = 0, gusts: Double = 15) -> HourlySample {
        HourlySample(
            time: now.addingTimeInterval(Double(offset) * 3600), weatherCode: code, isDay: true,
            precipitationProbability: probability, temperature: temperature,
            apparentTemperature: temperature - 1, relativeHumidity: 60, dewPoint: 7,
            precipitation: precipitation, windSpeed: 9, windGusts: gusts, uvIndex: 2
        )
    }

    private func hours(_ change: (inout [HourlySample]) -> Void = { _ in }) -> [HourlySample] {
        var h = (0..<24).map { hour($0) }
        change(&h)
        return h
    }

    private func sent(_ kind: AlertKind, hoursAgo: Double) -> AlertState {
        Alerts.recordSent(
            .empty,
            [Alert(kind: kind, at: now, titleKey: "", bodyKey: "", params: [:])],
            at: now.addingTimeInterval(-hoursAgo * 3600)
        )
    }

    private func kinds(_ h: [HourlySample], _ state: AlertState = .empty) -> [AlertKind] {
        Alerts.evaluate(hours: h, state: state, options: options()).map(\.kind)
    }

    // MARK: Plage de silence

    func testSilenceEnjambeMinuit() {
        XCTAssertTrue(Alerts.isQuiet(hour: 22, from: 22, to: 7))
        XCTAssertTrue(Alerts.isQuiet(hour: 3, from: 22, to: 7))
        XCTAssertFalse(Alerts.isQuiet(hour: 7, from: 22, to: 7))
        XCTAssertFalse(Alerts.isQuiet(hour: 14, from: 22, to: 7))
    }

    func testHorsSilenceLEnvoiPartToutDeSuite() {
        let send = date("2026-04-15T14:00:00")
        XCTAssertEqual(
            Alerts.deferPastQuietHours(send: send, event: date("2026-04-15T17:00:00"), options: options()),
            send
        )
    }

    func testLaNuitLEnvoiAttendLaReprise() {
        XCTAssertEqual(
            Alerts.deferPastQuietHours(
                send: date("2026-04-15T23:30:00"), event: date("2026-04-16T10:00:00"), options: options()
            ),
            date("2026-04-16T07:00:00")
        )
    }

    func testAvantLAubeLaRepriseEstLeMatinMeme() {
        XCTAssertEqual(
            Alerts.deferPastQuietHours(
                send: date("2026-04-16T03:00:00"), event: date("2026-04-16T10:00:00"), options: options()
            ),
            date("2026-04-16T07:00:00")
        )
    }

    func testEvenementPasseEstAbandonnePasRetarde() {
        XCTAssertNil(Alerts.deferPastQuietHours(
            send: date("2026-04-15T23:30:00"), event: date("2026-04-16T04:00:00"), options: options()
        ))
    }

    // MARK: La pluie

    func testLaPluieDansLHeureSAnnonceTantQuIlFaitSec() {
        let h = hours { $0[1] = hour(1, probability: 80, precipitation: 0.6) }
        let alerts = Alerts.evaluate(hours: h, state: .empty, options: options())
        XCTAssertEqual(alerts.map(\.kind), [.pluie])
        XCTAssertEqual(alerts[0].at, now.addingTimeInterval(3600))
        XCTAssertEqual(alerts[0].params["probability"], 80)
    }

    func testQuandIlPleutDejaOnNeLAnnoncePlus() {
        let h = hours {
            $0[0] = hour(0, precipitation: 1)
            $0[1] = hour(1, precipitation: 1)
        }
        XCTAssertTrue(kinds(h).isEmpty)
    }

    func testLaPluieDeCeSoirAttendDeSApprocher() {
        XCTAssertTrue(kinds(hours { $0[3] = hour(3, precipitation: 2) }).isEmpty)
    }

    // MARK: Orage, gel, chaleur, vent

    func testLOrageSAnnonceASonHeure() {
        let alerts = Alerts.evaluate(hours: hours { $0[5] = hour(5, code: 95) }, state: .empty, options: options())
        XCTAssertEqual(alerts.map(\.kind), [.orage])
        XCTAssertEqual(alerts[0].at, now.addingTimeInterval(5 * 3600))
    }

    func testLeGelDitLeMinimumAttendu() {
        let h = hours {
            $0[8] = hour(8, temperature: -0.5)
            $0[10] = hour(10, temperature: -3)
        }
        let alerts = Alerts.evaluate(hours: h, state: .empty, options: options())
        XCTAssertEqual(alerts.map(\.kind), [.gel])
        XCTAssertEqual(alerts[0].at, now.addingTimeInterval(8 * 3600))
        XCTAssertEqual(alerts[0].params["temperature"], -3)
    }

    func testLaChaleurSAnnonceALHeureLaPlusChaude() {
        let h = hours {
            $0[4] = hour(4, temperature: 33)
            $0[6] = hour(6, temperature: 36)
            $0[7] = hour(7, temperature: 34)
        }
        let alerts = Alerts.evaluate(hours: h, state: .empty, options: options())
        XCTAssertEqual(alerts.map(\.kind), [.chaleur])
        XCTAssertEqual(alerts[0].at, now.addingTimeInterval(6 * 3600))
        XCTAssertEqual(alerts[0].params["temperature"], 36)
    }

    func testLeVentDitSaRafaleLaPlusForte() {
        let h = hours {
            $0[2] = hour(2, gusts: 72)
            $0[3] = hour(3, gusts: 90)
        }
        let alerts = Alerts.evaluate(hours: h, state: .empty, options: options())
        XCTAssertEqual(alerts.map(\.kind), [.vent])
        XCTAssertEqual(alerts[0].at, now.addingTimeInterval(2 * 3600))
        XCTAssertEqual(alerts[0].params["gusts"], 90)
    }

    func testAuDelaDeDouzeHeuresIlEstTropTot() {
        let h = hours {
            $0[12] = hour(12, code: 95)
            $0[13] = hour(13, temperature: -2)
            $0[14] = hour(14, gusts: 100)
        }
        XCTAssertTrue(kinds(h).isEmpty)
    }

    func testUneJourneeSansRienADireNeDitRien() {
        XCTAssertTrue(kinds(hours()).isEmpty)
        XCTAssertTrue(kinds([]).isEmpty)
    }

    func testLeDomaineRenvoieDesClesJamaisDesPhrases() {
        let alert = Alerts.evaluate(hours: hours { $0[5] = hour(5, code: 95) }, state: .empty, options: options())[0]
        XCTAssertEqual(alert.titleKey, "alert.orage.title")
        XCTAssertEqual(alert.bodyKey, "alert.orage.body")
    }

    // MARK: Le délai de garde

    func testUneAlerteDejaPartieNeRepartPasDansLaFoulee() {
        XCTAssertTrue(kinds(hours { $0[5] = hour(5, code: 95) }, sent(.orage, hoursAgo: 2)).isEmpty)
    }

    func testPasseLeDelaiElleRepart() {
        XCTAssertEqual(kinds(hours { $0[5] = hour(5, code: 95) }, sent(.orage, hoursAgo: 7)), [.orage])
    }

    func testLeDelaiDUneNatureNeBaillonnePasUneAutre() {
        let h = hours {
            $0[5] = hour(5, code: 95)
            $0[8] = hour(8, temperature: -1)
        }
        XCTAssertEqual(kinds(h, sent(.orage, hoursAgo: 1)), [.gel])
    }

    func testLEtatSeSerialiseEtLitCeluiDUneVersionPrecedente() throws {
        let after = sent(.pluie, hoursAgo: 0)
        let round = try JSONDecoder().decode(AlertState.self, from: JSONEncoder().encode(after))
        XCTAssertEqual(round, after)

        // Une version précédente gardait l'état du sol : la clé en trop est
        // ignorée, l'état se relit.
        let ancien = Data(#"{"lastSent":{},"lastSoilState":"sature"}"#.utf8)
        XCTAssertEqual(try JSONDecoder().decode(AlertState.self, from: ancien), .empty)
    }

    func testLesCinqNaturesOntUnCodeDistinct() {
        XCTAssertEqual(Set(AlertKind.allCases.map(\.rawValue)).count, 5)
    }
}
