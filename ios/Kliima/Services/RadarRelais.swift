import Foundation

/// Le radar, demandé au relais (`/v1/radar`).
///
/// La mosaïque européenne pèse trois mégaoctets toutes les cinq minutes : le
/// relais la lit pour tout le monde, l'iPhone n'en reçoit que le résumé de sa
/// ville. Sans relais configuré (`KliimaRelay`), pas de radar.
enum RadarRelais {

    /// L'adresse du radar pour une ville, d'après celle du relais.
    static func adresse(relais: URL, _ parcelle: Parcelle) -> URL? {
        guard var parties = URLComponents(url: relais, resolvingAgainstBaseURL: false) else { return nil }
        let chemin = parties.path.hasSuffix("/") ? String(parties.path.dropLast()) : parties.path
        parties.path = chemin + "/v1/radar"
        parties.queryItems = [
            URLQueryItem(name: "lat", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "lon", value: String(format: "%.4f", parcelle.longitude)),
        ]
        return parties.url
    }

    /// Ce que voit le radar pour une ville ; `nil` sans relais ou en cas de
    /// panne.
    static func lire(_ parcelle: Parcelle, session: URLSession = .shared) async -> RadarPrevision? {
        guard let relais = PlanGrant.relayURL, let url = adresse(relais: relais, parcelle),
              let resultat = try? await session.data(from: url)
        else { return nil }
        let (data, reponse) = resultat
        if let http = reponse as? HTTPURLResponse, !(200..<300).contains(http.statusCode) { return nil }
        return Radar.decode(data)
    }
}
