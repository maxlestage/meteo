import XCTest
@testable import Kliima

/// Ce que le relais accorde, et ce qu'on refuse d'en déduire.
///
/// L'appel réseau n'est pas testable ici — il n'y a pas de relais sous la
/// main. Ce qui l'est, et qui compte autant : la lecture de la réponse, qui
/// doit refuser tout ce qui n'est pas un « oui » franc, et la fabrication de
/// la demande, où un échappement manquant ou un en-tête oublié suffit à tout
/// casser.
final class PlanGrantTests: XCTestCase {

    private func data(_ texte: String) -> Data { Data(texte.utf8) }

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
        // binaire. L'essai nominatif passe par les comptes, qui n'ont pas ce
        // défaut. Si ce test tombe, c'est qu'un secret a été commité.
        XCTAssertNil(PlanGrant.code)
    }

    // MARK: La demande

    private let base = URL(string: "https://exemple.test")!

    func testSansRienAPresenterLaDemandeEstNue() {
        let requete = PlanGrant.demande(base: base)
        XCTAssertEqual(requete?.url?.absoluteString, "https://exemple.test/v1/plan")
        XCTAssertNil(requete?.value(forHTTPHeaderField: "Authorization"))
    }

    func testLaSessionVoyageDansLEnTeteEtPasDansLAdresse() {
        // Une adresse finit dans les journaux d'un proxy ; un en-tête beaucoup
        // moins. La session ne doit jamais apparaître dans l'URL.
        let requete = PlanGrant.demande(base: base, session: "abc.def.ghi")
        XCTAssertEqual(requete?.value(forHTTPHeaderField: "Authorization"), "Bearer abc.def.ghi")
        XCTAssertEqual(requete?.url?.absoluteString, "https://exemple.test/v1/plan")
        XCTAssertFalse(requete?.url?.absoluteString.contains("abc") ?? true)
    }

    func testUneSessionVideNePoseAucunEnTete() {
        let requete = PlanGrant.demande(base: base, session: "")
        XCTAssertNil(requete?.value(forHTTPHeaderField: "Authorization"))
    }

    func testUnCodeEstEncodePlusCompris() {
        // Le piège : « + » vaut une espace dans une query.
        let requete = PlanGrant.demande(base: base, code: "a+b&c=d")
        let query = requete?.url?.query ?? ""
        XCTAssertTrue(query.contains("%2B"), query)
        XCTAssertFalse(query.contains("&c="), query)
    }
}
