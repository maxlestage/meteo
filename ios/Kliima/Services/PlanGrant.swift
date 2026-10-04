import Foundation

/// Le palier que le déploiement accorde, en plus de ce que dit la boutique.
///
/// Pendant l'essai, personne n'achète : les testeurs doivent voir le palier
/// payant sans passer par l'App Store, et l'auteur doit pouvoir le rendre à
/// n'importe quel moment. La décision vit donc **sur le relais**, dans sa
/// variable `KLIMA_PRO` — pas ici.
///
/// C'est tout l'intérêt : une valeur glissée dans une application distribuée
/// est une valeur publiée, qu'on ne retire qu'en publiant une nouvelle
/// version. Celle-là s'enlève en une commande, et le palier redescend au
/// démarrage suivant.
///
/// Ce service n'accorde rien de lui-même et ne retient rien : sans relais
/// configuré, il ne fait même pas d'appel, et l'application se comporte
/// exactement comme avant.
struct PlanGrant {

    /// Adresse du relais, lue dans `Info.plist`. Vide : aucun appel.
    static var relayURL: URL? {
        guard
            let valeur = Bundle.main.object(forInfoDictionaryKey: "KliimaRelay") as? String,
            !valeur.trimmingCharacters(in: .whitespaces).isEmpty,
            let url = URL(string: valeur.trimmingCharacters(in: .whitespaces))
        else { return nil }
        return url
    }

    /// Code à présenter, quand le relais en demande un. Absent le plus souvent.
    static var code: String? {
        guard
            let valeur = Bundle.main.object(forInfoDictionaryKey: "KliimaProCode") as? String,
            !valeur.isEmpty
        else { return nil }
        return valeur
    }

    /// Demande au relais ce qu'il accorde.
    ///
    /// Renvoie `false` dès que quelque chose manque — pas de relais, pas de
    /// réseau, réponse illisible. Un relais muet ne doit jamais faire perdre
    /// un abonnement réel, ni en inventer un.
    static func accorde(session: URLSession = .shared) async -> Bool {
        guard let base = relayURL else { return false }

        var composants = URLComponents(url: base.appendingPathComponent("v1/plan"),
                                       resolvingAgainstBaseURL: false)
        if let code {
            composants?.queryItems = [URLQueryItem(name: "code", value: code)]
        }
        guard let url = composants?.url else { return false }

        do {
            let (data, response) = try await session.data(from: url)
            guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
                return false
            }
            return lire(data)
        } catch {
            return false
        }
    }

    /// Lit la réponse du relais : `{"plan":"pro"}` ou `{"plan":"libre"}`.
    ///
    /// Séparé de l'appel pour être vérifiable sans réseau.
    static func lire(_ data: Data) -> Bool {
        struct Reponse: Decodable { let plan: String }
        guard let reponse = try? JSONDecoder().decode(Reponse.self, from: data) else { return false }
        return reponse.plan == Plan.pro.rawValue
    }
}
