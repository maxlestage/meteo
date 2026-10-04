//! Les paliers d'abonnement.
//!
//! Miroir Swift : `ios/Kliima/Models/Plan.swift`.
//!
//! Un principe gouverne le découpage : **on ne coupe jamais la réponse du
//! jour.** Un agriculteur qui ouvre Klima pour savoir s'il traite cet
//! après-midi doit l'obtenir sans payer. Ce qui se facture, c'est l'échelle
//! (plusieurs parcelles) et l'anticipation (alertes, cumuls, recoupement).
//! Amputer aujourd'hui rendrait le palier libre inutile, donc l'application
//! invendable.
//!
//! Ce module ne connaît ni StoreKit, ni prix, ni boutique : il dit seulement
//! ce qu'un palier ouvre. Le paiement est affaire d'interface, la règle est
//! affaire de domaine — et c'est ce qui permet de la tester des trois côtés.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Plan {
    Libre,
    Pro,
}

pub const PLANS: [Plan; 2] = [Plan::Libre, Plan::Pro];

impl Plan {
    pub fn code(self) -> &'static str {
        match self {
            Plan::Libre => "libre",
            Plan::Pro => "pro",
        }
    }

    /// Clé du nom affiché. « Klima » et « Klima Pro » — le nom de l'iPhone ne
    /// déborde pas ici.
    pub fn label_key(self) -> &'static str {
        match self {
            Plan::Libre => "plan.libre",
            Plan::Pro => "plan.pro",
        }
    }
}

/// Ce qu'un palier peut ouvrir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Feature {
    /// Comparer plusieurs instituts et afficher leur accord.
    Recoupement,
    /// Être prévenu sans ouvrir l'application.
    Alertes,
    /// Cumuls de pluie et de degrés-jours depuis une date choisie.
    Cumuls,
    /// Export des conditions à l'heure d'un traitement.
    Registre,
}

pub const FEATURES: [Feature; 4] =
    [Feature::Recoupement, Feature::Alertes, Feature::Cumuls, Feature::Registre];

impl Feature {
    pub fn code(self) -> &'static str {
        match self {
            Feature::Recoupement => "recoupement",
            Feature::Alertes => "alertes",
            Feature::Cumuls => "cumuls",
            Feature::Registre => "registre",
        }
    }

    /// Pourquoi la fonction est fermée — une clé, pas une phrase : l'interface
    /// la traduit, comme partout ailleurs dans le domaine.
    pub fn upgrade_reason_key(self) -> &'static str {
        match self {
            Feature::Recoupement => "plan.reason.recoupement",
            Feature::Alertes => "plan.reason.alertes",
            Feature::Cumuls => "plan.reason.cumuls",
            Feature::Registre => "plan.reason.registre",
        }
    }

    /// Ce que la fonction apporte, pour la page des paliers.
    pub fn feature_key(self) -> &'static str {
        match self {
            Feature::Recoupement => "plan.feature.recoupement",
            Feature::Alertes => "plan.feature.alertes",
            Feature::Cumuls => "plan.feature.cumuls",
            Feature::Registre => "plan.feature.registre",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanLimits {
    /// Nombre de parcelles suivies. `None` quand il n'y a pas de limite.
    pub parcelles: Option<usize>,
    /// Jours de prévision consultables.
    pub jours: u32,
    pub features: &'static [Feature],
}

/// Les limites d'un palier.
///
/// Un écart avec le TypeScript, et c'est le langage qui le demande : là-bas,
/// « pas de limite » s'écrit `Infinity`, un nombre qui se compare. Ici c'est
/// `None` — un entier n'a pas d'infini, et un `usize::MAX` déguisé en plafond
/// finit par être affiché à quelqu'un.
pub fn limits_for(plan: Plan) -> PlanLimits {
    match plan {
        Plan::Libre => PlanLimits { parcelles: Some(1), jours: 7, features: &[] },
        Plan::Pro => PlanLimits { parcelles: None, jours: 7, features: &FEATURES },
    }
}

/// Vrai si le palier ouvre cette fonction.
pub fn allows(plan: Plan, feature: Feature) -> bool {
    limits_for(plan).features.contains(&feature)
}

/// Vrai si le palier permet d'en suivre une de plus.
pub fn can_add_parcelle(plan: Plan, current: usize) -> bool {
    match limits_for(plan).parcelles {
        None => true,
        Some(limite) => current < limite,
    }
}

/// Ce qu'il advient des parcelles quand l'abonnement s'arrête.
///
/// Rien n'est effacé. Un métier saisonnier plus un abonnement mensuel donne un
/// cycle prévisible — résiliation à l'automne, retour au printemps — et un
/// abonné qui devrait ressaisir vingt parcelles ne revient pas. On rend les
/// parcelles excédentaires inaccessibles, jamais absentes : elles
/// réapparaissent telles quelles au réabonnement.
///
/// L'ordre est celui de la liste : les premières restent lisibles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParcelleAccess<T> {
    pub readable: Vec<T>,
    /// Conservées, mais hors du palier courant.
    pub locked: Vec<T>,
}

pub fn partition_parcelles<T: Clone>(plan: Plan, parcelles: &[T]) -> ParcelleAccess<T> {
    match limits_for(plan).parcelles {
        None => ParcelleAccess { readable: parcelles.to_vec(), locked: Vec::new() },
        Some(limite) => {
            let coupe = limite.min(parcelles.len());
            ParcelleAccess {
                readable: parcelles[..coupe].to_vec(),
                locked: parcelles[coupe..].to_vec(),
            }
        }
    }
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_palier_libre_donne_la_journee_entiere_sans_rien_amputer() {
        // La règle du modèle : on facture l'échelle et l'anticipation, jamais
        // la réponse du jour. Sept jours de prévision des deux côtés.
        assert_eq!(limits_for(Plan::Libre).jours, limits_for(Plan::Pro).jours);
        assert_eq!(limits_for(Plan::Libre).parcelles, Some(1));
    }

    #[test]
    fn le_palier_libre_nouvre_aucune_fonction_payante() {
        for feature in FEATURES {
            assert!(!allows(Plan::Libre, feature), "{}", feature.code());
        }
    }

    #[test]
    fn le_palier_pro_les_ouvre_toutes() {
        for feature in FEATURES {
            assert!(allows(Plan::Pro, feature), "{}", feature.code());
        }
    }

    #[test]
    fn la_deuxieme_parcelle_est_la_limite_du_palier_libre() {
        assert!(can_add_parcelle(Plan::Libre, 0));
        assert!(!can_add_parcelle(Plan::Libre, 1));
    }

    #[test]
    fn le_palier_pro_na_pas_de_plafond_de_parcelles() {
        assert!(can_add_parcelle(Plan::Pro, 0));
        assert!(can_add_parcelle(Plan::Pro, 500));
    }

    #[test]
    fn une_resiliation_verrouille_les_parcelles_elle_nen_perd_aucune() {
        let parcelles = ["Chartres", "Reims", "Toulouse"];
        let acces = partition_parcelles(Plan::Libre, &parcelles);

        assert_eq!(acces.readable, ["Chartres"]);
        assert_eq!(acces.locked, ["Reims", "Toulouse"]);

        // Rien n'a disparu : le réabonnement les rend telles quelles.
        let tout: Vec<&str> =
            acces.readable.iter().chain(acces.locked.iter()).copied().collect();
        assert_eq!(tout, parcelles);
    }

    #[test]
    fn au_retour_de_labonnement_tout_est_de_nouveau_lisible() {
        let parcelles = ["Chartres", "Reims", "Toulouse"];
        assert_eq!(
            partition_parcelles(Plan::Pro, &parcelles),
            ParcelleAccess { readable: parcelles.to_vec(), locked: Vec::new() }
        );
    }

    #[test]
    fn moins_de_parcelles_que_le_plafond_ne_verrouille_rien() {
        let acces = partition_parcelles(Plan::Libre, &["Chartres"]);
        assert_eq!(acces.readable, ["Chartres"]);
        assert!(acces.locked.is_empty());

        let vide: [&str; 0] = [];
        let acces = partition_parcelles(Plan::Libre, &vide);
        assert!(acces.readable.is_empty() && acces.locked.is_empty());
    }

    #[test]
    fn le_motif_de_blocage_est_une_cle_pas_une_phrase() {
        for feature in FEATURES {
            let cle = feature.upgrade_reason_key();
            assert_eq!(cle, format!("plan.reason.{}", feature.code()));
            assert!(!cle.contains(' '));
        }
    }

    #[test]
    fn chaque_palier_connu_a_ses_limites() {
        for plan in PLANS {
            assert!(limits_for(plan).jours > 0);
            assert!(limits_for(plan).parcelles.is_none_or(|p| p > 0));
        }
    }

    #[test]
    fn chaque_cle_de_palier_existe_dans_les_trois_langues() {
        use crate::i18n::LANGUAGES;
        use crate::messages::SHARED_MESSAGES;

        for language in LANGUAGES {
            for plan in PLANS {
                assert!(SHARED_MESSAGES.get(language, plan.label_key()).is_some());
            }
            for feature in FEATURES {
                assert!(SHARED_MESSAGES.get(language, feature.upgrade_reason_key()).is_some());
                assert!(SHARED_MESSAGES.get(language, feature.feature_key()).is_some());
            }
        }
    }
}
