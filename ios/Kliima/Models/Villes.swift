import Foundation

/// Les villes enregistrées : une liste, et ce que le palier permet d'y mettre.
///
/// Miroir de `rust/klima-core/src/villes.rs`, avec les mêmes cas de test.
/// Toute règle ajoutée d'un côté se porte de l'autre.
///
/// Le palier libre en garde une, Kliima ‣ Pro autant qu'on veut
/// (`Plan.limits`). Quand l'abonnement s'arrête, rien n'est effacé : les
/// villes au-delà de la limite sont fermées, pas perdues (`Plan.partition`).
///
/// Deux villes sont la même quand elles tombent dans la même maille de la
/// prévision : « Paris » et « Paris 4e » donnent la même météo.
enum Villes {

    /// Ce qu'il advient d'une ville qu'on enregistre.
    enum Ajout: Equatable {
        /// Ajoutée en fin de liste.
        case ajoutee
        /// Déjà là, à cette place : rien n'a changé.
        case dejaLa(Int)
        /// Le palier n'en permet pas une de plus : rien n'a changé.
        case limite

        var code: String {
            switch self {
            case .ajoutee: return "ajoutee"
            case .dejaLa: return "dejaLa"
            case .limite: return "limite"
            }
        }
    }

    /// Vrai si les deux villes tombent dans la même maille.
    static func memeVille(_ a: Parcelle, _ b: Parcelle) -> Bool {
        Position.snap(a.latitude) == Position.snap(b.latitude)
            && Position.snap(a.longitude) == Position.snap(b.longitude)
    }

    /// La place d'une ville dans la liste, si elle y est.
    static func position(_ villes: [Parcelle], _ ville: Parcelle) -> Int? {
        villes.firstIndex { memeVille($0, ville) }
    }

    /// Enregistre une ville, si le palier le permet. Les villes fermées par une
    /// résiliation comptent : elles sont gardées pour le retour de
    /// l'abonnement.
    static func ajouter(_ villes: inout [Parcelle], _ ville: Parcelle, plan: Plan) -> Ajout {
        if let place = position(villes, ville) {
            return .dejaLa(place)
        }
        guard plan.canAddParcelle(current: villes.count) else {
            return .limite
        }
        villes.append(ville)
        return .ajoutee
    }

    /// Retire une ville. Rien ne se passe si elle n'y est pas.
    static func retirer(_ villes: inout [Parcelle], _ ville: Parcelle) {
        villes.removeAll { memeVille($0, ville) }
    }
}
