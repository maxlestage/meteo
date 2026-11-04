import XCTest
@testable import Kliima

/// L'apprenti : mêmes cas que `rust/klima-core/src/apprenti.rs` et
/// `rust/klima-api/src/apprenti.rs`.
final class ApprentiTests: XCTestCase {

    private let h: TimeInterval = 3600
    private func t(_ heures: Double) -> Date { Date(timeIntervalSince1970: heures * 3600) }

    private func releves(_ n: Int, _ temperature: Double) -> [ApprentiReleve] {
        (0..<n).map { ApprentiReleve(time: t(Double($0)), temperature: temperature, mouille: $0 % 4 == 0) }
    }

    private func annonces(_ n: Int, _ temperature: Double) -> [ApprentiAnnonce] {
        (0..<n).map { ApprentiAnnonce(time: t(Double($0)), temperature: temperature, precipitation: $0 % 4 == 0 ? 1 : 0) }
    }

    private func note(_ id: String = "x", n: Int, biais: Double, ecart: Double? = nil) -> ApprentiNote {
        ApprentiNote(sourceId: id, n: n, biais: biais, ecart: ecart ?? abs(biais), pluieN: 0, pluieJustes: 0)
    }

    func testUnModeleQuiVoitTropChaudAUnBiaisPositif() {
        let n = Apprenti.noter("gfs", annonces: annonces(48, 12), releves: releves(48, 10), maintenant: t(100))
        XCTAssertEqual(n.n, 48)
        XCTAssertEqual(n.biais, 2)
        XCTAssertEqual(n.ecart, 2)
        XCTAssertEqual(n.pluieN, 48)
        XCTAssertEqual(n.pluieJustes, 48)
        XCTAssertEqual(n.justessePluie, 100)
    }

    func testLaCorrectionEstRetenuePuisBornee() {
        XCTAssertEqual(note(n: 23, biais: 2).correction, 0)
        XCTAssertEqual(note(n: 24, biais: 2).correction, 1)
        XCTAssertEqual(note(n: 168, biais: 2).correction, 1.8)
        XCTAssertEqual(note(n: 168, biais: 9).correction, 3)
        XCTAssertEqual(note(n: 168, biais: -9).correction, -3)
    }

    func testUnReleveHorsToleranceNeComptePas() {
        let annonces = [ApprentiAnnonce(time: t(10), temperature: 15, precipitation: nil)]
        let loin = [ApprentiReleve(time: t(10).addingTimeInterval(1200), temperature: 10, mouille: false)]
        XCTAssertEqual(Apprenti.noter("x", annonces: annonces, releves: loin, maintenant: t(20)).n, 0)
        let pres = [
            ApprentiReleve(time: t(10).addingTimeInterval(1800), temperature: 0, mouille: false),
            ApprentiReleve(time: t(10).addingTimeInterval(-300), temperature: 14, mouille: false),
        ]
        let n = Apprenti.noter("x", annonces: annonces, releves: pres, maintenant: t(20))
        XCTAssertEqual(n.n, 1)
        XCTAssertEqual(n.biais, 1)
    }

    func testLAvenirNeSeNotePas() {
        let n = Apprenti.noter("x", annonces: annonces(48, 12), releves: releves(48, 10), maintenant: t(9))
        XCTAssertEqual(n.n, 10)
    }

    func testLaPluieRateeSeCompte() {
        let annonces = [
            ApprentiAnnonce(time: t(0), temperature: nil, precipitation: 0),
            ApprentiAnnonce(time: t(1), temperature: nil, precipitation: 2),
        ]
        let releves = [
            ApprentiReleve(time: t(0), temperature: nil, mouille: true),
            ApprentiReleve(time: t(1), temperature: nil, mouille: true),
        ]
        let n = Apprenti.noter("x", annonces: annonces, releves: releves, maintenant: t(10))
        XCTAssertEqual([n.n, n.pluieN, n.pluieJustes], [0, 2, 1])
        XCTAssertEqual(n.justessePluie, 50)
    }

    func testCorrigerRedresseLaSourceNoteeEtElleSeule() {
        func serie(_ id: String) -> SerieSource {
            SerieSource(
                sourceId: id,
                heures: [HeureSource(time: t(0), temperature: 12, ressenti: 11)],
                jours: [JourSource(date: t(0), minimum: 8, maximum: 14)]
            )
        }
        let notes = [note("gfs", n: 168, biais: 2), note("icon", n: 10, biais: 2)]
        let r = Apprenti.corriger([serie("gfs"), serie("icon"), serie("met")], notes)
        XCTAssertEqual(r[0].heures[0].temperature!, 10.2, accuracy: 1e-9)
        XCTAssertEqual(r[0].heures[0].ressenti!, 9.2, accuracy: 1e-9)
        XCTAssertEqual(r[0].jours[0].minimum!, 6.2, accuracy: 1e-9)
        XCTAssertEqual(r[0].jours[0].maximum!, 12.2, accuracy: 1e-9)
        XCTAssertEqual(r[1].heures[0].temperature, 12)
        XCTAssertEqual(r[2].heures[0].temperature, 12)
    }

    func testLesSeuilsDeLApprentiSontInclusifs() {
        // Dix minutes pile d'un côté ou de l'autre : le relevé vaut encore.
        let annonce = [ApprentiAnnonce(time: t(10), temperature: 15, precipitation: nil)]
        func releve(_ dt: TimeInterval) -> [ApprentiReleve] {
            [ApprentiReleve(time: t(10).addingTimeInterval(dt), temperature: 14, mouille: false)]
        }
        let tolerance = ApprentiSeuils.toleranceS
        XCTAssertEqual(Apprenti.noter("x", annonces: annonce, releves: releve(tolerance), maintenant: t(20)).n, 1)
        XCTAssertEqual(Apprenti.noter("x", annonces: annonce, releves: releve(-tolerance), maintenant: t(20)).n, 1)
        XCTAssertEqual(Apprenti.noter("x", annonces: annonce, releves: releve(tolerance + 1), maintenant: t(20)).n, 0)
        // L'heure pile de maintenant est déjà passée.
        XCTAssertEqual(Apprenti.noter("x", annonces: annonce, releves: releve(0), maintenant: t(10)).n, 1)
        XCTAssertEqual(Apprenti.noter("x", annonces: annonce, releves: releve(0),
                                      maintenant: t(10).addingTimeInterval(-1)).n, 0)

        // 0,1 mm pile annoncé, c'est annoncer la pluie.
        func pluie(_ cumul: Double) -> [ApprentiAnnonce] { [ApprentiAnnonce(time: t(0), temperature: nil, precipitation: cumul)] }
        let mouille = [ApprentiReleve(time: t(0), temperature: nil, mouille: true)]
        XCTAssertEqual(Apprenti.noter("x", annonces: pluie(ApprentiSeuils.pluieMm), releves: mouille, maintenant: t(1)).pluieJustes, 1)
        XCTAssertEqual(Apprenti.noter("x", annonces: pluie(0.09), releves: mouille, maintenant: t(1)).pluieJustes, 0)

        // Vingt-quatre heures pile suffisent, et trois degrés pile passent.
        let min = ApprentiSeuils.relevesMin
        XCTAssertEqual(note(n: min, biais: 6).correction, ApprentiSeuils.correctionMax)
        XCTAssertEqual(note(n: min, biais: -6).correction, -ApprentiSeuils.correctionMax)
        XCTAssertEqual(note(n: 168, biais: .nan, ecart: 1).correction, 0, "un biais illisible ne corrige rien")
        XCTAssertEqual(Apprenti.meilleure([note(n: min, biais: 0, ecart: 0.5)])?.n, min)
        XCTAssertNil(Apprenti.meilleure([note(n: min - 1, biais: 0, ecart: 0.5)]))
    }

    func testLaMeilleureEstLaPlusJusteParmiLesAssezNotees() {
        let notes = [note("a", n: 100, biais: 0, ecart: 1.4), note("b", n: 10, biais: 0, ecart: 0.2), note("c", n: 100, biais: 0, ecart: 0.9)]
        XCTAssertEqual(Apprenti.meilleure(notes)?.sourceId, "c")
        XCTAssertNil(Apprenti.meilleure([note("b", n: 10, biais: 0, ecart: 0.2)]))
    }

    func testLaReponseDuRelaisSeLit() throws {
        let fil = Data(#"{"station":"LFPO","nom":"Paris/Orly Fld","jours":7,"notes":[{"id":"gfs_seamless","n":160,"biais":1.21,"ecart":1.62,"pluie_n":166,"pluie_justes":150}]}"#.utf8)
        let a = try XCTUnwrap(Apprenti.decode(fil))
        XCTAssertEqual(a.station, "LFPO")
        XCTAssertEqual(a.notes.first?.sourceId, "gfs_seamless")
        XCTAssertEqual(a.notes.first?.biais, 1.21)
        XCTAssertEqual(a.notes.first?.pluieJustes, 150)
        // Sans juge : rien.
        XCTAssertNil(Apprenti.decode(Data(#"{"station":null,"notes":[]}"#.utf8)))
        XCTAssertNil(Apprenti.decode(Data("[]".utf8)))
    }

    func testLAdresseDeLApprentiSuitCelleDuRelais() throws {
        let url = try XCTUnwrap(ApprentiRelais.adresse(relais: URL(string: "https://k.example/")!, Parcelle.paris))
        XCTAssertTrue(url.absoluteString.hasPrefix("https://k.example/v1/apprenti?lat="), url.absoluteString)
    }
}
