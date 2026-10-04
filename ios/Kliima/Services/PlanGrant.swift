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
/// Ce que l'application présente, elle, n'est pas un secret : c'est l'adresse
/// que le testeur a saisie, qu'il connaît déjà et que le relais compare à la
/// liste des invités. Rien à embarquer, rien à publier, et se retirer de
/// l'essai ne demande ni nouvelle version ni désinstallation.
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

    /// Code à présenter, quand le relais en demande un.
    ///
    /// Vide dans le dépôt, et destiné à le rester : un code écrit ici est
    /// lisible par quiconque lit le dépôt, bien avant d'être extrait du
    /// binaire. L'essai nominatif passe par l'adresse ci-dessous, qui n'a pas
    /// ce défaut.
    static var code: String? {
        guard
            let valeur = Bundle.main.object(forInfoDictionaryKey: "KliimaProCode") as? String,
            !valeur.isEmpty
        else { return nil }
        return valeur
    }

    /// Adresse saisie par le testeur, s'il en a saisi une.
    static var courriel: String? { SharedStore.loadCourriel() }

    /// L'adresse à interroger. Séparée de l'appel pour être vérifiable sans
    /// réseau : c'est là que se jouent l'échappement et le nom des paramètres.
    ///
    /// L'échappement n'est pas un détail. Une query se lit en form-urlencoded,
    /// où « + » vaut une espace : `max+ferme@ferme.fr` posé tel quel arrive au
    /// relais avec un trou au milieu et ne correspond à rien. `URLQueryItem`
    /// laisse passer le « + » — c'est permis dans une query — donc on encode
    /// nous-mêmes, et les adresses à étiquette cessent d'être un piège.
    static func requete(base: URL, code: String? = nil, courriel: String? = nil) -> URL? {
        var composants = URLComponents(url: base.appendingPathComponent("v1/plan"),
                                       resolvingAgainstBaseURL: false)

        var parametres: [URLQueryItem] = []
        if let code { parametres.append(URLQueryItem(name: "code", value: echapper(code))) }
        if let courriel {
            parametres.append(URLQueryItem(name: "courriel", value: echapper(courriel)))
        }
        // Sans paramètre, pas de « ? » esseulé en fin d'adresse.
        composants?.percentEncodedQueryItems = parametres.isEmpty ? nil : parametres
        return composants?.url
    }

    /// Encode une valeur pour une query, « + » compris.
    private static func echapper(_ valeur: String) -> String {
        var permis = CharacterSet.urlQueryAllowed
        // Les cinq qui changeraient le sens de la query plutôt que d'en être.
        permis.remove(charactersIn: "+&=?#")
        return valeur.addingPercentEncoding(withAllowedCharacters: permis) ?? valeur
    }

    /// Ce que le relais a répondu — et le fait qu'il n'ait rien répondu.
    ///
    /// La distinction est tout l'objet de ce type. Un « non » et un silence
    /// mènent au même palier à la première question, mais pas à la seconde :
    /// un refus retire un accès, un silence ne doit rien retirer du tout.
    /// Les confondre, c'est faire perdre son palier à un testeur qui passe
    /// sous un tunnel, et écrire cette perte sur le disque.
    enum Verdict {
        case accorde
        case refuse
        /// Pas de relais, pas de réseau, ou une réponse qu'on ne sait pas lire.
        case injoignable
    }

    /// Demande au relais ce qu'il accorde.
    static func verdict(session: URLSession = .shared) async -> Verdict {
        guard
            let base = relayURL,
            let url = requete(base: base, code: code, courriel: courriel)
        else { return .injoignable }

        do {
            let (data, response) = try await session.data(from: url)
            guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
                return .injoignable
            }
            return lire(data)
        } catch {
            return .injoignable
        }
    }

    /// Lit la réponse du relais : `{"plan":"pro"}` ou `{"plan":"libre"}`.
    ///
    /// Séparé de l'appel pour être vérifiable sans réseau.
    ///
    /// Seul un `libre` bien formé vaut un refus. Un corps tronqué, une page
    /// d'erreur, un JSON d'une autre forme : le relais n'a pas dit non, il a
    /// dit quelque chose qu'on ne comprend pas — et on ne retire pas un accès
    /// sur une phrase qu'on n'a pas comprise. Accorder, en revanche, demande
    /// toujours un « oui » franc : le doute n'ouvre jamais rien.
    static func lire(_ data: Data) -> Verdict {
        struct Reponse: Decodable { let plan: String }
        guard let reponse = try? JSONDecoder().decode(Reponse.self, from: data) else {
            return .injoignable
        }
        switch reponse.plan {
        case Plan.pro.rawValue: return .accorde
        case Plan.libre.rawValue: return .refuse
        default: return .injoignable
        }
    }
}
