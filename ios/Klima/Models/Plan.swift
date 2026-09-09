import Foundation

/// Les paliers d'abonnement.
///
/// Miroir de `core/src/plan.ts`. Toute règle ajoutée d'un côté se porte de
/// l'autre, avec les mêmes cas de test — comme les seuils agronomiques.
///
/// Un principe gouverne le découpage : **on ne coupe jamais la réponse du
/// jour.** Un agriculteur qui ouvre Klima pour savoir s'il traite cet
/// après-midi doit l'obtenir sans payer. Ce qui se facture, c'est l'échelle
/// (plusieurs parcelles) et l'anticipation (alertes, cumuls, recoupement).
///
/// Ce fichier ne connaît ni StoreKit, ni prix, ni boutique : il dit seulement
/// ce qu'un palier ouvre. L'achat est affaire d'interface.
enum Plan: String, CaseIterable, Codable, Sendable {
    case libre
    case pro
}

/// Ce qu'un palier peut ouvrir.
enum Feature: String, CaseIterable, Sendable {
    /// Comparer plusieurs instituts et afficher leur accord.
    case recoupement
    /// Être prévenu sans ouvrir l'application.
    case alertes
    /// Cumuls de pluie et de degrés-jours depuis une date choisie.
    case cumuls
    /// Export des conditions à l'heure d'un traitement.
    case registre

    /// Clé de catalogue expliquant pourquoi c'est fermé — une clé, pas une
    /// phrase : l'interface la traduit.
    var upgradeReasonKey: String { "plan.reason.\(rawValue)" }
}

struct PlanLimits: Equatable, Sendable {
    /// Nombre de parcelles suivies. `nil` quand il n'y a pas de limite.
    let parcelles: Int?
    /// Jours de prévision consultables.
    let jours: Int
    let features: Set<Feature>
}

extension Plan {
    var limits: PlanLimits {
        switch self {
        case .libre:
            return PlanLimits(parcelles: 1, jours: 7, features: [])
        case .pro:
            return PlanLimits(parcelles: nil, jours: 7, features: Set(Feature.allCases))
        }
    }

    /// Vrai si le palier ouvre cette fonction.
    func allows(_ feature: Feature) -> Bool {
        limits.features.contains(feature)
    }

    /// Vrai si le palier permet d'en suivre une de plus.
    func canAddParcelle(current: Int) -> Bool {
        guard let maximum = limits.parcelles else { return true }
        return current < maximum
    }
}

/// Ce qu'il advient des parcelles quand l'abonnement s'arrête.
///
/// Rien n'est effacé. Un métier saisonnier plus un abonnement mensuel donne un
/// cycle prévisible — résiliation à l'automne, retour au printemps — et un
/// abonné qui devrait ressaisir vingt parcelles ne revient pas. On rend les
/// parcelles excédentaires inaccessibles, jamais absentes.
struct ParcelleAccess<Element> {
    let readable: [Element]
    /// Conservées, mais hors du palier courant.
    let locked: [Element]
}

extension Plan {
    /// Sépare les parcelles lisibles de celles que le palier ferme, dans
    /// l'ordre de la liste : les premières restent lisibles.
    func partition<Element>(_ parcelles: [Element]) -> ParcelleAccess<Element> {
        guard let maximum = limits.parcelles, parcelles.count > maximum else {
            return ParcelleAccess(readable: parcelles, locked: [])
        }
        return ParcelleAccess(
            readable: Array(parcelles.prefix(maximum)),
            locked: Array(parcelles.dropFirst(maximum))
        )
    }
}
