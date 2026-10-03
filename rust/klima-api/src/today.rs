//! Synthèse de la journée en cours.
//!
//! Miroir de `core/src/today.ts`.
//!
//! C'est ce qu'affiche la section « météo du jour » du site de présentation :
//! le temps qu'il fera aujourd'hui, et ce que Klima en déduit pour la
//! parcelle.
//!
//! Le module vit dans `klima-api` et non dans le cœur parce qu'il part d'une
//! prévision décodée, pas de règles. La convention des horodatages lui rend
//! d'ailleurs service : « aujourd'hui » est une division par 86 400 000, là où
//! le TypeScript doit demander à `Intl` quel jour civil il est dans le fuseau
//! de la parcelle.

use klima_core::agro::{
    FrostRisk, HourlySample, SoilCondition, SprayOpportunity, frost_risk,
    next_spray_opportunity, soil_condition, spray_windows,
};
use klima_core::calendar::at_midnight;

use crate::open_meteo::AgroForecast;

#[derive(Debug, Clone, PartialEq)]
pub struct DayDigest {
    pub date: i64,
    pub weather_code: u16,
    pub temperature_min: f64,
    pub temperature_max: f64,
    /// Cumul de pluie attendu sur la journée (mm).
    pub precipitation_sum: f64,
    pub precipitation_probability_max: f64,
    /// Évapotranspiration de référence du jour (mm).
    pub et0_sum: f64,
    /// Pluie − ET0 sur la seule journée (mm).
    pub balance: f64,
    pub wind_gusts_max: f64,
    pub sunrise: Option<i64>,
    pub sunset: Option<i64>,
    /// Heures restantes de la journée, à partir de l'heure en cours.
    pub remaining_hours: Vec<HourlySample>,
    /// Fenêtre de traitement d'ici ce soir, s'il en reste une.
    pub spray: Option<SprayOpportunity>,
    pub soil: SoilCondition,
    /// Gel attendu la nuit prochaine — elle déborde sur le lendemain.
    pub frost: FrostRisk,
}

/// Journée en cours d'une prévision.
///
/// Renvoie `None` si la série ne couvre pas aujourd'hui, ce qui ne devrait
/// arriver qu'avec une réponse tronquée.
pub fn day_digest(forecast: &AgroForecast) -> Option<DayDigest> {
    let today = forecast.daily.first()?;

    let jour = at_midnight(today.date);
    let remaining_hours: Vec<HourlySample> = forecast
        .hourly
        .iter()
        .filter(|hour| at_midnight(hour.time) == jour)
        .cloned()
        .collect();

    // Le gel se juge sur la nuit qui vient, laquelle déborde sur le lendemain.
    let tonight = &forecast.hourly[..forecast.hourly.len().min(18)];
    let (min_temp, min_dew) = if tonight.is_empty() {
        (today.temperature_min, 0.0)
    } else {
        (
            tonight.iter().map(|h| h.temperature).fold(f64::INFINITY, f64::min),
            tonight.iter().map(|h| h.dew_point).fold(f64::INFINITY, f64::min),
        )
    };

    Some(DayDigest {
        date: today.date,
        weather_code: today.weather_code,
        temperature_min: today.temperature_min,
        temperature_max: today.temperature_max,
        precipitation_sum: today.precipitation_sum,
        precipitation_probability_max: today.precipitation_probability_max,
        et0_sum: today.et0_sum,
        balance: arrondi_dixieme(today.precipitation_sum - today.et0_sum),
        wind_gusts_max: today.wind_gusts_max,
        sunrise: today.sunrise,
        sunset: today.sunset,
        // Deux heures au moins, comme partout ailleurs : une fenêtre d'une
        // heure ne laisse pas le temps de traiter une parcelle.
        spray: next_spray_opportunity(&spray_windows(&remaining_hours), 2),
        soil: soil_condition(&remaining_hours),
        frost: frost_risk(min_temp, min_dew),
        remaining_hours,
    })
}

/// Le même arrondi que partout ailleurs dans le cœur.
fn arrondi_dixieme(value: f64) -> f64 {
    (value * 10.0 + 0.5).floor() / 10.0
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_meteo::Parcelle;
    use klima_core::agro::{CurrentSample, DailySample, FrostSeverity};
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

    fn day(offset: i64) -> DailySample {
        DailySample {
            date: minuit() + offset * DAY_MS,
            weather_code: 61,
            temperature_min: 9.0,
            temperature_max: 21.0,
            precipitation_sum: 2.4,
            precipitation_probability_max: 60.0,
            et0_sum: 3.6,
            wind_gusts_max: 32.0,
            sunrise: Some(minuit() + 6 * HOUR_MS + 30 * 60_000),
            sunset: Some(minuit() + 21 * HOUR_MS + 24 * 60_000),
        }
    }

    /// Prévision partant de 18 h : il reste six heures aujourd'hui, puis demain.
    fn forecast(hours: Vec<HourlySample>, days: Vec<DailySample>) -> AgroForecast {
        AgroForecast {
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
            },
            hourly: hours,
            daily: days,
            fetched_at: minuit() + 18 * HOUR_MS,
        }
    }

    fn depuis_dix_huit_heures() -> AgroForecast {
        forecast((0..30).map(|i| hour(18 + i)).collect(), vec![day(0), day(1)])
    }

    #[test]
    fn ne_retient_que_les_heures_restantes_de_la_journee() {
        let digest = day_digest(&depuis_dix_huit_heures()).unwrap();
        // De 18 h à 23 h inclus : six heures avant minuit.
        assert_eq!(digest.remaining_hours.len(), 6);
    }

    #[test]
    fn reprend_les_cumuls_du_jour_et_calcule_son_bilan() {
        let digest = day_digest(&depuis_dix_huit_heures()).unwrap();
        assert_eq!(digest.precipitation_sum, 2.4);
        assert_eq!(digest.et0_sum, 3.6);
        assert_eq!(digest.balance, -1.2);
        assert_eq!(digest.weather_code, 61);
    }

    #[test]
    fn propose_une_fenetre_de_traitement_dici_ce_soir() {
        let digest = day_digest(&depuis_dix_huit_heures()).unwrap();
        let spray = digest.spray.expect("une fenêtre");
        assert_eq!(spray.score, 100);
    }

    #[test]
    fn aucune_fenetre_si_le_vent_souffle_jusqua_la_nuit() {
        let venteux = forecast(
            (0..30)
                .map(|i| HourlySample { wind_speed: 38.0, ..hour(18 + i) })
                .collect(),
            vec![day(0), day(1)],
        );
        assert_eq!(day_digest(&venteux).unwrap().spray, None);
    }

    #[test]
    fn le_gel_se_juge_sur_la_nuit_qui_deborde_sur_le_lendemain() {
        let gelant = forecast(
            (0..30)
                .map(|i| HourlySample {
                    temperature: if i >= 8 { -3.0 } else { 10.0 },
                    dew_point: -5.0,
                    ..hour(18 + i)
                })
                .collect(),
            vec![day(0), day(1)],
        );
        let digest = day_digest(&gelant).unwrap();

        assert_eq!(digest.frost.severity, FrostSeverity::Modere);
        assert!(digest.frost.hoar_frost);
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
