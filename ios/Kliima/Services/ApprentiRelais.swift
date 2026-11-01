import Foundation

/// L'apprenti, demandé au relais (`/v1/apprenti`).
///
/// C'est le relais qui compare la semaine passée — trois interrogations qu’il
/// garde six heures pour tout le monde. Sans relais configuré (`KliimaRelay`),
/// pas d'apprenti : les modèles votent tels quels.
enum ApprentiRelais {

    /// L'adresse de l'apprenti pour une ville, d'après celle du relais.
    static func adresse(relais: URL, _ parcelle: Parcelle) -> URL? {
        guard var parties = URLComponents(url: relais, resolvingAgainstBaseURL: false) else { return nil }
        let chemin = parties.path.hasSuffix("/") ? String(parties.path.dropLast()) : parties.path
        parties.path = chemin + "/v1/apprenti"
        parties.queryItems = [
            URLQueryItem(name: "lat", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "lon", value: String(format: "%.4f", parcelle.longitude)),
        ]
        return parties.url
    }

    /// Ce que l'apprenti sait des modèles autour d'une ville ; `nil` sans
    /// relais, sans aéroport pour juger, ou en cas de panne — on retombe alors
    /// sur ce qu'il avait appris la dernière fois pour cette ville. Ce qui est
    /// lu est recopié pour le widget et les alertes.
    static func lire(_ parcelle: Parcelle) async -> Apprentissage? {
        guard let relais = PlanGrant.relayURL, let url = adresse(relais: relais, parcelle),
              let resultat = try? await URLSession.shared.data(from: url)
        else { return SharedStore.loadApprentissage(pour: parcelle) }
        let (data, reponse) = resultat
        if let http = reponse as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
            return SharedStore.loadApprentissage(pour: parcelle)
        }
        guard let appris = Apprenti.decode(data) else { return nil }
        SharedStore.save(appris, pour: parcelle)
        return appris
    }
}
