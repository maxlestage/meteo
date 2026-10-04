import XCTest
@testable import Kliima

/// L'échange d'un jeton d'Apple contre une session du relais.
///
/// L'appel lui-même, et « Se connecter avec Apple », demandent un appareil et
/// un relais : ils ne se testent pas ici. Ce qui se teste, c'est ce qui part et
/// ce qu'on accepte en retour — les deux endroits où une faute passerait
/// inaperçue jusqu'au premier testeur.
final class SessionTests: XCTestCase {

    private func data(_ texte: String) -> Data { Data(texte.utf8) }

    func testLeCorpsPorteLeJetonDAppleSousLeNomQueLeRelaisAttend() throws {
        let corps = Session.corps(jetonApple: "eyJ.abc.def")
        let lu = try JSONSerialization.jsonObject(with: corps) as? [String: String]
        XCTAssertEqual(lu, ["jetonApple": "eyJ.abc.def"])
    }

    func testUneReponseCompleteOuvreLaSession() {
        let ouverte = Session.lire(
            data(#"{"session":"s.e.s","courriel":"max@ferme.fr"}"#),
            utilisateurApple: "000123.abc"
        )
        XCTAssertEqual(ouverte, Session.Ouverte(jeton: "s.e.s", courriel: "max@ferme.fr",
                                                utilisateurApple: "000123.abc"))
    }

    func testUneAdresseRelaisDAppleSeGardeTelleQuelle() {
        // Quand la personne masque son adresse, Apple en donne une à lui.
        // C'est celle-là qu'il faudra inviter : on ne la transforme pas.
        let ouverte = Session.lire(
            data(#"{"session":"s","courriel":"x7k2@privaterelay.appleid.com"}"#),
            utilisateurApple: "u"
        )
        XCTAssertEqual(ouverte?.courriel, "x7k2@privaterelay.appleid.com")
    }

    func testUneReponseIncompleteNOuvreRien() {
        for corps in [
            #"{"session":"s"}"#,
            #"{"courriel":"max@ferme.fr"}"#,
            #"{"session":"","courriel":"max@ferme.fr"}"#,
            #"{"session":"s","courriel":""}"#,
            #"{"erreur":"refusé"}"#,
            "<html>502</html>",
            "",
        ] {
            XCTAssertNil(Session.lire(data(corps), utilisateurApple: "u"), corps)
        }
    }

    func testLaSessionSeRelitApresEncodage() throws {
        // Ce qui va au trousseau doit en revenir à l'identique.
        let ouverte = Session.Ouverte(jeton: "j", courriel: "max@ferme.fr", utilisateurApple: "u")
        let relue = try JSONDecoder().decode(Session.Ouverte.self,
                                             from: JSONEncoder().encode(ouverte))
        XCTAssertEqual(relue, ouverte)
    }
}
