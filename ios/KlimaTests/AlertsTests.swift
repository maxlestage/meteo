import XCTest
@testable import Klima

/// Les mêmes cas que `core/src/alerts.test.ts`. Quand un test change d'un
/// côté, il change de l'autre.
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

    private func options(now: Date? = nil) -> AlertOptions {
        AlertOptions(now: now ?? self.now, calendar: calendar)
    }

    private func hourAt(_ offset: Double) -> Date {
        now.addingTimeInterval(offset * 3600)
    }

    private func hour(_ offset: Double, temperature: Double = 14, precipitation: Double = 0) -> HourlySample {
        HourlySample(
            time: hourAt(offset), weatherCode: 3, isDay: true,
            precipitationProbability: 10, temperature: temperature,
            relativeHumidity: 60, dewPoint: 7, precipitation: precipitation,
            windSpeed: 9, windGusts: 15, soilTemperature6cm: 12,
            soilMoisture3to9cm: 0.22, et0: 0.1, vapourPressureDeficit: 0.6
        )
    }

    private func summary(
        soil: SoilState = .ressuye,
        frost: FrostRisk = FrostRisk(severity: .aucun, minTemperature: 6, hoarFrost: false),
        nextSpray: SprayOpportunity? = nil
    ) -> AgroSummary {
        AgroSummary(
            water: WaterBalance(
                precipitation: 10, evapotranspiration: 8, balance: 2,
                status: .equilibre, irrigationAdvice: 0
            ),
            soil: SoilCondition(
                moisture: 0.22, temperature: 12, state: soil,
                trafficable: true, sowable: true
            ),
            disease: DiseasePressure(leafWetnessHours: 2, level: .faible),
            frost: frost,
            gdd: 120,
            nextSpray: nextSpray
        )
    }

    private func spray(_ start: Double, _ end: Double, score: Int = 90) -> SprayOpportunity {
        SprayOpportunity(start: hourAt(start), end: hourAt(end), score: score)
    }

    // MARK: Plage de silence

    func testSilenceEnjambeMinuit() {
        XCTAssertTrue(Alerts.isQuiet(hour: 22, from: 21, to: 6))
        XCTAssertTrue(Alerts.isQuiet(hour: 3, from: 21, to: 6))
        XCTAssertFalse(Alerts.isQuiet(hour: 6, from: 21, to: 6))
        XCTAssertFalse(Alerts.isQuiet(hour: 14, from: 21, to: 6))
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
                send: date("2026-04-15T23:30:00"),
                event: date("2026-04-16T10:00:00"),
                options: options()
            ),
            date("2026-04-16T06:00:00")
        )
    }

    func testAvantLAubeLaRepriseEstLeMatinMeme() {
        XCTAssertEqual(
            Alerts.deferPastQuietHours(
                send: date("2026-04-16T03:00:00"),
                event: date("2026-04-16T10:00:00"),
                options: options()
            ),
            date("2026-04-16T06:00:00")
        )
    }

    func testEvenementPasseEstAbandonnePasRetarde() {
        // Mieux vaut se taire que raconter la veille.
        XCTAssertNil(Alerts.deferPastQuietHours(
            send: date("2026-04-15T23:30:00"),
            event: date("2026-04-16T04:00:00"),
            options: options()
        ))
    }

    // MARK: Fenêtre de traitement

    func testFenetreAnnonceeQuandIlResteLeTemps() {
        let alerts = Alerts.evaluate(
            summary: summary(nextSpray: spray(4, 7)), hours: [],
            state: .empty, options: options()
        )
        XCTAssertEqual(alerts.map(\.kind), [.fenetre])
        XCTAssertEqual(alerts[0].params, ["score": 90])
        XCTAssertEqual(alerts[0].at, hourAt(4))
    }

    func testDansUneHeureOnSeTait() {
        let alerts = Alerts.evaluate(
            summary: summary(nextSpray: spray(1, 4)), hours: [],
            state: .empty, options: options()
        )
        XCTAssertTrue(alerts.isEmpty)
    }

    func testDansTroisJoursIlEstTropTot() {
        let alerts = Alerts.evaluate(
            summary: summary(nextSpray: spray(72, 75)), hours: [],
            state: .empty, options: options()
        )
        XCTAssertTrue(alerts.isEmpty)
    }

    func testLeDomaineRenvoieDesClesPasDesPhrases() {
        let alerts = Alerts.evaluate(
            summary: summary(nextSpray: spray(4, 7)), hours: [],
            state: .empty, options: options()
        )
        XCTAssertEqual(alerts[0].titleKey, "alert.fenetre.title")
        XCTAssertEqual(alerts[0].bodyKey, "alert.fenetre.body")
        XCTAssertFalse(alerts[0].titleKey.contains(" "))
    }

    // MARK: Gel

    func testGelAnnonceAvecLaTemperature() {
        let alerts = Alerts.evaluate(
            summary: summary(frost: FrostRisk(severity: .modere, minTemperature: -2.4, hoarFrost: true)),
            hours: [hour(0, temperature: 3), hour(10, temperature: -1)],
            state: .empty, options: options()
        )
        XCTAssertEqual(alerts.map(\.kind), [.gel])
        XCTAssertEqual(alerts[0].at, hourAt(10))
        XCTAssertEqual(alerts[0].params, ["temperature": -2.4])
    }

    func testPasDeGelPasDAlerte() {
        XCTAssertTrue(Alerts.evaluate(
            summary: summary(), hours: [], state: .empty, options: options()
        ).isEmpty)
    }

    // MARK: Sol devenu portant

    func testCEstLePassageQuiCompte() {
        var state = AlertState.empty
        state.lastSoilState = .sature
        let alerts = Alerts.evaluate(summary: summary(), hours: [], state: state, options: options())
        XCTAssertEqual(alerts.map(\.kind), [.sol])
    }

    func testSansEtatPrecedentOnSeTait() {
        // Annoncer « le sol est ressuyé » à quelqu'un dont le sol l'est depuis
        // un mois est du bruit.
        XCTAssertTrue(Alerts.evaluate(
            summary: summary(), hours: [], state: .empty, options: options()
        ).isEmpty)
    }

    func testSolDejaRessuyeNeRedeclencheRien() {
        var state = AlertState.empty
        state.lastSoilState = .ressuye
        XCTAssertTrue(Alerts.evaluate(
            summary: summary(), hours: [], state: state, options: options()
        ).isEmpty)
    }

    // MARK: Pluie lavante

    func testPluieAnnonceeDansLesSixHeuresApresLaFenetre() {
        let alerts = Alerts.evaluate(
            summary: summary(nextSpray: spray(4, 7)),
            hours: [hour(8, precipitation: 2.4)],
            state: .empty, options: options()
        )
        XCTAssertEqual(alerts.map(\.kind), [.fenetre, .pluie])
        XCTAssertEqual(alerts[1].params, ["rain": 2.4])
    }

    func testBruineSousLeSeuilNeComptePas() {
        let alerts = Alerts.evaluate(
            summary: summary(nextSpray: spray(4, 7)),
            hours: [hour(8, precipitation: 0.05)],
            state: .empty, options: options()
        )
        XCTAssertEqual(alerts.map(\.kind), [.fenetre])
    }

    func testPluieBienApresNeLaveRien() {
        let alerts = Alerts.evaluate(
            summary: summary(nextSpray: spray(4, 7)),
            hours: [hour(20, precipitation: 4)],
            state: .empty, options: options()
        )
        XCTAssertEqual(alerts.map(\.kind), [.fenetre])
    }

    // MARK: Délai de garde

    func testAlerteDejaPartieNeRepartPas() {
        let state = Alerts.recordSent(
            .empty,
            [Alert(kind: .fenetre, at: now, titleKey: "", bodyKey: "", params: [:])],
            at: now.addingTimeInterval(-3600)
        )
        XCTAssertTrue(Alerts.evaluate(
            summary: summary(nextSpray: spray(4, 7)), hours: [], state: state, options: options()
        ).isEmpty)
    }

    func testPasseLeDelaiElleRepart() {
        let state = Alerts.recordSent(
            .empty,
            [Alert(kind: .fenetre, at: now, titleKey: "", bodyKey: "", params: [:])],
            at: now.addingTimeInterval(-7 * 3600)
        )
        XCTAssertEqual(Alerts.evaluate(
            summary: summary(nextSpray: spray(4, 7)), hours: [], state: state, options: options()
        ).count, 1)
    }

    func testUnDelaiNeBaillonnePasUneAutreNature() {
        let state = Alerts.recordSent(
            .empty,
            [Alert(kind: .fenetre, at: now, titleKey: "", bodyKey: "", params: [:])],
            at: now
        )
        let alerts = Alerts.evaluate(
            summary: summary(
                frost: FrostRisk(severity: .faible, minTemperature: -0.5, hoarFrost: false),
                nextSpray: spray(4, 7)
            ),
            hours: [], state: state, options: options()
        )
        XCTAssertEqual(alerts.map(\.kind), [.gel])
    }

    func testEtatSerialisableEtSansEffetDeBord() throws {
        let before = Alerts.recordSoil(.empty, .sature)
        let after = Alerts.recordSent(
            before,
            [Alert(kind: .gel, at: now, titleKey: "", bodyKey: "", params: [:])],
            at: now
        )

        XCTAssertNil(before.lastSent[AlertKind.gel.rawValue])
        XCTAssertEqual(after.lastSoilState, .sature)

        let round = try JSONDecoder().decode(AlertState.self, from: JSONEncoder().encode(after))
        XCTAssertEqual(round, after)
    }

    // MARK: Le calme est le cas normal

    func testJourneeSansRienADireNeDitRien() {
        XCTAssertTrue(Alerts.evaluate(
            summary: summary(), hours: [hour(0), hour(1), hour(2)],
            state: .empty, options: options()
        ).isEmpty)
    }
}
