//! Les cumuls depuis une date choisie.
//!
//! Miroir Swift : `ios/Kliima/Models/Cumuls.swift`.
//!
//! Un agriculteur ne raisonne pas en « sept derniers jours » mais depuis un
//! événement : le semis, le dernier traitement, la reprise de végétation. Ce
//! module additionne la pluie, l'évapotranspiration et les degrés-jours depuis
//! la date qu'on lui donne.
//!
//! Il dit toujours **ce qu'il a réellement couvert**. Une série de prévision
//! ne remonte pas dans le passé : demander le cumul depuis un semis d'octobre
//! à une série qui commence hier donnerait un chiffre faux, et un chiffre faux
//! dans un outil de décision est pire qu'une absence de chiffre. Le résultat
//! porte donc les bornes effectives et le nombre de jours manquants.
//!
//! Une différence assumée avec le TypeScript : là-bas, « minuit » est celui du
//! fuseau de la machine qui exécute. Ici, les horodatages sont déjà ceux de la
//! parcelle — c'est la règle du dépôt — et une journée est donc une simple
//! division. Deux serveurs dans deux fuseaux ne rendront plus deux cumuls.

use crate::agro::{DailySample, growing_degree_days, thresholds::GDD_CEILING};
use crate::calendar::at_midnight;

const DAY_MS: i64 = 86_400_000;

#[derive(Debug, Clone, PartialEq)]
pub struct Cumul {
    /// Date demandée, ramenée à son minuit.
    pub requested_from: i64,
    /// Première journée réellement disponible dans la série.
    pub from: i64,
    /// Dernière journée prise en compte.
    pub to: i64,
    /// Journées effectivement additionnées.
    pub days: usize,
    /// Journées demandées qui manquent à la série. Zéro quand la couverture
    /// est complète ; au-delà, le cumul est un minorant.
    pub missing_days: i64,
    pub precipitation: f64,
    pub evapotranspiration: f64,
    /// Pluie − ET0 (mm). Négatif : la parcelle a puisé dans sa réserve.
    pub balance: f64,
    pub gdd: f64,
}

impl Cumul {
    /// Vrai si le cumul couvre toute la période demandée.
    pub fn is_complete(&self) -> bool {
        self.missing_days == 0
    }
}

/// Additionne les journées depuis `from` incluse.
///
/// Renvoie `None` quand aucune journée de la série n'entre dans la période :
/// il n'y a alors rien d'honnête à afficher.
pub fn accumulate(days: &[DailySample], from: i64, base: f64) -> Option<Cumul> {
    let start = at_midnight(from);
    let kept: Vec<&DailySample> =
        days.iter().filter(|d| at_midnight(d.date) >= start).collect();
    if kept.is_empty() {
        return None;
    }

    let first = at_midnight(kept[0].date);
    let last = at_midnight(kept[kept.len() - 1].date);

    let precipitation: f64 = kept.iter().map(|d| d.precipitation_sum).sum();
    let evapotranspiration: f64 = kept.iter().map(|d| d.et0_sum).sum();
    let gdd: f64 = kept
        .iter()
        .map(|d| growing_degree_days(d.temperature_min, d.temperature_max, base, GDD_CEILING))
        .sum();

    Some(Cumul {
        requested_from: start,
        from: first,
        to: last,
        days: kept.len(),
        // Ce que la série ne couvre pas : les journées entre la demande et son début.
        missing_days: ((first - start) / DAY_MS).max(0),
        precipitation: round(precipitation),
        evapotranspiration: round(evapotranspiration),
        balance: round(precipitation - evapotranspiration),
        gdd: round(gdd),
    })
}

/// Arrondi au dixième, à la manière de JavaScript — voir `agro::round`.
fn round(value: f64) -> f64 {
    (value * 10.0 + 0.5).floor() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agro::thresholds::GDD_BASE;

    /// 10 avril 2026, minuit UTC.
    const AVRIL_10: i64 = 1_775_779_200_000;

    fn jour(decalage: i64, pluie: f64, et0: f64) -> DailySample {
        DailySample {
            date: AVRIL_10 + decalage * DAY_MS,
            weather_code: 3,
            temperature_min: 8.0,
            temperature_max: 20.0,
            precipitation_sum: pluie,
            precipitation_probability_max: 40.0,
            et0_sum: et0,
            wind_gusts_max: 30.0,
            sunrise: None,
            sunset: None,
        }
    }

    fn serie() -> Vec<DailySample> {
        vec![jour(0, 2.0, 3.0), jour(1, 5.0, 2.0), jour(2, 0.0, 4.0)]
    }

    #[test]
    fn additionne_pluie_et0_et_bilan_sur_la_periode() {
        let c = accumulate(&serie(), AVRIL_10, GDD_BASE).unwrap();
        assert_eq!(c.days, 3);
        assert_eq!(c.precipitation, 7.0);
        assert_eq!(c.evapotranspiration, 9.0);
        assert_eq!(c.balance, -2.0);
    }

    #[test]
    fn capitalise_les_degres_jours_au_dessus_de_la_base() {
        // (8 + 20) / 2 = 14 ; 14 − 10 = 4, trois fois.
        assert_eq!(accumulate(&serie(), AVRIL_10, GDD_BASE).unwrap().gdd, 12.0);
    }

    #[test]
    fn une_date_en_cours_de_serie_ne_compte_que_ce_qui_suit() {
        let c = accumulate(&serie(), AVRIL_10 + DAY_MS, GDD_BASE).unwrap();
        assert_eq!(c.days, 2);
        assert_eq!(c.precipitation, 5.0);
    }

    #[test]
    fn lheure_de_la_date_demandee_nexclut_pas_sa_journee() {
        // 11 avril à 18 h : la journée du 11 compte quand même.
        let c = accumulate(&serie(), AVRIL_10 + DAY_MS + 18 * 3_600_000, GDD_BASE).unwrap();
        assert_eq!(c.days, 2);
        assert_eq!(c.from, AVRIL_10 + DAY_MS);
    }

    #[test]
    fn une_demande_anterieure_a_la_serie_se_dit_incomplete() {
        let c = accumulate(&serie(), AVRIL_10 - 5 * DAY_MS, GDD_BASE).unwrap();
        assert_eq!(c.days, 3);
        assert_eq!(c.missing_days, 5);
        assert!(!c.is_complete());
        assert_eq!(c.requested_from, AVRIL_10 - 5 * DAY_MS);
        assert_eq!(c.from, AVRIL_10);
    }

    #[test]
    fn une_couverture_complete_le_dit_aussi() {
        assert!(accumulate(&serie(), AVRIL_10, GDD_BASE).unwrap().is_complete());
    }

    #[test]
    fn une_date_posterieure_a_la_serie_ne_renvoie_rien() {
        assert!(accumulate(&serie(), AVRIL_10 + 21 * DAY_MS, GDD_BASE).is_none());
    }

    #[test]
    fn une_serie_vide_ne_renvoie_rien() {
        assert!(accumulate(&[], AVRIL_10, GDD_BASE).is_none());
    }

    #[test]
    fn les_bornes_effectives_encadrent_ce_qui_a_ete_compte() {
        let c = accumulate(&serie(), AVRIL_10, GDD_BASE).unwrap();
        assert_eq!(c.from, AVRIL_10);
        assert_eq!(c.to, AVRIL_10 + 2 * DAY_MS);
    }

    #[test]
    fn une_base_de_degres_jours_differente_change_le_resultat() {
        // Base 6 : (8 + 20) / 2 = 14 ; 14 − 6 = 8, trois fois.
        assert_eq!(accumulate(&serie(), AVRIL_10, 6.0).unwrap().gdd, 24.0);
    }
}
