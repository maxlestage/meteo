//! Prendre la position de la personne, plutôt que lui montrer Chartres.
//!
//! Miroir Rust de `core/src/position.ts` et de `ios/Kliima/Models/Position.swift`,
//! avec les mêmes cas de test. Deux règles, et elles tiennent ensemble :
//!
//! - **On ne demande qu'à défaut.** Une adresse partagée désigne une parcelle,
//!   et une parcelle déjà choisie a été choisie : aller chercher la position
//!   par-dessus reviendrait à défaire le geste de quelqu'un.
//! - **On arrondit avant d'en faire une parcelle.** La prévision est la même
//!   dans tout le carré ; le lien n'a pas à dire à deux mètres près où se
//!   tient la personne.

use crate::grid::snap;

/// D'où vient la parcelle affichée au démarrage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParcelleOrigin {
    /// L'adresse la nommait : un lien partagé, un favori.
    Adresse,
    /// La dernière consultée, retrouvée en mémoire.
    Memoire,
    /// Rien ne la désignait : c'est celle par défaut.
    Defaut,
}

/// Une parcelle : un nom et un point.
#[derive(Debug, Clone, PartialEq)]
pub struct Parcelle {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

/// Faut-il aller chercher la position au démarrage ?
pub fn locates_on_start(origin: ParcelleOrigin) -> bool {
    origin == ParcelleOrigin::Defaut
}

/// La parcelle d'une position, arrondie à la maille.
///
/// Le nom vient de l'interface : le domaine ne fabrique pas de phrases.
pub fn parcelle_from_position(name: &str, latitude: f64, longitude: f64) -> Parcelle {
    Parcelle {
        name: name.to_string(),
        latitude: snap(latitude),
        longitude: snap(longitude),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::CELL_DEGREES;

    #[test]
    fn faute_de_mieux_oui() {
        assert!(locates_on_start(ParcelleOrigin::Defaut));
    }

    // Les deux cas où il y a un choix derrière la parcelle affichée.
    #[test]
    fn une_adresse_partagee_ou_une_parcelle_choisie_ne_se_defont_pas() {
        assert!(!locates_on_start(ParcelleOrigin::Adresse));
        assert!(!locates_on_start(ParcelleOrigin::Memoire));
    }

    #[test]
    fn porte_le_nom_que_lui_donne_linterface() {
        let p = parcelle_from_position("Ma position", 48.4468, 1.4892);
        assert_eq!(p.name, "Ma position");
    }

    #[test]
    fn est_arrondie_a_la_maille_pas_au_metre_pres() {
        let p = parcelle_from_position("x", 48.44681234, 1.48923456);
        assert!((p.latitude - 48.44).abs() < 1e-9);
        assert!((p.longitude - 1.48).abs() < 1e-9);
    }

    #[test]
    fn deux_positions_du_meme_carre_donnent_la_meme_parcelle() {
        let a = parcelle_from_position("x", 48.4468, 1.4892);
        let b = parcelle_from_position("x", 48.4490, 1.4870);
        assert_eq!(a, b);
    }

    #[test]
    fn ne_deplace_jamais_de_plus_dune_demi_maille() {
        for (lat, lon) in [(48.4468, 1.4892), (-33.8688, 151.2093), (64.1466, -21.9426)] {
            let p = parcelle_from_position("x", lat, lon);
            assert!((p.latitude - lat).abs() <= CELL_DEGREES / 2.0 + 1e-9);
            assert!((p.longitude - lon).abs() <= CELL_DEGREES / 2.0 + 1e-9);
        }
    }
}
