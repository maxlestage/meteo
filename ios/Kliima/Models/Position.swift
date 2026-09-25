import Foundation

/// D'où vient la parcelle affichée au démarrage.
///
/// Miroir Swift de `ParcelleOrigin` (`core/src/position.ts`). L'iPhone n'a pas
/// d'adresse à lire : seules `memoire` et `defaut` s'y présentent. Le cas
/// `adresse` existe quand même, pour que les deux énumérations restent lisibles
/// l'une à côté de l'autre.
enum ParcelleOrigin {
    /// L'adresse la nommait : un lien partagé, un favori. (Web seulement.)
    case adresse
    /// La dernière consultée, retrouvée dans le groupe partagé.
    case memoire
    /// Rien ne la désignait : c'est celle par défaut.
    case defaut
}

/// Prendre la position de la personne, plutôt que lui montrer Chartres.
///
/// Miroir Swift de `core/src/position.ts`, avec les mêmes cas de test. Deux
/// règles, et elles tiennent ensemble :
///
/// - **On ne demande qu'à défaut.** Une parcelle déjà choisie a été choisie :
///   aller chercher la position par-dessus reviendrait à défaire le geste de
///   quelqu'un. Faute de choix, il n'y a rien à défaire.
/// - **On arrondit avant d'en faire une parcelle.** La parcelle est écrite dans
///   le groupe partagé et voyage jusqu'au widget ; elle n'a pas à dire à deux
///   mètres près où se tient la personne, alors que la prévision est la même
///   dans tout le carré.
enum Position {

    /// Côté de la maille, en degrés. Le même qu'en TypeScript (`CELL_DEGREES`)
    /// et que celui du relais : c'est la résolution des modèles les plus fins.
    static let cellDegrees = 0.02

    /// Décimales gardées après l'arrondi, pour une valeur stable.
    private static let precision = 1000.0

    /// Arrondit une coordonnée au centre de sa maille.
    ///
    /// `floor(x + 0,5)` et non `rounded()` : JavaScript arrondit les demis vers
    /// le haut, Swift les écarte de zéro. Les deux ne diffèrent que sur un demi
    /// exact, et sur un négatif — mais deux plateformes qui ne s'accordent pas
    /// sur un cas limite finissent par ne pas s'accorder sur une cellule de
    /// cache.
    static func snap(_ value: Double) -> Double {
        let cellules = (value / cellDegrees + 0.5).rounded(.down)
        return ((cellules * cellDegrees) * precision).rounded() / precision
    }

    /// Faut-il aller chercher la position au démarrage ?
    static func locatesOnStart(_ origin: ParcelleOrigin) -> Bool {
        origin == .defaut
    }

    /// La parcelle d'une position, arrondie à la maille.
    ///
    /// Le nom vient de l'interface : le domaine ne fabrique pas de phrases.
    static func parcelle(named name: String, latitude: Double, longitude: Double) -> Parcelle {
        Parcelle(name: name, latitude: snap(latitude), longitude: snap(longitude))
    }
}
