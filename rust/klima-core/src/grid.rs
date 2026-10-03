//! La maille de cache.
//!
//! Deux parcelles voisines partagent la même prévision : les modèles ne
//! distinguent pas deux points séparés de moins de leur propre résolution. On
//! arrondit donc les coordonnées à une maille, et tout ce qui tombe dans la
//! même cellule partage une seule interrogation du fournisseur.
//!
//! La maille vaut 0,02° — environ 2,2 km en latitude, 1,5 km en longitude à la
//! latitude de la France. C'est la résolution des modèles les plus fins qu'on
//! interroge (AROME à 1,3 km, ICON-D2 à 2 km) : arrondir plus grossièrement
//! ferait perdre de la précision à un outil dont c'est justement l'argument.
//!
//! La même maille sert une seconde fois : quand on prend la position de
//! quelqu'un, on l'arrondit avant d'en faire une parcelle. Une parcelle finit
//! dans l'adresse d'une page, et une adresse se partage.

/// Côté de la maille, en degrés.
pub const CELL_DEGREES: f64 = 0.02;

/// Décimales gardées après l'arrondi, pour une clé stable.
const PRECISION: f64 = 1000.0;

/// Arrondit une coordonnée au centre de sa maille.
///
/// `floor(x + 0,5)` et non `round` : JavaScript arrondit les demis vers le
/// haut, Rust les écarte de zéro. Les deux ne diffèrent que sur un demi exact
/// et sur un négatif — mais deux implémentations qui ne s'accordent pas sur un
/// cas limite finissent par ne pas s'accorder sur une cellule de cache.
pub fn snap(value: f64) -> f64 {
    let cells = (value / CELL_DEGREES + 0.5).floor();
    ((cells * CELL_DEGREES) * PRECISION).round() / PRECISION
}

/// La cellule dans laquelle tombe un point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub latitude: f64,
    pub longitude: f64,
}

/// La cellule d'un point.
pub fn cell_for(latitude: f64, longitude: f64) -> Cell {
    Cell { latitude: snap(latitude), longitude: snap(longitude) }
}

/// Clé de cache d'une cellule, pour un usage donné.
pub fn cell_key(usage: &str, cell: &Cell) -> String {
    format!("{usage}:{},{}", cell.latitude, cell.longitude)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deux_points_de_la_meme_cellule_donnent_la_meme_cle() {
        // Chartres et un point à quelques centaines de mètres, du même côté de
        // la frontière de maille : une seule interrogation pour les deux.
        let a = cell_for(48.4468, 1.4892);
        let b = cell_for(48.449, 1.487);
        assert_eq!(cell_key("forecast", &a), cell_key("forecast", &b));
    }

    #[test]
    fn deux_points_proches_peuvent_tomber_de_part_et_dautre() {
        // Comportement assumé : découper en mailles crée des frontières. Le
        // coût est une interrogation de plus, jamais une réponse fausse.
        let a = cell_for(48.4468, 1.4892);
        let b = cell_for(48.4512, 1.4892);
        assert_ne!(cell_key("forecast", &a), cell_key("forecast", &b));
    }

    /// Valeurs relevées sur l'implémentation TypeScript, un point par
    /// hémisphère. C'est ce qui garantit l'accord entre les trois écritures.
    #[test]
    fn snap_saccorde_avec_typescript_et_swift() {
        let cas = [
            (48.44681234, 48.44),
            (1.48923456, 1.48),
            (-33.8688, -33.86),
            (151.2093, 151.2),
            (0.0001, 0.0),
            (64.1466, 64.14),
            (-21.9426, -21.94),
        ];
        for (entree, attendu) in cas {
            assert!((snap(entree) - attendu).abs() < 1e-9, "snap({entree}) = {}", snap(entree));
        }
    }

    #[test]
    fn ne_deplace_jamais_de_plus_dune_demi_maille() {
        for point in [48.4468, -33.8688, 0.0001, 64.1466, -21.9426, 151.2093] {
            assert!((snap(point) - point).abs() <= CELL_DEGREES / 2.0 + 1e-9);
        }
    }
}
