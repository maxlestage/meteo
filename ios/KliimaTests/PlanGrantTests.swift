import XCTest
@testable import Kliima

/// Ce que le relais accorde, et ce qu'on refuse d'en déduire.
///
/// L'appel réseau n'est pas testable ici — il n'y a pas de relais sous la
/// main. Ce qui l'est, et qui compte autant : la lecture de la réponse, qui
/// doit refuser tout ce qui n'est pas un « oui » franc.
final class PlanGrantTests: XCTestCase {

    private func data(_ texte: String) -> Data { Data(texte.utf8) }

    func testUnOuiFrancAccordeLePalier() {
        XCTAssertTrue(PlanGrant.lire(data(#"{"plan":"pro"}"#)))
    }

    func testToutLeResteNAccordeRien() {
        // Un relais en panne, une réponse tronquée, une page d'erreur : aucune
        // de ces choses ne doit ouvrir un palier payant.
        let refus = [
            #"{"plan":"libre"}"#,
            #"{"plan":"Pro"}"#,
            #"{"plan":""}"#,
            #"{"palier":"pro"}"#,
            #"{}"#,
            "pro",
            "<html>502 Bad Gateway</html>",
            "",
        ]
        for corps in refus {
            XCTAssertFalse(PlanGrant.lire(data(corps)), corps)
        }
    }

    func testSansRelaisConfigureAucunAppelNEstFait() {
        // Le gabarit livré a les deux clés vides : l'application se comporte
        // alors exactement comme avant que cette option existe.
        XCTAssertNil(PlanGrant.relayURL)
        XCTAssertNil(PlanGrant.code)
    }
}
