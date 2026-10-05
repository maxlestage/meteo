//! Les villes enregistrées : une liste, et ce que le palier permet d'y mettre.
//!
//! Miroir Swift : `ios/Kliima/Models/Villes.swift`, avec les mêmes cas de
//! test. Toute règle ajoutée d'un côté se porte de l'autre.
//!
//! Le palier libre en garde une, le palier payant autant qu'on veut
//! (`plan::limits_for`). Quand l'abonnement s'arrête, rien n'est effacé : les
//! villes au-delà de la limite sont fermées, pas perdues
//! (`plan::partition_parcelles`), et reviennent telles quelles.
//!
//! Deux villes sont la même quand elles tombent dans la même maille de la
//! prévision : « Paris » et « Paris 4e » donnent la même météo, les garder
//! toutes les deux ne serait qu'un doublon dans la liste.

use crate::grid::cell_for;
use crate::plan::{Plan, can_add_parcelle};
use crate::position::Parcelle;

/// Vrai si les deux villes tombent dans la même maille.
pub fn meme_ville(a: &Parcelle, b: &Parcelle) -> bool {
    cell_for(a.latitude, a.longitude) == cell_for(b.latitude, b.longitude)
}

/// La place d'une ville dans la liste, si elle y est.
pub fn position(villes: &[Parcelle], ville: &Parcelle) -> Option<usize> {
    villes.iter().position(|v| meme_ville(v, ville))
}

/// Ce qu'il advient d'une ville qu'on enregistre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ajout {
    /// Ajoutée en fin de liste.
    Ajoutee,
    /// Déjà là, à cette place : rien n'a changé.
    DejaLa(usize),
    /// Le palier n'en permet pas une de plus : rien n'a changé.
    Limite,
}

impl Ajout {
    pub fn code(&self) -> &'static str {
        match self {
            Ajout::Ajoutee => "ajoutee",
            Ajout::DejaLa(_) => "dejaLa",
            Ajout::Limite => "limite",
        }
    }
}

/// Enregistre une ville, si le palier le permet.
///
/// Les villes fermées par une résiliation comptent : elles sont gardées pour
/// le retour de l'abonnement, et en ajouter une par-dessus les repousserait
/// hors de portée sans qu'on l'ait demandé.
pub fn ajouter(villes: &mut Vec<Parcelle>, ville: Parcelle, plan: Plan) -> Ajout {
    if let Some(place) = position(villes, &ville) {
        return Ajout::DejaLa(place);
    }
    if !can_add_parcelle(plan, villes.len()) {
        return Ajout::Limite;
    }
    villes.push(ville);
    Ajout::Ajoutee
}

/// Retire une ville. Rien ne se passe si elle n'y est pas.
pub fn retirer(villes: &mut Vec<Parcelle>, ville: &Parcelle) {
    villes.retain(|v| !meme_ville(v, ville));
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::partition_parcelles;

    fn ville(nom: &str, latitude: f64, longitude: f64) -> Parcelle {
        Parcelle { name: nom.to_owned(), latitude, longitude, admin: None, country: None }
    }

    fn paris() -> Parcelle { ville("Paris", 48.8566, 2.3522) }
    fn lyon() -> Parcelle { ville("Lyon", 45.764, 4.8357) }
    fn nantes() -> Parcelle { ville("Nantes", 47.2184, -1.5536) }

    #[test]
    fn deux_noms_dans_la_meme_maille_sont_la_meme_ville() {
        assert!(meme_ville(&paris(), &ville("Paris 4e", 48.8546, 2.3577)));
        assert!(!meme_ville(&paris(), &lyon()));
    }

    #[test]
    fn le_palier_libre_en_garde_une() {
        let mut villes = Vec::new();
        assert_eq!(ajouter(&mut villes, paris(), Plan::Libre), Ajout::Ajoutee);
        assert_eq!(ajouter(&mut villes, lyon(), Plan::Libre), Ajout::Limite);
        assert_eq!(villes, [paris()]);
    }

    #[test]
    fn le_palier_payant_en_garde_autant_qu_on_veut() {
        let mut villes = Vec::new();
        for v in [paris(), lyon(), nantes()] {
            assert_eq!(ajouter(&mut villes, v, Plan::Pro), Ajout::Ajoutee);
        }
        assert_eq!(villes.len(), 3);
        assert_eq!(villes[2].name, "Nantes", "dans l'ordre où on les ajoute");
    }

    #[test]
    fn une_ville_deja_la_n_est_pas_ajoutee_deux_fois() {
        let mut villes = vec![paris(), lyon()];
        assert_eq!(ajouter(&mut villes, ville("Lyon 1er", 45.7610, 4.8330), Plan::Pro), Ajout::DejaLa(1));
        assert_eq!(villes.len(), 2);
        // Même au plafond, une ville déjà là se dit « déjà là », pas « limite ».
        let mut une = vec![paris()];
        assert_eq!(ajouter(&mut une, paris(), Plan::Libre), Ajout::DejaLa(0));
    }

    #[test]
    fn les_villes_fermees_par_une_resiliation_comptent() {
        let mut villes = vec![paris(), lyon(), nantes()];
        assert_eq!(ajouter(&mut villes, ville("Brest", 48.3904, -4.4861), Plan::Libre), Ajout::Limite);
        let acces = partition_parcelles(Plan::Libre, &villes);
        assert_eq!(acces.readable, [paris()]);
        assert_eq!(acces.locked, [lyon(), nantes()]);
    }

    #[test]
    fn retirer_une_ville_libere_sa_place() {
        let mut villes = vec![paris()];
        retirer(&mut villes, &ville("Paris 4e", 48.8546, 2.3577));
        assert!(villes.is_empty());
        retirer(&mut villes, &lyon());
        assert_eq!(ajouter(&mut villes, lyon(), Plan::Libre), Ajout::Ajoutee);
        assert_eq!(position(&villes, &lyon()), Some(0));
        assert_eq!(position(&villes, &paris()), None);
    }
}
