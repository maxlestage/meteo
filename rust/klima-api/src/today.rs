//! Synthèse de la journée en cours.
//!
//! C'est ce qu'affiche la section « météo du jour » du site de présentation :
//! le temps qu'il fera aujourd'hui, et ce que Klima en déduit pour sortir —
//! la pluie qui vient, ce qu'il faut emporter, la force du soleil.
//!
//! Le module vit dans `klima-api` et non dans le cœur parce qu'il part d'une
//! prévision décodée, pas de règles. La convention des horodatages lui rend
//! d'ailleurs service : « aujourd'hui » est une division par 86 400 000, là où
//! il faudrait sinon demander à `Intl` quel jour civil il est dans le fuseau
//! de la ville.

use klima_core::calendar::at_midnight;
use klima_core::meteo::HourlySample;
use klima_core::ville::{Conseil, NiveauUv, Pluie, conseils, niveau_uv, prochaine_pluie};

use crate::open_meteo::Forecast;

#[derive(Debug, Clone, PartialEq)]
pub struct DayDigest {
    pub date: i64,
    pub weather_code: u16,
    pub temperature_min: f64,
    pub temperature_max: f64,
    /// Cumul de pluie attendu sur la journée (mm).
    pub precipitation_sum: f64,
    pub precipitation_probability_max: f64,
    pub wind_gusts_max: f64,
    /// Indice UV le plus fort de la journée, et son niveau.
    pub uv_index_max: f64,
    pub uv: NiveauUv,
    pub sunrise: Option<i64>,
    pub sunset: Option<i64>,
    /// Heures restantes de la journée, à partir de l'heure en cours.
    pub remaining_hours: Vec<HourlySample>,
    /// La pluie des douze prochaines heures.
    pub pluie: Pluie,
    /// Ce qu'il faut emporter pour les douze prochaines heures.
    pub conseils: Vec<Conseil>,
}

/// Journée en cours d'une prévision.
///
/// Renvoie `None` si la série ne couvre pas aujourd'hui, ce qui ne devrait
/// arriver qu'avec une réponse tronquée.
pub fn day_digest(forecast: &Forecast) -> Option<DayDigest> {
    let today = forecast.daily.first()?;

    let jour = at_midnight(today.date);
    let remaining_hours: Vec<HourlySample> = forecast
        .hourly
        .iter()
        .filter(|hour| at_midnight(hour.time) == jour)
        .cloned()
        .collect();

    Some(DayDigest {
        date: today.date,
        weather_code: today.weather_code,
        temperature_min: today.temperature_min,
        temperature_max: today.temperature_max,
        precipitation_sum: today.precipitation_sum,
        precipitation_probability_max: today.precipitation_probability_max,
        wind_gusts_max: today.wind_gusts_max,
        uv_index_max: today.uv_index_max,
        uv: niveau_uv(today.uv_index_max),
        sunrise: today.sunrise,
        sunset: today.sunset,
        // La pluie et les conseils regardent douze heures devant, au-delà de
        // minuit s'il le faut : le soir, c'est la nuit et le matin qui
        // comptent.
        pluie: prochaine_pluie(&forecast.hourly),
        conseils: conseils(&forecast.hourly),
        remaining_hours,
    })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_meteo::Parcelle;
    use klima_core::meteo::{CurrentSample, DailySample};
    use klima_core::calendar::civil;

    const HOUR_MS: i64 = 3_600_000;
    const DAY_MS: i64 = 86_400_000;

    /// Minuit à la parcelle, le 12 mai 2026.
    fn minuit() -> i64 {
        civil(2026, 5, 12, 0, 0)
    }

    fn hour(offset: i64) -> HourlySample {
        HourlySample {
            time: minuit() + offset * HOUR_MS,
            weather_code: 3,
            is_day: true,
            precipitation_probability: 20.0,
            temperature: 18.0,
            apparent_temperature: 17.0,
            relative_humidity: 65.0,
            dew_point: 11.0,
            precipitation: 0.0,
            wind_speed: 8.0,
            wind_gusts: 14.0,
            uv_index: 0.0,
        }
    }

    fn day(offset: i64) -> DailySample {
        DailySample {
            date: minuit() + offset * DAY_MS,
            weather_code: 61,
            temperature_min: 9.0,
            temperature_max: 21.0,
            precipitation_sum: 2.4,
            precipitation_probability_max: 60.0,
            wind_gusts_max: 32.0,
            uv_index_max: 6.4,
            sunrise: Some(minuit() + 6 * HOUR_MS + 30 * 60_000),
            sunset: Some(minuit() + 21 * HOUR_MS + 24 * 60_000),
        }
    }

    /// Prévision partant de 18 h : il reste six heures aujourd'hui, puis demain.
    fn forecast(hours: Vec<HourlySample>, days: Vec<DailySample>) -> Forecast {
        Forecast {
            parcelle: Parcelle {
                name: "Chartres".to_owned(),
                latitude: 48.44,
                longitude: 1.48,
                admin: None,
                country: None,
            },
            timezone: "Europe/Paris".to_owned(),
            utc_offset_seconds: 7200,
            elevation: 155.0,
            current: CurrentSample {
                time: minuit() + 18 * HOUR_MS,
                temperature: 19.0,
                apparent_temperature: 19.0,
                weather_code: 61,
                is_day: true,
                relative_humidity: 70.0,
                wind_speed: 10.0,
                wind_gusts: 20.0,
                pressure: 1015.0,
            },
            hourly: hours,
            daily: days,
            fetched_at: minuit() + 18 * HOUR_MS,
        }
    }

    fn depuis_dix_huit_heures() -> Forecast {
        forecast((0..30).map(|i| hour(18 + i)).collect(), vec![day(0), day(1)])
    }

    #[test]
    fn ne_retient_que_les_heures_restantes_de_la_journee() {
        let digest = day_digest(&depuis_dix_huit_heures()).unwrap();
        // De 18 h à 23 h inclus : six heures avant minuit.
        assert_eq!(digest.remaining_hours.len(), 6);
    }

    #[test]
    fn reprend_le_cumul_et_luv_du_jour() {
        let digest = day_digest(&depuis_dix_huit_heures()).unwrap();
        assert_eq!(digest.precipitation_sum, 2.4);
        assert_eq!(digest.uv_index_max, 6.4);
        assert_eq!(digest.uv, NiveauUv::Eleve);
        assert_eq!(digest.weather_code, 61);
    }

    #[test]
    fn une_soiree_seche_et_douce_ne_demande_rien() {
        let digest = day_digest(&depuis_dix_huit_heures()).unwrap();
        assert_eq!(digest.pluie, Pluie::Aucune { heures: 12 });
        assert!(digest.conseils.is_empty());
    }

    #[test]
    fn la_pluie_de_la_nuit_se_dit_des_le_soir() {
        let pluvieux = forecast(
            (0..30)
                .map(|i| HourlySample {
                    precipitation: if i == 4 { 1.5 } else { 0.0 },
                    precipitation_probability: if i == 4 { 70.0 } else { 10.0 },
                    ..hour(18 + i)
                })
                .collect(),
            vec![day(0), day(1)],
        );
        let digest = day_digest(&pluvieux).unwrap();
        assert_eq!(
            digest.pluie,
            Pluie::Prevue { debut: minuit() + 22 * HOUR_MS, probabilite: 70.0, cumul: 1.5 }
        );
        assert_eq!(digest.conseils, vec![Conseil::Parapluie]);
    }

    #[test]
    fn serie_journaliere_vide() {
        assert_eq!(day_digest(&forecast(vec![hour(18)], vec![])), None);
    }

    #[test]
    fn aujourdhui_est_une_division_pas_une_question_de_fuseau() {
        // Les horodatages étant ceux de la parcelle, la journée se découpe par
        // calcul. Une heure de la veille à 23 h et une du lendemain à 0 h ne
        // tombent pas dans la même journée, même à une heure d'écart.
        let digest = day_digest(&forecast(
            vec![hour(-1), hour(0), hour(23), hour(24)],
            vec![day(0)],
        ))
        .unwrap();

        assert_eq!(digest.remaining_hours.len(), 2);
        assert_eq!(digest.remaining_hours[0].time, minuit());
        assert_eq!(digest.remaining_hours[1].time, minuit() + 23 * HOUR_MS);
    }
}
