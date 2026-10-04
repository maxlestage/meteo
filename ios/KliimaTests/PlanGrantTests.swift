import XCTest
@testable import Kliima

/// Ce que le relais accorde, et ce qu'on refuse d'en déduire.
///
/// L'appel réseau n'est pas testable ici — il n'y a pas de relais sous la
/// main. Ce qui l'est, et qui compte autant : la lecture de la réponse, qui
/// doit refuser tout ce qui n'est pas un « oui » franc, et la fabrication de
/// l'adresse interrogée, où un échappement manquant suffit à tout casser.
final class PlanGrantTests: XCTestCase {

    private func data(_ texte: String) -> Data { Data(texte.utf8) }

    override func tearDown() {
        SharedStore.saveCourriel("")
        super.tearDown()
    }

    func testUnOuiFrancAccordeLePalier() {
        XCTAssertEqual(PlanGrant.lire(data(#"{"plan":"pro"}"#)), .accorde)
    }

    func testUnNonFrancEstLeSeulRefus() {
        XCTAssertEqual(PlanGrant.lire(data(#"{"plan":"libre"}"#)), .refuse)
    }

    func testCeQuOnNeComprendPasNeRetireRien() {
        // Ni n'accorde rien. Un corps tronqué, une page d'erreur, un JSON
        // d'une autre forme : le relais n'a pas dit non, il a dit quelque
        // chose qu'on ne comprend pas. On ne retire pas un accès là-dessus —
        // sinon un creux de réseau coûte son palier à un testeur, et la perte
        // s'écrit sur le disque.
        let muets = [
            #"{"plan":"Pro"}"#,
            #"{"plan":""}"#,
            #"{"palier":"pro"}"#,
            #"{}"#,
            "pro",
            "<html>502 Bad Gateway</html>",
            "",
        ]
        for corps in muets {
            XCTAssertEqual(PlanGrant.lire(data(corps)), .injoignable, corps)
        }
    }

    func testAucunCorpsNAccordeLePalierParMegarde() {
        // L'autre moitié de la règle : le doute n'ouvre jamais rien.
        for corps in [#"{"plan":"libre"}"#, #"{}"#, "", "<html>502</html>"] {
            XCTAssertNotEqual(PlanGrant.lire(data(corps)), .accorde, corps)
        }
    }

    // MARK: Ce que le gabarit livré contient

    func testLeRelaisEstConfigureEtEnHttps() {
        // Une adresse publique n'est pas un secret : celle-là est versionnée,
        // sinon chaque archivage recommencerait le même réglage à la main.
        XCTAssertNotNil(PlanGrant.relayURL)
        XCTAssertEqual(PlanGrant.relayURL?.scheme, "https")
    }

    func testAucunCodeNEstLivreDansLeDepot() {
        // La règle, et pas une préférence : un code écrit dans `Info.plist` est
        // lisible par quiconque lit le dépôt, bien avant d'être extrait du
        // binaire. L'essai nominatif passe par l'adresse, qui n'a pas ce
        // défaut. Si ce test tombe, c'est qu'un secret a été commité.
        XCTAssertNil(PlanGrant.code)
    }

    // MARK: L'adresse d'essai

    func testUneAdresseSeGardeEtSeRetire() {
        SharedStore.saveCourriel("max@ferme.fr")
        XCTAssertEqual(SharedStore.loadCourriel(), "max@ferme.fr")
        XCTAssertEqual(PlanGrant.courriel, "max@ferme.fr")

        // Une saisie vide efface : c'est ainsi qu'on se retire de l'essai sans
        // réinstaller.
        SharedStore.saveCourriel("   ")
        XCTAssertNil(SharedStore.loadCourriel())
        XCTAssertNil(PlanGrant.courriel)
    }

    func testLesBlancsAutourNeComptentPas() {
        SharedStore.saveCourriel("  max@ferme.fr\n")
        XCTAssertEqual(SharedStore.loadCourriel(), "max@ferme.fr")
    }

    // MARK: L'adresse interrogée

    private let base = URL(string: "https://exemple.test")!

    func testSansRienAPresenterLAdresseNaPasDeQuery() {
        let url = PlanGrant.requete(base: base)
        XCTAssertEqual(url?.absoluteString, "https://exemple.test/v1/plan")
    }

    func testLAdresseVoyageSousCourriel() {
        let url = PlanGrant.requete(base: base, courriel: "max@ferme.fr")
        XCTAssertEqual(url?.absoluteString, "https://exemple.test/v1/plan?courriel=max@ferme.fr")
    }

    func testUneAdresseAEtiquetteEstEncodee() {
        // Le piège : une query se lit en form-urlencoded, où « + » vaut une
        // espace. Sans encodage, `max+ferme@ferme.fr` arrive au relais avec un
        // trou au milieu et ne correspond à aucune invitation.
        let url = PlanGrant.requete(base: base, courriel: "max+ferme@ferme.fr")
        XCTAssertEqual(
            url?.absoluteString,
            "https://exemple.test/v1/plan?courriel=max%2Bferme@ferme.fr"
        )
    }

    func testLesSeparateursDeQuerySontEncodes() {
        // Une saisie ne doit pas pouvoir ajouter un paramètre.
        let url = PlanGrant.requete(base: base, courriel: "a&code=x@ferme.fr")
        let query = url?.query ?? ""
        XCTAssertFalse(query.contains("&code="), query)
        XCTAssertTrue(query.contains("%26"), query)
    }

    func testLesDeuxPreuvesPeuventVoyagerEnsemble() {
        let url = PlanGrant.requete(base: base, code: "sillon", courriel: "max@ferme.fr")
        XCTAssertEqual(
            url?.absoluteString,
            "https://exemple.test/v1/plan?code=sillon&courriel=max@ferme.fr"
        )
    }
}
