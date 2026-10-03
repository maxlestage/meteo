//! Cœur agronomique de Klima.
//!
//! Toutes les fonctions sont pures : elles prennent des mesures météo brutes
//! et renvoient des indicateurs exploitables au champ. Les mêmes seuils vivent
//! dans `core/src/agro.ts` et dans `ios/Kliima/Models/AgroIndicators.swift` ;
//! les cas de test sont recopiés à l'identique des trois côtés.
//!
//! Les horodatages sont des millisecondes depuis l'époque, et non un type date
//! d'une bibliothèque : le cœur n'a aucune dépendance, et une règle
//! agronomique n'a pas besoin de connaître les fuseaux. L'interface les
//! habille.

/// Seuils agronomiques partagés.
///
/// Les valeurs suivent les recommandations usuelles de pulvérisation
/// (vent < 19 km/h — arrêté du 4 mai 2017 en France) et le modèle FAO-56.
pub mod thresholds {
    /// Base de calcul des degrés-jours (maïs, tournesol : 6 à 10 °C).
    pub const GDD_BASE: f64 = 10.0;
    /// Plafond au-delà duquel la plante ne capitalise plus (°C).
    pub const GDD_CEILING: f64 = 30.0;
    /// Vent minimum : en dessous, l'inversion thermique fait dériver le produit.
    pub const SPRAY_WIND_MIN: f64 = 3.0;
    /// Vent maximum réglementaire pour la pulvérisation (km/h).
    pub const SPRAY_WIND_MAX: f64 = 19.0;
    pub const SPRAY_GUST_MAX: f64 = 25.0;
    pub const SPRAY_TEMP_MIN: f64 = 5.0;
    pub const SPRAY_TEMP_MAX: f64 = 25.0;
    pub const SPRAY_HUMIDITY_MIN: f64 = 40.0;
    /// Au-delà de ce VPD, la gouttelette s'évapore avant d'atteindre la cible.
    pub const SPRAY_VPD_MAX: f64 = 1.2;
    /// Pluie tolérée sur l'heure et l'heure suivante (mm).
    pub const SPRAY_RAIN_MAX: f64 = 0.1;
    /// Humidité relative à partir de laquelle le feuillage est mouillé.
    pub const LEAF_WETNESS_HUMIDITY: f64 = 90.0;
    pub const DISEASE_TEMP_MIN: f64 = 8.0;
    pub const DISEASE_TEMP_MAX: f64 = 30.0;
    /// Humidité volumique du sol : ressuyage et portance.
    pub const SOIL_TOO_WET: f64 = 0.35;
    pub const SOIL_IDEAL_MIN: f64 = 0.15;
    pub const SOIL_TOO_DRY: f64 = 0.12;
    /// Déficit hydrique déclenchant une alerte d'irrigation (mm sur 7 jours).
    pub const IRRIGATION_DEFICIT: f64 = -15.0;
}

use thresholds::*;

/// Une heure de prévision.
#[derive(Debug, Clone, PartialEq)]
pub struct HourlySample {
    /// Millisecondes depuis l'époque.
    pub time: i64,
    pub weather_code: u16,
    pub is_day: bool,
    pub precipitation_probability: f64,
    pub temperature: f64,
    pub relative_humidity: f64,
    pub dew_point: f64,
    pub precipitation: f64,
    pub wind_speed: f64,
    pub wind_gusts: f64,
    pub soil_temperature_6cm: f64,
    pub soil_moisture_3to9cm: f64,
    pub et0: f64,
    pub vapour_pressure_deficit: f64,
}

/// Une journée de prévision.
#[derive(Debug, Clone, PartialEq)]
pub struct DailySample {
    pub date: i64,
    pub weather_code: u16,
    pub temperature_min: f64,
    pub temperature_max: f64,
    pub precipitation_sum: f64,
    pub precipitation_probability_max: f64,
    pub et0_sum: f64,
    pub wind_gusts_max: f64,
    pub sunrise: Option<i64>,
    pub sunset: Option<i64>,
}

/* ---------------------------------------------------------------- */
/* Degrés-jours de croissance                                        */
/* ---------------------------------------------------------------- */

/// Degrés-jours d'une journée (méthode « moyenne plafonnée »).
///
/// GDD = clamp((Tmin + Tmax) / 2, base, plafond) − base
pub fn growing_degree_days(minimum: f64, maximum: f64, base: f64, ceiling: f64) -> f64 {
    let mean = (minimum + maximum) / 2.0;
    let capped = mean.max(base).min(ceiling);
    round(capped - base, 1)
}

/// Cumul de degrés-jours sur une série de journées.
pub fn cumulative_gdd(days: &[DailySample], base: f64) -> f64 {
    let total: f64 = days
        .iter()
        .map(|d| growing_degree_days(d.temperature_min, d.temperature_max, base, GDD_CEILING))
        .sum();
    round(total, 1)
}

/* ---------------------------------------------------------------- */
/* Bilan hydrique                                                    */
/* ---------------------------------------------------------------- */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterStatus {
    Deficit,
    Equilibre,
    Excedent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaterBalance {
    pub precipitation: f64,
    pub evapotranspiration: f64,
    /// Pluie − ET0 (mm). Négatif = la parcelle puise dans sa réserve.
    pub balance: f64,
    pub status: WaterStatus,
    /// Conseil d'irrigation en mm à apporter (0 si inutile).
    pub irrigation_advice: f64,
}

/// Bilan hydrique climatique P − ET0 sur la période fournie.
pub fn water_balance(days: &[DailySample]) -> WaterBalance {
    let precipitation = round(days.iter().map(|d| d.precipitation_sum).sum(), 1);
    let evapotranspiration = round(days.iter().map(|d| d.et0_sum).sum(), 1);
    let balance = round(precipitation - evapotranspiration, 1);

    let status = if balance <= IRRIGATION_DEFICIT {
        WaterStatus::Deficit
    } else if balance >= 15.0 {
        WaterStatus::Excedent
    } else {
        WaterStatus::Equilibre
    };

    WaterBalance {
        precipitation,
        evapotranspiration,
        balance,
        status,
        irrigation_advice: if status == WaterStatus::Deficit { balance.abs() } else { 0.0 },
    }
}

/* ---------------------------------------------------------------- */
/* Fenêtres de pulvérisation                                         */
/* ---------------------------------------------------------------- */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SprayVerdict {
    Favorable,
    Acceptable,
    Defavorable,
}

/// Motif de dégradation d'une heure, sous forme structurée : le domaine dit ce
/// qui cloche et avec quelles valeurs, l'interface le formule dans sa langue.
#[derive(Debug, Clone, PartialEq)]
pub enum SprayBlocker {
    WindTooStrong { wind: f64, limit: f64 },
    WindTooWeak,
    Gusts { gusts: f64 },
    Rain { amount: f64 },
    TooHot { temperature: f64 },
    TooCold { temperature: f64 },
    DryAir { humidity: f64 },
    VapourPressureDeficit { vpd: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SprayWindow {
    pub time: i64,
    pub verdict: SprayVerdict,
    /// Score 0–100 : 100 = conditions idéales.
    pub score: i32,
    pub blockers: Vec<SprayBlocker>,
}

/// L'heure demandée n'existe pas dans la série.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfRange(pub usize);

/// Évalue l'heure `index` de la série pour un traitement phytosanitaire.
///
/// Deux familles de critères : les rédhibitoires (vent au-delà de la limite
/// réglementaire, rafales, pluie imminente) rendent l'heure inexploitable quoi
/// qu'il arrive ; les critères de confort (température, hygrométrie, VPD)
/// dégradent le score.
pub fn evaluate_spray_hour(hours: &[HourlySample], index: usize) -> Result<SprayWindow, OutOfRange> {
    let h = hours.get(index).ok_or(OutOfRange(index))?;
    let next = hours.get(index + 1);

    let mut blockers = Vec::new();
    let mut score: i32 = 100;
    // Un critère rédhibitoire interdit le passage, quel que soit le reste.
    let mut disqualified = false;

    if h.wind_speed > SPRAY_WIND_MAX {
        blockers.push(SprayBlocker::WindTooStrong { wind: h.wind_speed, limit: SPRAY_WIND_MAX });
        score -= 45;
        disqualified = true;
    } else if h.wind_speed < SPRAY_WIND_MIN {
        blockers.push(SprayBlocker::WindTooWeak);
        score -= 25;
    }

    if h.wind_gusts > SPRAY_GUST_MAX {
        blockers.push(SprayBlocker::Gusts { gusts: h.wind_gusts });
        score -= 20;
        disqualified = true;
    }

    let rain_soon = h.precipitation + next.map_or(0.0, |n| n.precipitation);
    if rain_soon > SPRAY_RAIN_MAX {
        blockers.push(SprayBlocker::Rain { amount: round(rain_soon, 1) });
        score -= 45;
        disqualified = true;
    }

    if h.temperature > SPRAY_TEMP_MAX {
        blockers.push(SprayBlocker::TooHot { temperature: h.temperature });
        score -= 20;
    } else if h.temperature < SPRAY_TEMP_MIN {
        blockers.push(SprayBlocker::TooCold { temperature: h.temperature });
        score -= 20;
    }

    if h.relative_humidity < SPRAY_HUMIDITY_MIN {
        blockers.push(SprayBlocker::DryAir { humidity: h.relative_humidity });
        score -= 15;
    }

    if h.vapour_pressure_deficit > SPRAY_VPD_MAX {
        blockers.push(SprayBlocker::VapourPressureDeficit {
            vpd: round(h.vapour_pressure_deficit, 2),
        });
        score -= 15;
    }

    score = score.clamp(0, 100);
    if disqualified {
        score = score.min(35);
    }

    let verdict = if disqualified {
        SprayVerdict::Defavorable
    } else if score >= 80 {
        SprayVerdict::Favorable
    } else if score >= 55 {
        SprayVerdict::Acceptable
    } else {
        SprayVerdict::Defavorable
    };

    Ok(SprayWindow { time: h.time, verdict, score, blockers })
}

/// Évalue toute la série horaire.
pub fn spray_windows(hours: &[HourlySample]) -> Vec<SprayWindow> {
    (0..hours.len())
        .map(|i| evaluate_spray_hour(hours, i).expect("index tiré de la série"))
        .collect()
}

/// Plage continue exploitable pour un traitement.
#[derive(Debug, Clone, PartialEq)]
pub struct SprayOpportunity {
    pub start: i64,
    pub end: i64,
    /// Score moyen de la plage, 0–100.
    pub score: i32,
}

/// Prochaine plage d'au moins `min_length` heures consécutives exploitables.
pub fn next_spray_opportunity(
    windows: &[SprayWindow],
    min_length: usize,
) -> Option<SprayOpportunity> {
    let mut run: Vec<&SprayWindow> = Vec::new();
    for w in windows {
        if w.verdict == SprayVerdict::Defavorable {
            run.clear();
            continue;
        }
        run.push(w);
        if run.len() >= min_length {
            let first = run[0];
            let last = run[run.len() - 1];
            let mean = run.iter().map(|x| x.score as f64).sum::<f64>() / run.len() as f64;
            return Some(SprayOpportunity {
                start: first.time,
                end: last.time + 3_600_000,
                score: round(mean, 0) as i32,
            });
        }
    }
    None
}

/* ---------------------------------------------------------------- */
/* Gel                                                               */
/* ---------------------------------------------------------------- */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrostSeverity {
    Aucun,
    Faible,
    Modere,
    Severe,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrostRisk {
    pub severity: FrostSeverity,
    pub min_temperature: f64,
    /// Vrai si le point de rosée est négatif : gelée blanche probable.
    pub hoar_frost: bool,
}

/// Risque de gel d'une nuit, d'après la température mini et le point de rosée.
pub fn frost_risk(min_temperature: f64, min_dew_point: f64) -> FrostRisk {
    let severity = if min_temperature <= -4.0 {
        FrostSeverity::Severe
    } else if min_temperature <= -2.0 {
        FrostSeverity::Modere
    } else if min_temperature <= 1.0 {
        FrostSeverity::Faible
    } else {
        FrostSeverity::Aucun
    };

    FrostRisk {
        severity,
        min_temperature: round(min_temperature, 1),
        hoar_frost: severity != FrostSeverity::Aucun && min_dew_point <= 0.0,
    }
}

/* ---------------------------------------------------------------- */
/* Pression maladie                                                  */
/* ---------------------------------------------------------------- */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiseaseLevel {
    Faible,
    Moyenne,
    Elevee,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiseasePressure {
    /// Heures d'humectation du feuillage.
    pub leaf_wetness_hours: usize,
    pub level: DiseaseLevel,
}

/// Approximation de la pression cryptogamique (mildiou, septoriose) : on
/// compte les heures d'humectation du feuillage dans la plage de température
/// favorable au champignon, à la manière des tables de Mills.
pub fn disease_pressure(hours: &[HourlySample]) -> DiseasePressure {
    let leaf_wetness_hours = hours
        .iter()
        .filter(|h| {
            h.relative_humidity >= LEAF_WETNESS_HUMIDITY
                && h.temperature >= DISEASE_TEMP_MIN
                && h.temperature <= DISEASE_TEMP_MAX
        })
        .count();

    let level = if leaf_wetness_hours >= 12 {
        DiseaseLevel::Elevee
    } else if leaf_wetness_hours >= 6 {
        DiseaseLevel::Moyenne
    } else {
        DiseaseLevel::Faible
    };

    DiseasePressure { leaf_wetness_hours, level }
}

/* ---------------------------------------------------------------- */
/* Portance et travail du sol                                        */
/* ---------------------------------------------------------------- */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoilState {
    Sature,
    Ressuye,
    Sec,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoilCondition {
    pub moisture: f64,
    pub temperature: f64,
    pub state: SoilState,
    /// Vrai si le sol porte les engins sans risque de tassement.
    pub trafficable: bool,
    /// Vrai si le sol est assez chaud pour lever (semis de printemps).
    pub sowable: bool,
}

/// État du sol moyenné sur la série horaire fournie.
pub fn soil_condition(hours: &[HourlySample]) -> SoilCondition {
    if hours.is_empty() {
        return SoilCondition {
            moisture: 0.0,
            temperature: 0.0,
            state: SoilState::Sec,
            trafficable: false,
            sowable: false,
        };
    }

    let moisture = round(mean(hours.iter().map(|h| h.soil_moisture_3to9cm)), 3);
    let temperature = round(mean(hours.iter().map(|h| h.soil_temperature_6cm)), 1);

    let state = if moisture >= SOIL_TOO_WET {
        SoilState::Sature
    } else if moisture <= SOIL_TOO_DRY {
        SoilState::Sec
    } else {
        SoilState::Ressuye
    };

    SoilCondition {
        moisture,
        temperature,
        state,
        trafficable: state != SoilState::Sature,
        sowable: temperature >= 8.0 && moisture >= SOIL_IDEAL_MIN && state != SoilState::Sature,
    }
}

/* ---------------------------------------------------------------- */
/* Synthèse                                                          */
/* ---------------------------------------------------------------- */

#[derive(Debug, Clone, PartialEq)]
pub struct AgroSummary {
    pub water: WaterBalance,
    pub soil: SoilCondition,
    pub disease: DiseasePressure,
    pub frost: FrostRisk,
    pub gdd: f64,
    pub next_spray: Option<SprayOpportunity>,
}

/// Assemble tous les indicateurs pour le tableau de bord.
pub fn summarize(hours: &[HourlySample], days: &[DailySample]) -> AgroSummary {
    let next24h = &hours[..hours.len().min(24)];
    let tonight = &hours[..hours.len().min(18)];

    let (min_temp, min_dew) = if tonight.is_empty() {
        (days.first().map_or(0.0, |d| d.temperature_min), 0.0)
    } else {
        (
            tonight.iter().map(|h| h.temperature).fold(f64::INFINITY, f64::min),
            tonight.iter().map(|h| h.dew_point).fold(f64::INFINITY, f64::min),
        )
    };

    AgroSummary {
        water: water_balance(days),
        soil: soil_condition(next24h),
        disease: disease_pressure(next24h),
        frost: frost_risk(min_temp, min_dew),
        gdd: cumulative_gdd(days, GDD_BASE),
        next_spray: next_spray_opportunity(&spray_windows(hours), 2),
    }
}

/* ---------------------------------------------------------------- */

fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let (sum, count) = values.fold((0.0, 0usize), |(s, c), v| (s + v, c + 1));
    sum / count as f64
}

/// Arrondi à `decimals` décimales, à la manière de JavaScript.
///
/// `floor(x + 0,5)` et non `round` : JavaScript arrondit les demis vers le
/// haut, Rust les écarte de zéro. Les trois écritures doivent rendre le même
/// nombre, sans quoi un score de 55 d'un côté devient 54 de l'autre — et le
/// verdict change avec lui.
fn round(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor + 0.5).floor() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Même gabarit que `core/src/agro.test.ts` : 12 mai 2026, 6 h UTC.
    const BASE_MS: i64 = 1_778_997_600_000;

    fn hour(index: i64) -> HourlySample {
        HourlySample {
            time: BASE_MS + index * 3_600_000,
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

    fn day() -> DailySample {
        DailySample {
            date: BASE_MS,
            weather_code: 3,
            temperature_min: 10.0,
            temperature_max: 22.0,
            precipitation_sum: 2.0,
            precipitation_probability_max: 30.0,
            et0_sum: 3.5,
            wind_gusts_max: 25.0,
            sunrise: None,
            sunset: None,
        }
    }

    // ---- Degrés-jours ------------------------------------------------

    #[test]
    fn moyenne_au_dessus_de_la_base() {
        assert_eq!(growing_degree_days(10.0, 22.0, GDD_BASE, GDD_CEILING), 6.0);
    }

    #[test]
    fn journee_trop_froide_ne_capitalise_rien() {
        assert_eq!(growing_degree_days(2.0, 8.0, GDD_BASE, GDD_CEILING), 0.0);
    }

    #[test]
    fn plafonnement_de_la_moyenne_a_trente_degres() {
        assert_eq!(growing_degree_days(30.0, 42.0, GDD_BASE, GDD_CEILING), 20.0);
    }

    #[test]
    fn cumul_sur_plusieurs_jours() {
        let chaud = DailySample { temperature_min: 14.0, temperature_max: 26.0, ..day() };
        assert_eq!(cumulative_gdd(&[day(), chaud], GDD_BASE), 16.0);
    }

    // ---- Bilan hydrique ----------------------------------------------

    #[test]
    fn deficit_declenche_un_conseil_dirrigation() {
        let sec = DailySample { precipitation_sum: 0.0, ..day() };
        let result = water_balance(&vec![sec; 8]);
        assert_eq!(result.evapotranspiration, 28.0);
        assert_eq!(result.balance, -28.0);
        assert_eq!(result.status, WaterStatus::Deficit);
        assert_eq!(result.irrigation_advice, 28.0);
    }

    #[test]
    fn pluies_abondantes_excedent_pas_dirrigation() {
        let pluvieux = DailySample { precipitation_sum: 20.0, ..day() };
        let result = water_balance(&[pluvieux]);
        assert_eq!(result.status, WaterStatus::Excedent);
        assert_eq!(result.irrigation_advice, 0.0);
    }

    #[test]
    fn serie_equilibree() {
        let result = water_balance(&[DailySample { precipitation_sum: 4.0, et0_sum: 3.5, ..day() }]);
        assert_eq!(result.status, WaterStatus::Equilibre);
    }

    // ---- Fenêtres de pulvérisation -----------------------------------

    #[test]
    fn conditions_ideales() {
        let w = evaluate_spray_hour(&[hour(0)], 0).unwrap();
        assert_eq!(w.score, 100);
        assert_eq!(w.verdict, SprayVerdict::Favorable);
        assert!(w.blockers.is_empty());
    }

    #[test]
    fn vent_reglementaire_depasse() {
        let h = HourlySample { wind_speed: 24.0, ..hour(0) };
        let w = evaluate_spray_hour(&[h], 0).unwrap();
        assert_eq!(w.verdict, SprayVerdict::Defavorable);
        assert_eq!(w.blockers[0], SprayBlocker::WindTooStrong { wind: 24.0, limit: 19.0 });
    }

    #[test]
    fn pluie_attendue_a_lheure_suivante() {
        let maintenant = HourlySample { precipitation: 0.0, ..hour(0) };
        let ensuite = HourlySample { precipitation: 1.4, ..hour(1) };
        let w = evaluate_spray_hour(&[maintenant, ensuite], 0).unwrap();
        assert_eq!(w.verdict, SprayVerdict::Defavorable);
        assert!(w.blockers.contains(&SprayBlocker::Rain { amount: 1.4 }));
    }

    #[test]
    fn vent_nul_inversion_thermique_signalee() {
        let h = HourlySample { wind_speed: 0.0, ..hour(0) };
        let w = evaluate_spray_hour(&[h], 0).unwrap();
        assert!(w.blockers.contains(&SprayBlocker::WindTooWeak));
        assert_eq!(w.verdict, SprayVerdict::Acceptable);
    }

    #[test]
    fn score_borne_a_zero() {
        let h = HourlySample {
            wind_speed: 60.0,
            wind_gusts: 90.0,
            precipitation: 12.0,
            temperature: 38.0,
            relative_humidity: 10.0,
            vapour_pressure_deficit: 5.0,
            ..hour(0)
        };
        let w = evaluate_spray_hour(&[h], 0).unwrap();
        assert_eq!(w.score, 0);
    }

    #[test]
    fn la_derniere_heure_de_la_serie_est_evaluable() {
        let windows = spray_windows(&[hour(0), hour(1)]);
        assert_eq!(windows.len(), 2);
    }

    #[test]
    fn index_hors_serie() {
        assert_eq!(evaluate_spray_hour(&[hour(0)], 5), Err(OutOfRange(5)));
    }

    // ---- Prochaine opportunité ---------------------------------------

    #[test]
    fn retient_la_premiere_plage_de_deux_heures_consecutives() {
        let hours = vec![
            HourlySample { wind_speed: 30.0, ..hour(0) },
            hour(1),
            hour(2),
            hour(3),
        ];
        let o = next_spray_opportunity(&spray_windows(&hours), 2).expect("une plage");
        assert_eq!(o.start, BASE_MS + 3_600_000);
        assert_eq!(o.end, BASE_MS + 3 * 3_600_000);
        assert_eq!(o.score, 100);
    }

    #[test]
    fn aucune_fenetre_quand_tout_est_defavorable() {
        let hours = vec![
            HourlySample { wind_speed: 45.0, ..hour(0) },
            HourlySample { wind_speed: 45.0, ..hour(1) },
        ];
        assert_eq!(next_spray_opportunity(&spray_windows(&hours), 2), None);
    }

    // ---- Gel ----------------------------------------------------------

    #[test]
    fn nuit_douce() {
        assert_eq!(frost_risk(6.0, 3.0).severity, FrostSeverity::Aucun);
    }

    #[test]
    fn gelee_blanche() {
        let risk = frost_risk(-0.5, -2.0);
        assert_eq!(risk.severity, FrostSeverity::Faible);
        assert!(risk.hoar_frost);
    }

    #[test]
    fn gel_severe() {
        assert_eq!(frost_risk(-6.0, -8.0).severity, FrostSeverity::Severe);
    }

    // ---- Pression maladie ---------------------------------------------

    #[test]
    fn feuillage_humide_et_doux_pression_elevee() {
        let hours: Vec<_> = (0..14)
            .map(|i| HourlySample { relative_humidity: 95.0, temperature: 16.0, ..hour(i) })
            .collect();
        let result = disease_pressure(&hours);
        assert_eq!(result.leaf_wetness_hours, 14);
        assert_eq!(result.level, DiseaseLevel::Elevee);
    }

    #[test]
    fn humidite_forte_mais_trop_froid_pas_dhumectation() {
        let hours: Vec<_> = (0..14)
            .map(|i| HourlySample { relative_humidity: 97.0, temperature: 3.0, ..hour(i) })
            .collect();
        assert_eq!(disease_pressure(&hours).level, DiseaseLevel::Faible);
    }

    // ---- Sol ------------------------------------------------------------

    #[test]
    fn sol_sature_pas_de_portance() {
        let soil = soil_condition(&[HourlySample { soil_moisture_3to9cm: 0.41, ..hour(0) }]);
        assert_eq!(soil.state, SoilState::Sature);
        assert!(!soil.trafficable);
        assert!(!soil.sowable);
    }

    #[test]
    fn sol_ressuye_et_chaud_semis_possible() {
        let soil = soil_condition(&[HourlySample {
            soil_moisture_3to9cm: 0.22,
            soil_temperature_6cm: 12.0,
            ..hour(0)
        }]);
        assert_eq!(soil.state, SoilState::Ressuye);
        assert!(soil.sowable);
    }

    #[test]
    fn sol_froid_semis_deconseille() {
        let soil = soil_condition(&[HourlySample { soil_temperature_6cm: 5.0, ..hour(0) }]);
        assert!(!soil.sowable);
    }

    #[test]
    fn serie_vide() {
        assert!(!soil_condition(&[]).trafficable);
    }

    // ---- Synthèse --------------------------------------------------------

    #[test]
    fn assemble_tous_les_indicateurs() {
        let hours: Vec<_> = (0..24).map(hour).collect();
        let days = vec![day(); 7];
        let summary = summarize(&hours, &days);
        assert_eq!(summary.gdd, 42.0);
        assert_eq!(summary.soil.state, SoilState::Ressuye);
        assert!(summary.next_spray.is_some());
        assert_eq!(summary.frost.severity, FrostSeverity::Aucun);
    }
}
