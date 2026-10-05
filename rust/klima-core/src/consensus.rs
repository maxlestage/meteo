//! Recoupement de plusieurs modèles de prévision.
//!
//! Une seule source donne une réponse ; plusieurs sources donnent une réponse
//! *et* une idée de sa fiabilité. Quand les modèles s'accordent, on peut
//! annoncer un chiffre sans réserve ; quand ils divergent, il faut le dire
//! plutôt que d'afficher une fausse précision.

use crate::providers::{ProviderOutcome, SourceReading};
use std::collections::HashSet;

/// Degré d'accord entre les sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agreement {
    Forte,
    Moyenne,
    Faible,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    /// Valeur retenue : la médiane, moins sensible qu'une moyenne à un modèle
    /// isolé.
    pub median: f64,
    pub min: f64,
    pub max: f64,
    /// Écart entre les extrêmes.
    pub spread: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Consensus {
    pub readings: Vec<SourceReading>,
    /// Fournisseurs ayant répondu, sur ceux qui ont été interrogés.
    pub providers_answered: usize,
    pub providers_queried: usize,
    pub temperature: Spread,
    pub precipitation: Spread,
    pub wind_speed: Spread,
    /// Vrai si tous les modèles s'accordent sur la présence ou l'absence de pluie.
    pub agree_on_rain: bool,
    pub agreement: Agreement,
}

/// Seuils de lecture de l'accord entre modèles.
pub mod thresholds {
    /// Écart de température en deçà duquel l'accord est jugé fort (°C).
    pub const STRONG_TEMPERATURE_SPREAD: f64 = 1.5;
    /// Au-delà, l'accord est jugé faible (°C).
    pub const WEAK_TEMPERATURE_SPREAD: f64 = 3.0;
    /// Pluie considérée comme annoncée à partir de ce cumul horaire (mm).
    pub const RAIN_THRESHOLD: f64 = 0.1;
}

use thresholds::*;

/// Recoupe les relevés.
///
/// Renvoie `None` s'il n'y a rien à comparer : un seul modèle ne fait pas un
/// consensus, et le dire vaut mieux que de le laisser croire.
pub fn consensus(
    readings: &[SourceReading],
    answered: usize,
    queried: usize,
) -> Option<Consensus> {
    if readings.len() < 2 {
        return None;
    }

    let temperature = spread(readings.iter().map(|r| r.temperature));
    let precipitation = spread(readings.iter().map(|r| r.precipitation));
    let wind_speed = spread(readings.iter().map(|r| r.wind_speed));

    let premier_pluvieux = readings[0].precipitation >= RAIN_THRESHOLD;
    let agree_on_rain =
        readings.iter().all(|r| (r.precipitation >= RAIN_THRESHOLD) == premier_pluvieux);

    let distincts: HashSet<_> = readings.iter().map(|r| r.source.provider).collect();

    Some(Consensus {
        readings: readings.to_vec(),
        providers_answered: if answered > 0 { answered } else { distincts.len() },
        providers_queried: if queried > 0 { queried } else { distincts.len() },
        agreement: agreement_from(temperature.spread, agree_on_rain),
        temperature,
        precipitation,
        wind_speed,
        agree_on_rain,
    })
}

/// Recoupe ce que les fournisseurs ont renvoyé, en gardant trace de ceux qui
/// n'ont rien pu dire : l'interface doit pouvoir annoncer « 5 sources sur 6 ».
pub fn consensus_from_outcomes(outcomes: &[ProviderOutcome]) -> Option<Consensus> {
    let readings: Vec<SourceReading> =
        outcomes.iter().flat_map(|o| o.readings.iter().cloned()).collect();
    let answered = outcomes.iter().filter(|o| !o.readings.is_empty()).count();
    consensus(&readings, answered, outcomes.len())
}

/// L'accord se juge d'abord sur la température — la variable la mieux prévue —
/// puis sur le désaccord franc que constitue « il pleut / il ne pleut pas ».
fn agreement_from(temperature_spread: f64, agree_on_rain: bool) -> Agreement {
    if temperature_spread > WEAK_TEMPERATURE_SPREAD {
        return Agreement::Faible;
    }
    if !agree_on_rain {
        return Agreement::Moyenne;
    }
    if temperature_spread <= STRONG_TEMPERATURE_SPREAD {
        Agreement::Forte
    } else {
        Agreement::Moyenne
    }
}

fn spread(values: impl Iterator<Item = f64>) -> Spread {
    let mut sorted: Vec<f64> = values.collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("pas de NaN dans un relevé"));
    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    Spread {
        median: round(median(&sorted), 1),
        min: round(min, 1),
        max: round(max, 1),
        spread: round(max - min, 1),
    }
}

fn median(sorted: &[f64]) -> f64 {
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[middle]
    } else {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    }
}

/// Arrondi à la manière de JavaScript — comme dans `ville`.
fn round(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor + 0.5).floor() / factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::OPEN_METEO_SOURCES;

    fn reading(index: usize, temperature: f64, precipitation: f64, wind_speed: f64) -> SourceReading {
        SourceReading {
            source: OPEN_METEO_SOURCES[index].clone(),
            temperature,
            precipitation,
            wind_speed,
        }
    }

    fn t(index: usize, temperature: f64) -> SourceReading {
        reading(index, temperature, 0.0, 10.0)
    }

    #[test]
    fn un_seul_modele_ne_fait_pas_un_consensus() {
        assert!(consensus(&[t(0, 18.0)], 0, 0).is_none());
        assert!(consensus(&[], 0, 0).is_none());
    }

    #[test]
    fn la_valeur_retenue_est_la_mediane_pas_la_moyenne() {
        // 18, 18,4, 19,2, 25 : la moyenne serait tirée par le modèle isolé.
        let r = consensus(&[t(0, 18.0), t(1, 18.4), t(2, 19.2), t(3, 25.0)], 0, 0).unwrap();
        assert_eq!(r.temperature.median, 18.8);
        assert_eq!(r.temperature.min, 18.0);
        assert_eq!(r.temperature.max, 25.0);
        assert_eq!(r.temperature.spread, 7.0);
    }

    #[test]
    fn mediane_dun_nombre_impair_de_modeles() {
        let r = consensus(&[t(0, 14.0), t(1, 15.0), t(2, 17.0)], 0, 0).unwrap();
        assert_eq!(r.temperature.median, 15.0);
    }

    #[test]
    fn modeles_serres_accord_fort() {
        let r = consensus(&[t(0, 18.0), t(1, 19.0)], 0, 0).unwrap();
        assert_eq!(r.temperature.spread, 1.0);
        assert!(r.agree_on_rain);
        assert_eq!(r.agreement, Agreement::Forte);
    }

    #[test]
    fn deux_degres_decart_accord_moyen() {
        let r = consensus(&[t(0, 18.0), t(1, 20.0)], 0, 0).unwrap();
        assert_eq!(r.agreement, Agreement::Moyenne);
    }

    #[test]
    fn plus_de_trois_degres_decart_accord_faible() {
        let r = consensus(&[t(0, 18.0), t(1, 22.0)], 0, 0).unwrap();
        assert_eq!(r.agreement, Agreement::Faible);
    }

    #[test]
    fn desaccord_sur_la_pluie_laccord_ne_peut_pas_etre_fort() {
        let r = consensus(
            &[reading(0, 18.0, 0.0, 10.0), reading(1, 18.5, 1.2, 10.0)],
            0,
            0,
        )
        .unwrap();
        assert!(r.temperature.spread < 1.5);
        assert!(!r.agree_on_rain);
        assert_eq!(r.agreement, Agreement::Moyenne);
    }

    #[test]
    fn une_trace_de_pluie_sous_le_seuil_ne_compte_pas_comme_un_desaccord() {
        let r = consensus(
            &[reading(0, 18.0, 0.0, 10.0), reading(1, 18.4, 0.05, 10.0)],
            0,
            0,
        )
        .unwrap();
        assert!(r.agree_on_rain);
    }

    #[test]
    fn tous_daccord_sur_la_pluie() {
        let r = consensus(
            &[
                reading(0, 14.0, 2.1, 10.0),
                reading(1, 14.5, 1.8, 10.0),
                reading(2, 14.2, 3.0, 10.0),
            ],
            0,
            0,
        )
        .unwrap();
        assert!(r.agree_on_rain);
        assert_eq!(r.agreement, Agreement::Forte);
    }

    #[test]
    fn le_vent_est_recoupe_comme_le_reste() {
        let r = consensus(
            &[reading(0, 18.0, 0.0, 12.0), reading(1, 18.0, 0.0, 24.0)],
            0,
            0,
        )
        .unwrap();
        assert_eq!(r.wind_speed.median, 18.0);
        assert_eq!(r.wind_speed.spread, 12.0);
    }

    #[test]
    fn retient_combien_de_fournisseurs_ont_repondu() {
        let r = consensus_from_outcomes(&[
            ProviderOutcome { provider_id: "a".into(), readings: vec![t(0, 18.0), t(1, 18.4)] },
            ProviderOutcome { provider_id: "b".into(), readings: vec![] },
            ProviderOutcome { provider_id: "c".into(), readings: vec![t(2, 18.6)] },
        ])
        .unwrap();
        assert_eq!(r.readings.len(), 3);
        assert_eq!(r.providers_answered, 2);
        assert_eq!(r.providers_queried, 3);
    }

    #[test]
    fn aucun_fournisseur_na_repondu() {
        assert!(
            consensus_from_outcomes(&[ProviderOutcome {
                provider_id: "a".into(),
                readings: vec![],
            }])
            .is_none()
        );
    }
}
