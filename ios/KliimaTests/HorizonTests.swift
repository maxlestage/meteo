import XCTest
@testable import Kliima

/// L'heure qui court, la suivante, et la chronologie calculée d'avance.
///
/// C'est ce qui décide de ce que montrent l'île dynamique et les widgets quand
/// l'application est fermée : une erreur ici ne se voit qu'une heure plus tard,
/// sur l'écran verrouillé de quelqu'un.
final class HorizonTests: XCTestCase {

    /// 14 h 00 pile, en instant absolu.
    private let quatorze = Date(timeIntervalSince1970: TimeInterval(1_790_000_000 - 1_790_000_000 % 3600))

    private func heure(_ decalage: Int, temperature: Double = 18, debut: Date? = nil,
                       pluie: Double = 0) -> HourlySample {
        HourlySample(
            time: (debut ?? quatorze).addingTimeInterval(Double(decalage) * 3600),
            weatherCode: 3,
            isDay: true,
            precipitationProbability: 20,
            temperature: temperature,
            apparentTemperature: temperature - 1,
            relativeHumidity: 65,
            dewPoint: 11,
            precipitation: pluie,
            windSpeed: 8,
            windGusts: 14,
            uvIndex: 1
        )
    }

    private func serie(_ n: Int = 6, debut: Date? = nil) -> [HourlySample] {
        (0..<n).map { heure($0, temperature: 18 + Double($0), debut: debut) }
    }

    private func courant(_ temperature: Double = 17.5) -> CurrentSample {
        CurrentSample(time: quatorze, temperature: temperature, apparentTemperature: 16,
                      weatherCode: 1, isDay: true, relativeHumidity: 60, windSpeed: 9, windGusts: 15,
                      pressure: 1015)
    }

    // MARK: L'heure qui court

    func testAQuatorzeHeuresVingtCestLHeureDeQuatorzeHeures() {
        // Le défaut qu'on corrige : la première heure *après* l'instant
        // montrait quinze heures à quatorze heures vingt.
        let a1420 = quatorze.addingTimeInterval(20 * 60)
        XCTAssertEqual(Horizon.heure(contenant: a1420, dans: serie())?.time, quatorze)
    }

    func testAlHeurePileCestLaNouvelleHeure() {
        let quinze = quatorze.addingTimeInterval(3600)
        XCTAssertEqual(Horizon.heure(contenant: quinze, dans: serie())?.time, quinze)
    }

    func testApresLaFinDeLaSerieRienNestPresenteCommeActuel() {
        let tard = quatorze.addingTimeInterval(6 * 3600 + 60)
        XCTAssertNil(Horizon.heure(contenant: tard, dans: serie()))
    }

    func testLaSuivanteEstLHeureDApres() {
        let a1420 = quatorze.addingTimeInterval(20 * 60)
        XCTAssertEqual(Horizon.heureSuivante(apres: a1420, dans: serie())?.time,
                       quatorze.addingTimeInterval(3600))
    }

    func testLaBasculeEstLeProchainDebutDHeure() {
        let a1420 = quatorze.addingTimeInterval(20 * 60)
        XCTAssertEqual(Horizon.prochaineBascule(apres: a1420, dans: serie()),
                       quatorze.addingTimeInterval(3600))
    }

    func testUnFuseauALaDemieBasculeALaDemie() {
        // Une parcelle à UTC+5:30 : ses heures commencent à la demie. Un
        // arrondi à l'heure pleine ferait basculer trente minutes trop tôt.
        let demie = quatorze.addingTimeInterval(30 * 60)
        let a1440 = quatorze.addingTimeInterval(40 * 60)
        let heures = serie(debut: demie)
        XCTAssertEqual(Horizon.heure(contenant: a1440, dans: heures)?.time, demie)
        XCTAssertEqual(Horizon.prochaineBascule(apres: a1440, dans: heures),
                       demie.addingTimeInterval(3600))
    }

    // MARK: La chronologie

    func testLaChronologieCommenceMaintenantPuisUneEntreeParHeure() {
        let a1420 = quatorze.addingTimeInterval(20 * 60)
        let moments = Horizon.chronologie(courant: courant(), heures: serie(), jours: [],
                                          depuis: a1420, nombre: 3)
        XCTAssertEqual(moments.map(\.date), [
            a1420,
            quatorze.addingTimeInterval(3600),
            quatorze.addingTimeInterval(2 * 3600),
            quatorze.addingTimeInterval(3 * 3600),
        ])
    }

    func testLePremierMomentGardeLaMesureLesSuivantsLaPrevision() {
        let moments = Horizon.chronologie(courant: courant(17.5), heures: serie(), jours: [],
                                          depuis: quatorze.addingTimeInterval(60), nombre: 2)
        XCTAssertEqual(moments[0].courant.temperature, 17.5, "la mesure, plus fraîche")
        XCTAssertEqual(moments[1].courant.temperature, 19, "la prévision de quinze heures")
        XCTAssertEqual(moments[2].courant.temperature, 20)
    }

    func testChaqueMomentCommenceSaListeDHeuresParLaSienne() {
        let moments = Horizon.chronologie(courant: courant(), heures: serie(), jours: [],
                                          depuis: quatorze.addingTimeInterval(60), nombre: 2)
        XCTAssertEqual(moments[0].heures.first?.time, quatorze)
        XCTAssertEqual(moments[1].heures.first?.time, quatorze.addingTimeInterval(3600))
    }

    func testLaChronologieSArreteALaFinDeLaSerie() {
        let moments = Horizon.chronologie(courant: courant(), heures: serie(3), jours: [],
                                          depuis: quatorze.addingTimeInterval(60), nombre: 12)
        XCTAssertEqual(moments.count, 3, "maintenant, puis 15 h et 16 h — rien d'inventé après")
    }

    // MARK: Le jour embrasse ce qu'on affiche

    private func jour(_ date: Date) -> DailySample {
        DailySample(date: date, weatherCode: 3, temperatureMin: 9, temperatureMax: 13,
                    precipitationSum: 0, precipitationProbabilityMax: 0, windGustsMax: 0,
                    uvIndexMax: 0, sunrise: nil, sunset: nil)
    }

    private var minuit: Date { quatorze.addingTimeInterval(-14 * 3600) }

    func testLeJourEmbrasseLaTemperatureAffichee() throws {
        let jours = [jour(minuit), jour(minuit.addingTimeInterval(86_400))]
        var j = try XCTUnwrap(Horizon.jour(contenant: quatorze.addingTimeInterval(3600),
                                           embrassant: 14.2, dans: jours))
        XCTAssertEqual(j.temperatureMin, 9)
        XCTAssertEqual(j.temperatureMax, 14.2, "« 14° » sous « ↑ 13° »")
        j = try XCTUnwrap(Horizon.jour(contenant: quatorze, embrassant: 7.5, dans: jours))
        XCTAssertEqual(j.temperatureMin, 7.5)
        XCTAssertEqual(j.temperatureMax, 13)
    }

    func testLeJourNeRetrecitJamais() throws {
        let j = try XCTUnwrap(Horizon.jour(contenant: quatorze, embrassant: 11, dans: [jour(minuit)]))
        XCTAssertEqual(j.temperatureMin, 9)
        XCTAssertEqual(j.temperatureMax, 13)
    }

    func testSeulLeJourDeLInstantSElargit() throws {
        let jours = [jour(minuit), jour(minuit.addingTimeInterval(86_400))]
        // Demain quatorze heures : c'est demain qui s'élargit, pas aujourd'hui.
        let demain = try XCTUnwrap(Horizon.jour(contenant: quatorze.addingTimeInterval(86_400),
                                                embrassant: 20, dans: jours))
        XCTAssertEqual(demain.date, minuit.addingTimeInterval(86_400))
        XCTAssertEqual(demain.temperatureMax, 20)
        XCTAssertEqual(jours[0].temperatureMax, 13)
    }

    func testHorsDeLaSerieLeJourResteTelQuel() throws {
        let jours = [jour(minuit)]
        // La veille : le premier jour est rendu, mais pas élargi.
        let veille = try XCTUnwrap(Horizon.jour(contenant: minuit.addingTimeInterval(-3600),
                                                embrassant: 30, dans: jours))
        XCTAssertEqual(veille.temperatureMax, 13)
        // Le surlendemain : la série est finie.
        let apres = try XCTUnwrap(Horizon.jour(contenant: minuit.addingTimeInterval(30 * 3600),
                                               embrassant: 30, dans: jours))
        XCTAssertEqual(apres.temperatureMax, 13)
        XCTAssertNil(Horizon.jour(contenant: quatorze, embrassant: 20, dans: []))
        let nan = try XCTUnwrap(Horizon.jour(contenant: quatorze, embrassant: .nan, dans: jours))
        XCTAssertEqual(nan.temperatureMin, 9)
        XCTAssertEqual(nan.temperatureMax, 13)
    }

    /// Le défaut vu sur les widgets et la complication : l'entrée de dix-sept
    /// heures montrait 21° à côté d'un maximum de 13°.
    func testChaqueMomentDeLaChronologieEmbrasseSaTemperature() {
        let moments = Horizon.chronologie(courant: courant(17.5), heures: serie(), jours: [jour(minuit)],
                                          depuis: quatorze.addingTimeInterval(60), nombre: 4)
        for moment in moments {
            XCTAssertGreaterThanOrEqual(moment.jour?.temperatureMax ?? -.infinity, moment.courant.temperature)
            XCTAssertEqual(moment.jour?.temperatureMin, 9)
        }
        XCTAssertEqual(moments[0].jour?.temperatureMax, 17.5)
        XCTAssertEqual(moments[3].jour?.temperatureMax, 21)
    }

    // MARK: La pile intelligente

    /// Douze heures sèches, où il pleut à l'heure `pluie`.
    private func seriePluvieuse(_ pluie: Int?) -> [HourlySample] {
        (0..<12).map { heure($0, temperature: 18 + Double($0), pluie: $0 == pluie ? 1.2 : 0) }
    }

    func testIlPleutLaPileRemonteKliima() {
        XCTAssertEqual(Horizon.pertinence(seriePluvieuse(0), a: quatorze.addingTimeInterval(20 * 60)), 80)
    }

    func testUnePluieDansDeuxHeuresPesePlusQuUnCielSec() {
        let a1420 = quatorze.addingTimeInterval(20 * 60)
        XCTAssertEqual(Horizon.pertinence(seriePluvieuse(2), a: a1420), 60)
        XCTAssertEqual(Horizon.pertinence(seriePluvieuse(nil), a: a1420), 10)
    }

    func testUnePluieLointaineNeComptePas() {
        XCTAssertEqual(Horizon.pertinence(seriePluvieuse(3), a: quatorze), 10, "trois heures pile : trop loin")
        XCTAssertEqual(Horizon.pertinence(seriePluvieuse(6), a: quatorze), 10)
    }

    func testLesTroisHeuresSeComptentDepuisLEntree() {
        // Une entrée posée à 19 h, calculée à 14 h : une pluie à 20 h est à
        // une heure d'elle, pas à six.
        let serie = Array(seriePluvieuse(6).dropFirst(5))
        XCTAssertEqual(Horizon.pertinence(serie, a: quatorze.addingTimeInterval(5 * 3600)), 60)
    }
}
