//! L'heure qui court, celle qui vient — prises dans une série horaire.
//!
//! Miroir Swift : `ios/Kliima/Models/Horizon.swift`, avec les mêmes cas de
//! test. L'iPhone s'en sert pour l'île dynamique et ses widgets ; le relais,
//! pour calculer ce qu'il pousse à une île quand l'heure tourne.
//!
//! Les bornes viennent de la série elle-même, jamais d'un arrondi à l'heure
//! pleine : une parcelle à UTC+5:30 a ses heures à la demie, et un arrondi
//! ferait basculer l'affichage trente minutes trop tôt.
//!
//! Les instants sont ceux du reste du cœur : des millisecondes à l'heure de la
//! parcelle. L'appelant compare donc avec « maintenant à la parcelle ».

use crate::meteo::{DailySample, HourlySample};
use crate::ville::{prochaine_pluie, Pluie};

const HEURE_MS: i64 = 3_600_000;

/// L'heure de la série qui contient `instant`.
///
/// Avant le début de la série, la première heure ; après sa fin, rien — mieux
/// vaut ne rien montrer qu'une heure passée présentée comme actuelle.
pub fn heure_contenant(instant: i64, heures: &[HourlySample]) -> Option<&HourlySample> {
    match heures.iter().rev().find(|h| h.time <= instant) {
        Some(courante) => (instant < courante.time + HEURE_MS).then_some(courante),
        None => heures.first(),
    }
}

/// L'heure qui suit celle qui contient `instant`.
pub fn heure_suivante(instant: i64, heures: &[HourlySample]) -> Option<&HourlySample> {
    let courante = heure_contenant(instant, heures)?;
    heures.iter().find(|h| h.time > courante.time)
}

/// Le prochain début d'heure de la série, strictement après `instant` : le
/// moment où ce qui est affiché cesse d'être vrai.
pub fn prochaine_bascule(instant: i64, heures: &[HourlySample]) -> Option<i64> {
    heures.iter().find(|h| h.time > instant).map(|h| h.time)
}

/// Le jour de la série qui contient `instant`.
pub fn jour_contenant(instant: i64, jours: &[DailySample]) -> Option<&DailySample> {
    jours.iter().rev().find(|j| j.date <= instant).or_else(|| jours.first())
}

/// Le jour qui contient `instant`, élargi jusqu'à `temperature` — celle qu'on
/// affiche pour cet instant.
///
/// Un widget posé pour quinze heures montre la température prévue à quinze
/// heures à côté du minimum et du maximum du jour. Les deux ne viennent pas
/// des mêmes voix — l'heure est la médiane des températures horaires, le jour
/// celle des extrêmes publiés — et l'on pouvait lire « 14° » puis « 9° / 13° ».
/// Ce qu'il fait à cet instant est atteint ce jour-là : le jour s'élargit
/// jusqu'à lui, il ne rétrécit jamais. Même règle que
/// `fusion::embrasser_l_instant`, appliquée à chaque instant affiché.
pub fn jour_embrassant(instant: i64, temperature: f64, jours: &[DailySample]) -> Option<DailySample> {
    let mut jour = jour_contenant(instant, jours)?.clone();
    // Hors du jour trouvé (avant la série, ou après sa fin), rien à embrasser :
    // la température d'un autre jour ne dit rien de celui-ci.
    let dedans = jour.date <= instant && instant < jour.date + 24 * HEURE_MS;
    if dedans && temperature.is_finite() {
        jour.temperature_min = jour.temperature_min.min(temperature);
        jour.temperature_max = jour.temperature_max.max(temperature);
    }
    Some(jour)
}

/// Le poids d'un moment dans la pile intelligente : plus haut quand la pluie
/// approche, pour que la pile remonte Kliima ‣ d'elle-même au moment où l'on
/// en a besoin. `heures` commence à l'heure du moment ; les trois heures se
/// comptent depuis `instant`, l'heure de l'entrée — pas depuis le moment où
/// la chronologie est calculée.
pub fn pertinence(instant: i64, heures: &[HourlySample]) -> u8 {
    match prochaine_pluie(heures) {
        Pluie::EnCours { .. } => pertinence::PLUIE_EN_COURS,
        Pluie::Prevue { debut, .. } if debut - instant < pertinence::PLUIE_DANS_MS => {
            pertinence::PLUIE_PROCHE
        }
        _ => pertinence::SEC,
    }
}

/// Les poids de la pile intelligente.
pub mod pertinence {
    /// Il pleut.
    pub const PLUIE_EN_COURS: u8 = 80;
    /// La pluie arrive dans moins de trois heures.
    pub const PLUIE_PROCHE: u8 = 60;
    /// Sec, ou pluie plus lointaine.
    pub const SEC: u8 = 10;
    /// Trois heures.
    pub const PLUIE_DANS_MS: i64 = 3 * super::HEURE_MS;
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    /// 14 h 00 pile.
    const QUATORZE: i64 = 1_790_000_000_000 - 1_790_000_000_000 % HEURE_MS;
    const MINUTE: i64 = 60_000;

    fn heure(decalage: i64, temperature: f64, debut: i64) -> HourlySample {
        HourlySample {
            time: debut + decalage * HEURE_MS,
            weather_code: 3,
            is_day: true,
            precipitation_probability: 20.0,
            temperature,
            apparent_temperature: temperature,
            relative_humidity: 65.0,
            dew_point: 11.0,
            precipitation: 0.0,
            wind_speed: 8.0,
            wind_gusts: 14.0,
            uv_index: 1.0,
        }
    }

    fn serie(n: i64, debut: i64) -> Vec<HourlySample> {
        (0..n).map(|i| heure(i, 18.0 + i as f64, debut)).collect()
    }

    /// Une série de douze heures sèches où il pleut à l'heure `pluie`.
    fn serie_pluvieuse(pluie: Option<usize>) -> Vec<HourlySample> {
        let mut s = serie(12, QUATORZE);
        if let Some(i) = pluie {
            s[i].precipitation = 1.2;
        }
        s
    }

    #[test]
    fn il_pleut_la_pile_remonte_kliima() {
        assert_eq!(pertinence(QUATORZE + 20 * MINUTE, &serie_pluvieuse(Some(0))), 80);
    }

    #[test]
    fn une_pluie_dans_deux_heures_pese_plus_qu_un_ciel_sec() {
        assert_eq!(pertinence(QUATORZE + 20 * MINUTE, &serie_pluvieuse(Some(2))), 60);
        assert_eq!(pertinence(QUATORZE + 20 * MINUTE, &serie_pluvieuse(None)), 10);
    }

    #[test]
    fn une_pluie_lointaine_ne_compte_pas() {
        assert_eq!(pertinence(QUATORZE, &serie_pluvieuse(Some(3))), 10, "trois heures pile : trop loin");
        assert_eq!(pertinence(QUATORZE, &serie_pluvieuse(Some(6))), 10);
    }

    #[test]
    fn les_trois_heures_se_comptent_depuis_l_entree() {
        // Une entrée posée à 19 h, calculée à 14 h : une pluie à 20 h est à
        // une heure d'elle, pas à six.
        let s = serie_pluvieuse(Some(6));
        assert_eq!(pertinence(QUATORZE + 5 * HEURE_MS, &s[5..]), 60);
    }

    #[test]
    fn a_quatorze_heures_vingt_cest_lheure_de_quatorze_heures() {
        // Le défaut corrigé côté iPhone : la première heure *après* l'instant
        // montrait quinze heures à quatorze heures vingt.
        let s = serie(6, QUATORZE);
        assert_eq!(heure_contenant(QUATORZE + 20 * MINUTE, &s).map(|h| h.time), Some(QUATORZE));
    }

    #[test]
    fn a_lheure_pile_cest_la_nouvelle_heure() {
        let s = serie(6, QUATORZE);
        assert_eq!(heure_contenant(QUATORZE + HEURE_MS, &s).map(|h| h.time), Some(QUATORZE + HEURE_MS));
    }

    #[test]
    fn apres_la_fin_de_la_serie_rien_nest_presente_comme_actuel() {
        let s = serie(6, QUATORZE);
        assert!(heure_contenant(QUATORZE + 6 * HEURE_MS + MINUTE, &s).is_none());
    }

    #[test]
    fn la_suivante_est_lheure_dapres() {
        let s = serie(6, QUATORZE);
        assert_eq!(
            heure_suivante(QUATORZE + 20 * MINUTE, &s).map(|h| h.time),
            Some(QUATORZE + HEURE_MS)
        );
    }

    #[test]
    fn la_bascule_est_le_prochain_debut_dheure() {
        let s = serie(6, QUATORZE);
        assert_eq!(prochaine_bascule(QUATORZE + 20 * MINUTE, &s), Some(QUATORZE + HEURE_MS));
    }

    #[test]
    fn un_fuseau_a_la_demie_bascule_a_la_demie() {
        let demie = QUATORZE + 30 * MINUTE;
        let s = serie(6, demie);
        assert_eq!(heure_contenant(QUATORZE + 40 * MINUTE, &s).map(|h| h.time), Some(demie));
        assert_eq!(prochaine_bascule(QUATORZE + 40 * MINUTE, &s), Some(demie + HEURE_MS));
    }

    #[test]
    fn le_jour_est_le_dernier_commence() {
        let jour = |date: i64| DailySample {
            date,
            weather_code: 3,
            temperature_min: 10.0,
            temperature_max: 20.0,
            precipitation_sum: 0.0,
            precipitation_probability_max: 0.0,
            uv_index_max: 0.0,
            wind_gusts_max: 0.0,
            sunrise: None,
            sunset: None,
        };
        let minuit = QUATORZE - 14 * HEURE_MS;
        let jours = [jour(minuit), jour(minuit + 24 * HEURE_MS)];
        assert_eq!(jour_contenant(QUATORZE, &jours).map(|j| j.date), Some(minuit));
        assert_eq!(
            jour_contenant(minuit + 25 * HEURE_MS, &jours).map(|j| j.date),
            Some(minuit + 24 * HEURE_MS)
        );
        // Minuit pile, c'est déjà le lendemain.
        assert_eq!(
            jour_contenant(minuit + 24 * HEURE_MS, &jours).map(|j| j.date),
            Some(minuit + 24 * HEURE_MS)
        );
        // Avant la série, faute de mieux, le premier jour ; sans série, rien.
        assert_eq!(jour_contenant(minuit - HEURE_MS, &jours).map(|j| j.date), Some(minuit));
        assert!(jour_contenant(QUATORZE, &[]).is_none());
    }

    fn jour_de(date: i64) -> DailySample {
        DailySample {
            date,
            weather_code: 3,
            temperature_min: 9.0,
            temperature_max: 13.0,
            precipitation_sum: 0.0,
            precipitation_probability_max: 0.0,
            uv_index_max: 0.0,
            wind_gusts_max: 0.0,
            sunrise: None,
            sunset: None,
        }
    }

    #[test]
    fn le_jour_embrasse_la_temperature_affichee() {
        let minuit = QUATORZE - 14 * HEURE_MS;
        let jours = [jour_de(minuit), jour_de(minuit + 24 * HEURE_MS)];
        let jour = jour_embrassant(QUATORZE + HEURE_MS, 14.2, &jours).unwrap();
        assert_eq!((jour.temperature_min, jour.temperature_max), (9.0, 14.2), "« 14° » sous « ↑ 13° »");
        let jour = jour_embrassant(QUATORZE, 7.5, &jours).unwrap();
        assert_eq!((jour.temperature_min, jour.temperature_max), (7.5, 13.0));
    }

    #[test]
    fn le_jour_ne_retrecit_jamais() {
        let minuit = QUATORZE - 14 * HEURE_MS;
        let jour = jour_embrassant(QUATORZE, 11.0, &[jour_de(minuit)]).unwrap();
        assert_eq!((jour.temperature_min, jour.temperature_max), (9.0, 13.0));
    }

    #[test]
    fn seul_le_jour_de_l_instant_s_elargit() {
        let minuit = QUATORZE - 14 * HEURE_MS;
        let jours = [jour_de(minuit), jour_de(minuit + 24 * HEURE_MS)];
        // Demain quatorze heures : c'est demain qui s'élargit, pas aujourd'hui.
        let demain = jour_embrassant(QUATORZE + 24 * HEURE_MS, 20.0, &jours).unwrap();
        assert_eq!(demain.date, minuit + 24 * HEURE_MS);
        assert_eq!(demain.temperature_max, 20.0);
        assert_eq!(jours[0].temperature_max, 13.0);
    }

    #[test]
    fn hors_de_la_serie_le_jour_reste_tel_quel() {
        let minuit = QUATORZE - 14 * HEURE_MS;
        let jours = [jour_de(minuit)];
        // La veille : le premier jour est rendu, mais pas élargi.
        let veille = jour_embrassant(minuit - HEURE_MS, 30.0, &jours).unwrap();
        assert_eq!(veille.temperature_max, 13.0);
        // Le surlendemain : la série est finie.
        let apres = jour_embrassant(minuit + 30 * HEURE_MS, 30.0, &jours).unwrap();
        assert_eq!(apres.temperature_max, 13.0);
        assert!(jour_embrassant(QUATORZE, 20.0, &[]).is_none());
        let nan = jour_embrassant(QUATORZE, f64::NAN, &jours).unwrap();
        assert_eq!((nan.temperature_min, nan.temperature_max), (9.0, 13.0));
    }
}
