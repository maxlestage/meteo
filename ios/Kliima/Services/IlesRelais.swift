import Foundation

/// L'inscription d'une île dynamique auprès du relais, pour qu'il la tienne à
/// l'heure.
///
/// Sans le relais, une activité en direct ne change que quand iOS réveille
/// l'application — au mieux toutes les demi-heures, parfois des heures plus
/// tard. Le relais, lui, connaît l'heure : à chaque heure qui commence, il
/// pousse la nouvelle heure et la suivante, par le service de notifications
/// d'Apple.
///
/// **Ce qui part.** Le jeton qu'Apple a remis pour cette activité, et la
/// maille de la parcelle — le point arrondi, comme partout où une parcelle
/// quitte l'appareil. Pas son nom, pas de compte : le relais n'a pas à savoir
/// qui suit quoi.
///
/// Tout est facultatif. Sans relais, sans poussée côté serveur, ou hors
/// réseau, l'île bascule quand même seule à l'heure pile, avec l'heure
/// suivante calculée d'avance ; elle ne reçoit simplement rien de neuf.
enum IlesRelais {

    /// Le jeton tel qu'Apple et le relais l'écrivent : en hexadécimal.
    static func hex(_ jeton: Data) -> String {
        jeton.map { String(format: "%02x", $0) }.joined()
    }

    /// Le corps de l'inscription. Séparé de l'appel pour être vérifiable.
    static func corps(jeton: Data, latitude: Double, longitude: Double) -> Data {
        let objet: [String: Any] = [
            "jeton": hex(jeton),
            "latitude": Position.snap(latitude),
            "longitude": Position.snap(longitude),
        ]
        return (try? JSONSerialization.data(withJSONObject: objet)) ?? Data()
    }

    static func inscrire(jeton: Data, latitude: Double, longitude: Double,
                         session: URLSession = .shared) async {
        guard let base = PlanGrant.relayURL else { return }
        var requete = URLRequest(url: base.appendingPathComponent("v1/activites"))
        requete.httpMethod = "POST"
        requete.setValue("application/json", forHTTPHeaderField: "Content-Type")
        requete.httpBody = corps(jeton: jeton, latitude: latitude, longitude: longitude)
        // La réponse ne change rien ici : un refus laisse l'île basculer seule.
        _ = try? await session.data(for: requete)
    }

    static func retirer(jeton: Data, session: URLSession = .shared) async {
        guard let base = PlanGrant.relayURL else { return }
        var requete = URLRequest(url: base.appendingPathComponent("v1/activites"))
        requete.httpMethod = "DELETE"
        requete.setValue("application/json", forHTTPHeaderField: "Content-Type")
        requete.httpBody = try? JSONSerialization.data(withJSONObject: ["jeton": hex(jeton)])
        _ = try? await session.data(for: requete)
    }
}
