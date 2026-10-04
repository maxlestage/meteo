import AuthenticationServices
import Foundation
import Security

/// Le compte : l'identité qu'Apple prouve, et la session qu'en tire le relais.
///
/// Deux questions distinctes, et ce fichier ne répond qu'à la première.
///
/// - **Qui est-ce ?** « Se connecter avec Apple » remet un jeton signé par
///   Apple, qui dit : cette personne contrôle cette adresse. Rien à taper, rien
///   à deviner — une adresse invitée ne suffit plus, il faut en être titulaire.
/// - **A-t-elle le palier ?** C'est le relais qui le dit, d'après sa liste
///   `KLIMA_PRO`. Retirer quelqu'un de la liste lui retire l'accès à la
///   question suivante, session ou pas.
///
/// Le jeton d'Apple ne vit que dix minutes. Le relais l'échange donc contre une
/// session qu'il signe lui-même, et c'est elle qu'on garde — dans le
/// trousseau, pas dans les réglages : c'est un titre d'accès, il ne voyage pas
/// avec une sauvegarde iCloud en clair et ne se lit pas d'une autre app.
///
/// Pas de mot de passe, pas de compte à créer, pas de base de données.
///
/// **Ce qui n'a jamais tourné.** Ce fichier n'a pas d'appareil pour
/// s'exécuter dans l'environnement où il a été écrit : il est relu, pas éprouvé.
enum Session {

    /// Ce que le relais a rendu à la connexion.
    struct Ouverte: Codable, Equatable {
        /// La session du relais, à présenter à chaque question.
        let jeton: String
        /// L'adresse qu'Apple a prouvée — éventuellement une adresse relais
        /// d'Apple, si la personne a choisi de masquer la sienne.
        let courriel: String
        /// L'identifiant stable qu'Apple donne à cette personne pour cette
        /// application : c'est lui qu'on interroge pour savoir si le compte a
        /// été révoqué depuis les réglages.
        let utilisateurApple: String
    }

    /// Pourquoi une connexion n'a pas abouti. L'interface en tire une phrase.
    enum Echec: Error, Equatable {
        /// Pas de relais configuré : il n'y a personne à qui se présenter.
        case sansRelais
        /// Le relais n'a pas de secret de session : les comptes y sont fermés.
        case comptesFermes
        /// Le relais n'a pas reconnu le jeton d'Apple.
        case refuse
        /// Apple n'a pas ouvert la connexion, avec son code d'erreur. Le
        /// code est montré tel quel : c'est lui qui dit si c'est l'appareil,
        /// le compte Apple ou la configuration de l'application.
        case apple(code: Int)
        /// Pas de réseau, ou une réponse qu'on ne sait pas lire.
        case injoignable
    }

    // MARK: Trousseau

    private static let service = "com.kliima.app.session"
    private static let compte = "session"

    /// La session enregistrée, s'il y en a une.
    static var courante: Ouverte? {
        var requete = base
        requete[kSecReturnData as String] = true
        requete[kSecMatchLimit as String] = kSecMatchLimitOne

        var resultat: AnyObject?
        guard SecItemCopyMatching(requete as CFDictionary, &resultat) == errSecSuccess,
              let data = resultat as? Data
        else { return nil }
        return try? JSONDecoder().decode(Ouverte.self, from: data)
    }

    static func enregistrer(_ session: Ouverte) {
        guard let data = try? JSONEncoder().encode(session) else { return }
        oublier()
        var ajout = base
        ajout[kSecValueData as String] = data
        // Lisible après le premier déverrouillage : un rafraîchissement en
        // arrière-plan doit pouvoir redemander le palier.
        ajout[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlock
        SecItemAdd(ajout as CFDictionary, nil)
    }

    static func oublier() {
        SecItemDelete(base as CFDictionary)
    }

    private static var base: [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: compte,
        ]
    }

    // MARK: Connexion

    /// Échange le jeton d'Apple contre une session du relais, et l'enregistre.
    static func ouvrir(
        jetonApple: String,
        utilisateurApple: String,
        session: URLSession = .shared
    ) async -> Result<Ouverte, Echec> {
        guard let base = PlanGrant.relayURL else { return .failure(.sansRelais) }

        var requete = URLRequest(url: base.appendingPathComponent("v1/session"))
        requete.httpMethod = "POST"
        requete.setValue("application/json", forHTTPHeaderField: "Content-Type")
        requete.httpBody = corps(jetonApple: jetonApple)

        do {
            let (data, response) = try await session.data(for: requete)
            guard let http = response as? HTTPURLResponse else { return .failure(.injoignable) }
            switch http.statusCode {
            case 200..<300:
                guard let ouverte = lire(data, utilisateurApple: utilisateurApple) else {
                    return .failure(.injoignable)
                }
                enregistrer(ouverte)
                return .success(ouverte)
            case 401: return .failure(.refuse)
            case 503: return .failure(.comptesFermes)
            default: return .failure(.injoignable)
            }
        } catch {
            return .failure(.injoignable)
        }
    }

    /// Le corps de la demande. Séparé de l'appel pour être vérifiable.
    static func corps(jetonApple: String) -> Data {
        (try? JSONEncoder().encode(["jetonApple": jetonApple])) ?? Data()
    }

    /// Lit la réponse du relais : `{"session": "…", "courriel": "…"}`.
    static func lire(_ data: Data, utilisateurApple: String) -> Ouverte? {
        struct Reponse: Decodable { let session: String; let courriel: String }
        guard let reponse = try? JSONDecoder().decode(Reponse.self, from: data),
              !reponse.session.isEmpty, !reponse.courriel.isEmpty
        else { return nil }
        return Ouverte(jeton: reponse.session, courriel: reponse.courriel,
                       utilisateurApple: utilisateurApple)
    }

    // MARK: Révocation

    /// Oublie la session si la personne a retiré l'accès à Kliima ‣ depuis
    /// Réglages › Identifiant Apple › Se connecter avec Apple.
    ///
    /// Sans cette vérification, la session survivrait six mois à un retrait
    /// explicite. Le relais, lui, ne peut pas le savoir : c'est l'appareil
    /// qu'Apple renseigne.
    static func verifierAupresDApple() async {
        guard let session = courante else { return }
        let etat = try? await ASAuthorizationAppleIDProvider()
            .credentialState(forUserID: session.utilisateurApple)
        switch etat {
        case .some(.revoked), .some(.notFound): oublier()
        // Autorisé, transféré, ou Apple injoignable : on ne retire rien sur
        // une réponse qu'on n'a pas eue — un silence n'est pas un refus.
        default: break
        }
    }
}
