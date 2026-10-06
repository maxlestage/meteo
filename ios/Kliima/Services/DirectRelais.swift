import Foundation

/// Le direct : la météo que le relais pousse par WebSocket, dès qu'elle change.
///
/// Miroir de `rust/klima-ui/src/crochets/direct.rs`, sur le même protocole
/// (`rust/klima-relay/src/direct.rs`). L'application s'abonne à la ville
/// affichée ; le relais pousse sept sujets — `base`, `ensemble`, `quarts`,
/// `met`, `station`, `air`, `ciel` —, chacun dans la forme exacte que le fournisseur
/// renvoie : les décodeurs du service les lisent comme une réponse de requête.
///
/// Le direct s'ajoute aux requêtes, il ne les remplace pas : la première
/// prévision arrive par HTTP, et si la connexion tombe — l'application passée
/// en arrière-plan, un tunnel —, elle se rouvre après deux secondes, puis
/// quatre, huit… jusqu'à une minute. Sans relais configuré (`KliimaRelay`),
/// rien ne s'ouvre. Le widget, la montre et les tâches de fond n'ont pas de
/// connexion qui dure : eux restent sur les requêtes.
@MainActor
final class DirectRelais {

    /// Un message du relais.
    struct Message: Equatable {
        let sujet: String
        let latitude: Double
        let longitude: Double
        /// Le corps tel que le fournisseur l'a renvoyé.
        let corps: Data
    }

    /// Battement : sans lui, un routeur coupe une connexion silencieuse.
    static let battement: UInt64 = 25_000_000_000

    private let session: URLSession
    private var boucle: Task<Void, Never>?

    init(session: URLSession = .shared) {
        self.session = session
    }

    /// L'adresse du direct, d'après celle du relais (`https` → `wss`).
    static func adresse(relais: URL) -> URL? {
        guard var parties = URLComponents(url: relais, resolvingAgainstBaseURL: false) else { return nil }
        switch parties.scheme {
        case "https": parties.scheme = "wss"
        case "http": parties.scheme = "ws"
        default: return nil
        }
        let chemin = parties.path.hasSuffix("/") ? String(parties.path.dropLast()) : parties.path
        parties.path = chemin + "/v1/direct"
        return parties.url
    }

    /// L'abonnement, tel que le relais l'attend.
    static func abonnement(_ parcelle: Parcelle, jours: Int) -> String {
        "{\"latitude\":\(parcelle.latitude),\"longitude\":\(parcelle.longitude),\"jours\":\(jours)}"
    }

    /// Lit un message : son sujet, son point, et son corps.
    static func lire(_ texte: String) -> Message? {
        guard
            let objet = try? JSONSerialization.jsonObject(with: Data(texte.utf8)) as? [String: Any],
            let sujet = objet["sujet"] as? String,
            let latitude = (objet["latitude"] as? NSNumber)?.doubleValue,
            let longitude = (objet["longitude"] as? NSNumber)?.doubleValue,
            let corps = objet["corps"],
            JSONSerialization.isValidJSONObject(corps),
            let donnees = try? JSONSerialization.data(withJSONObject: corps)
        else { return nil }
        return Message(sujet: sujet, latitude: latitude, longitude: longitude, corps: donnees)
    }

    /// L'attente avant la `n`-ième reconnexion, en secondes.
    static func attente(essai: Int) -> Double {
        min(pow(2, Double(min(essai, 6))), 60)
    }

    /// Suit une ville : ouvre la connexion, s'abonne, et rend chaque message.
    /// Un nouvel appel remplace le précédent — changer de ville, c'est suivre
    /// la nouvelle.
    func suivre(_ parcelle: Parcelle, jours: Int, recevoir: @escaping @MainActor (Message) -> Void) {
        arreter()
        guard let relais = PlanGrant.relayURL, let adresse = Self.adresse(relais: relais) else { return }
        let abonnement = Self.abonnement(parcelle, jours: jours)
        let session = session

        boucle = Task { @MainActor in
            var essai = 0
            while !Task.isCancelled {
                let tache = session.webSocketTask(with: adresse)
                tache.resume()
                let battements = Task {
                    while !Task.isCancelled {
                        try? await Task.sleep(nanoseconds: Self.battement)
                        tache.sendPing { _ in }
                    }
                }
                do {
                    try await tache.send(.string(abonnement))
                    while !Task.isCancelled {
                        let recu = try await tache.receive()
                        essai = 0
                        let texte: String?
                        switch recu {
                        case .string(let s): texte = s
                        case .data(let d): texte = String(data: d, encoding: .utf8)
                        @unknown default: texte = nil
                        }
                        if let message = texte.flatMap(Self.lire) {
                            recevoir(message)
                        }
                    }
                } catch {
                    // La connexion est tombée : on la rouvre, un peu plus tard
                    // à chaque échec.
                }
                battements.cancel()
                tache.cancel(with: .goingAway, reason: nil)
                guard !Task.isCancelled else { break }
                essai += 1
                try? await Task.sleep(nanoseconds: UInt64(Self.attente(essai: essai) * 1_000_000_000))
            }
        }
    }

    func arreter() {
        boucle?.cancel()
        boucle = nil
    }
}
