import XCTest
@testable import Kliima

/// Les villes enregistrées : mêmes cas que `rust/klima-core/src/villes.rs`.
final class VillesTests: XCTestCase {

    private func ville(_ nom: String, _ latitude: Double, _ longitude: Double) -> Parcelle {
        Parcelle(name: nom, latitude: latitude, longitude: longitude)
    }

    private var paris: Parcelle { ville("Paris", 48.8566, 2.3522) }
    private var lyon: Parcelle { ville("Lyon", 45.764, 4.8357) }
    private var nantes: Parcelle { ville("Nantes", 47.2184, -1.5536) }

    func testDeuxNomsDansLaMemeMailleSontLaMemeVille() {
        XCTAssertTrue(Villes.memeVille(paris, ville("Paris 4e", 48.8546, 2.3577)))
        XCTAssertFalse(Villes.memeVille(paris, lyon))
    }

    func testLePalierLibreEnGardeUne() {
        var villes: [Parcelle] = []
        XCTAssertEqual(Villes.ajouter(&villes, paris, plan: .libre), .ajoutee)
        XCTAssertEqual(Villes.ajouter(&villes, lyon, plan: .libre), .limite)
        XCTAssertEqual(villes, [paris])
    }

    func testLePalierPayantEnGardeAutantQuOnVeut() {
        var villes: [Parcelle] = []
        for v in [paris, lyon, nantes] {
            XCTAssertEqual(Villes.ajouter(&villes, v, plan: .pro), .ajoutee)
        }
        XCTAssertEqual(villes.count, 3)
        XCTAssertEqual(villes[2].name, "Nantes", "dans l'ordre où on les ajoute")
    }

    func testUneVilleDejaLaNEstPasAjouteeDeuxFois() {
        var villes = [paris, lyon]
        XCTAssertEqual(Villes.ajouter(&villes, ville("Lyon 1er", 45.7610, 4.8330), plan: .pro), .dejaLa(1))
        XCTAssertEqual(villes.count, 2)
        // Même au plafond, une ville déjà là se dit « déjà là », pas « limite ».
        var une = [paris]
        XCTAssertEqual(Villes.ajouter(&une, paris, plan: .libre), .dejaLa(0))
    }

    func testLesVillesFermeesParUneResiliationComptent() {
        var villes = [paris, lyon, nantes]
        XCTAssertEqual(Villes.ajouter(&villes, ville("Brest", 48.3904, -4.4861), plan: .libre), .limite)
        let acces = Plan.libre.partition(villes)
        XCTAssertEqual(acces.readable, [paris])
        XCTAssertEqual(acces.locked, [lyon, nantes])
    }

    func testRetirerUneVilleLibereSaPlace() {
        var villes = [paris]
        Villes.retirer(&villes, ville("Paris 4e", 48.8546, 2.3577))
        XCTAssertTrue(villes.isEmpty)
        Villes.retirer(&villes, lyon)
        XCTAssertEqual(Villes.ajouter(&villes, lyon, plan: .libre), .ajoutee)
        XCTAssertEqual(Villes.position(villes, lyon), 0)
        XCTAssertNil(Villes.position(villes, paris))
    }
}
