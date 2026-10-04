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

use crate::agro::{DailySample, HourlySample};

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
            relative_humidity: 65.0,
            dew_point: 11.0,
            precipitation: 0.0,
            wind_speed: 8.0,
            wind_gusts: 14.0,
            soil_temperature_6cm: 15.0,
            soil_moisture_3to9cm: 0.24,
            et0: 0.2,
            vapour_pressure_deficit: 0.7,
        }
    }

    fn serie(n: i64, debut: i64) -> Vec<HourlySample> {
        (0..n).map(|i| heure(i, 18.0 + i as f64, debut)).collect()
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
            et0_sum: 0.0,
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
    }
}
