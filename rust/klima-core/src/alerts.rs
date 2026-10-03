//! Les alertes : ce que Klima dit sans qu'on ouvre l'application.
//!
//! Miroirs : `core/src/alerts.ts` et `ios/Kliima/Models/Alerts.swift`.
//!
//! Ce module ne notifie rien. Il répond à une seule question — « qu'y a-t-il à
//! dire, maintenant ? » — à partir de la prévision et de ce qu'on a déjà dit.
//! L'acheminement (notification locale, APNs) est affaire de plateforme ; la
//! règle est affaire de domaine, et c'est ce qui permet de la tester des trois
//! côtés sans appareil.
//!
//! Trois principes valent mieux que trente réglages :
//!
//! 1. **Une alerte sans marge de manœuvre est du bruit.** Prévenir qu'une
//!    fenêtre s'ouvre dans dix minutes ne sert à rien : on ne sort pas le
//!    pulvérisateur en dix minutes. D'où un préavis minimum.
//! 2. **On ne réveille personne.** Une alerte calculée la nuit attend le
//!    matin. Si l'événement est passé entre-temps, elle est abandonnée : mieux
//!    vaut se taire que raconter la veille.
//! 3. **On ne répète pas.** Une même nature d'alerte ne repart pas avant un
//!    délai de garde, sauf si ce qu'elle annonce a changé.
//!
//! Comme dans `cumuls`, les horodatages sont ceux de la parcelle : « 21 h »
//! est l'heure qu'il fait au champ, pas celle du serveur qui calcule. Le
//! silence nocturne est donc une simple division, et deux serveurs dans deux
//! fuseaux ne réveilleront plus personne à des heures différentes.

use std::collections::BTreeMap;

use crate::agro::{AgroSummary, HourlySample, SoilState, thresholds::SPRAY_RAIN_MAX};
use crate::i18n::{Params, params};

const HOUR_MS: i64 = 3_600_000;
const DAY_MS: i64 = 86_400_000;

/// La nature de ce qui est annoncé. Le délai de garde est tenu par nature :
/// une alerte de gel ne bâillonne pas une fenêtre de traitement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AlertKind {
    Fenetre,
    Gel,
    Sol,
    Pluie,
}

pub const ALERT_KINDS: [AlertKind; 4] =
    [AlertKind::Fenetre, AlertKind::Gel, AlertKind::Sol, AlertKind::Pluie];

impl AlertKind {
    /// Nom stable, pour les journaux et l'état sérialisé.
    pub fn code(self) -> &'static str {
        match self {
            AlertKind::Fenetre => "fenetre",
            AlertKind::Gel => "gel",
            AlertKind::Sol => "sol",
            AlertKind::Pluie => "pluie",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    pub kind: AlertKind,
    /// Moment de ce qui est annoncé — pas celui de l'envoi.
    pub at: i64,
    /// Clés de catalogue : le domaine ne fabrique pas de phrases.
    pub title_key: &'static str,
    pub body_key: &'static str,
    pub params: Params,
}

/// Ce qu'on sait déjà avoir dit. Fait pour survivre aux redémarrages.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AlertState {
    /// Dernier envoi par nature.
    pub last_sent: BTreeMap<AlertKind, i64>,
    /// État du sol au dernier examen, pour repérer le moment où il devient
    /// portant. Absent au premier lancement : on ne sait pas d'où l'on vient.
    pub last_soil_state: Option<SoilState>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlertOptions {
    pub now: i64,
    /// Heure locale à partir de laquelle on se tait, et heure de reprise.
    pub quiet_from: i64,
    pub quiet_to: i64,
    /// Délai de garde entre deux alertes de même nature (heures).
    pub cooldown_hours: f64,
    /// Préavis minimum avant une fenêtre de traitement (heures).
    pub lead_hours: f64,
    /// Préavis maximum : au-delà, il est trop tôt pour en parler.
    pub horizon_hours: f64,
}

impl AlertOptions {
    /// Les réglages par défaut, à l'instant donné.
    ///
    /// On se tait de 21 h à 6 h. Le gel de la nuit se dit donc en soirée,
    /// quand on peut encore bâcher ou allumer, pas à trois heures du matin.
    pub fn at(now: i64) -> Self {
        AlertOptions {
            now,
            quiet_from: 21,
            quiet_to: 6,
            cooldown_hours: 6.0,
            lead_hours: 2.0,
            horizon_hours: 18.0,
        }
    }
}

/// L'heure locale d'un horodatage de parcelle.
fn hour_of(ms: i64) -> i64 {
    ms.rem_euclid(DAY_MS) / HOUR_MS
}

/// Vrai si l'heure locale tombe dans la plage de silence.
pub fn is_quiet(hour: i64, from: i64, to: i64) -> bool {
    if from <= to { hour >= from && hour < to } else { hour >= from || hour < to }
}

/// Repousse un envoi à la fin du silence. Renvoie `None` si l'événement
/// annoncé sera passé d'ici là : une alerte en retard est pire que pas
/// d'alerte.
pub fn defer_past_quiet_hours(send: i64, event: i64, from: i64, to: i64) -> Option<i64> {
    if !is_quiet(hour_of(send), from, to) {
        return Some(send);
    }

    let midnight = send.div_euclid(DAY_MS) * DAY_MS;
    let day = if hour_of(send) >= to { midnight + DAY_MS } else { midnight };
    let resume = day + to * HOUR_MS;

    if resume <= event { Some(resume) } else { None }
}

/// Vrai si cette nature d'alerte est encore sous délai de garde.
fn held(kind: AlertKind, state: &AlertState, options: &AlertOptions) -> bool {
    match state.last_sent.get(&kind) {
        None => false,
        Some(last) => {
            ((options.now - last) as f64) < options.cooldown_hours * HOUR_MS as f64
        }
    }
}

/// Ce qu'il y a à dire maintenant.
///
/// Renvoie les alertes prêtes à partir, dans l'ordre où elles ont été
/// trouvées. Une liste vide est le cas normal : la plupart des heures n'ont
/// rien à annoncer, et c'est ce qui rend les alertes supportables.
pub fn evaluate_alerts(
    summary: &AgroSummary,
    hours: &[HourlySample],
    state: &AlertState,
    options: &AlertOptions,
) -> Vec<Alert> {
    let mut alerts = Vec::new();

    // 1. La fenêtre de traitement qui s'ouvre — la raison d'être des alertes.
    if let Some(spray) = &summary.next_spray {
        if !held(AlertKind::Fenetre, state, options) {
            let in_hours = (spray.start - options.now) as f64 / HOUR_MS as f64;
            if in_hours >= options.lead_hours && in_hours <= options.horizon_hours {
                alerts.push(Alert {
                    kind: AlertKind::Fenetre,
                    at: spray.start,
                    title_key: "alert.fenetre.title",
                    body_key: "alert.fenetre.body",
                    params: params([("score", spray.score.into())]),
                });
            }
        }
    }

    // 2. Le gel de la nuit. On le dit tant qu'il reste quelque chose à faire.
    if summary.frost.severity != crate::agro::FrostSeverity::Aucun
        && !held(AlertKind::Gel, state, options)
    {
        alerts.push(Alert {
            kind: AlertKind::Gel,
            at: first_freezing_hour(hours).unwrap_or(options.now),
            title_key: "alert.gel.title",
            body_key: "alert.gel.body",
            params: params([("temperature", summary.frost.min_temperature.into())]),
        });
    }

    // 3. Le sol devenu portant : le moment où l'on peut entrer en parcelle.
    //    C'est un changement d'état, pas un état — sans le précédent, on se tait.
    if state.last_soil_state == Some(SoilState::Sature)
        && summary.soil.state == SoilState::Ressuye
        && !held(AlertKind::Sol, state, options)
    {
        alerts.push(Alert {
            kind: AlertKind::Sol,
            at: options.now,
            title_key: "alert.sol.title",
            body_key: "alert.sol.body",
            params: params([("moisture", summary.soil.moisture.into())]),
        });
    }

    // 4. La pluie qui laverait un traitement fait à l'instant.
    if let Some(spray) = &summary.next_spray {
        if !held(AlertKind::Pluie, state, options) {
            if let Some(washout) = rain_after(hours, spray.end, 6) {
                alerts.push(Alert {
                    kind: AlertKind::Pluie,
                    at: washout.time,
                    title_key: "alert.pluie.title",
                    body_key: "alert.pluie.body",
                    params: params([("rain", washout.precipitation.into())]),
                });
            }
        }
    }

    alerts
}

/// Première heure sous zéro de la série, s'il y en a une.
fn first_freezing_hour(hours: &[HourlySample]) -> Option<i64> {
    hours.iter().find(|hour| hour.temperature <= 0.0).map(|hour| hour.time)
}

/// Première pluie significative dans les `within` heures suivant `from`.
fn rain_after(hours: &[HourlySample], from: i64, within: i64) -> Option<&HourlySample> {
    let limit = from + within * HOUR_MS;
    hours.iter().find(|hour| {
        hour.time >= from && hour.time <= limit && hour.precipitation > SPRAY_RAIN_MAX
    })
}

/// Enregistre ce qui vient d'être dit, pour ne pas le redire.
pub fn record_sent(state: &AlertState, alerts: &[Alert], sent_at: i64) -> AlertState {
    let mut next = state.clone();
    for alert in alerts {
        next.last_sent.insert(alert.kind, sent_at);
    }
    next
}

/// Retient l'état du sol pour repérer le prochain ressuyage.
pub fn record_soil(state: &AlertState, soil: SoilState) -> AlertState {
    AlertState { last_soil_state: Some(soil), ..state.clone() }
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agro::{
        DiseaseLevel, DiseasePressure, FrostRisk, FrostSeverity, SoilCondition, SprayOpportunity,
        WaterBalance, WaterStatus,
    };
    use crate::i18n::ParamValue;

    /// Le 15 avril 2026, 9 h à la parcelle.
    const NOW: i64 = 20_558 * DAY_MS + 9 * HOUR_MS;

    fn options() -> AlertOptions {
        AlertOptions::at(NOW)
    }

    fn hour_at(offset: i64) -> i64 {
        NOW + offset * HOUR_MS
    }

    fn hour(offset: i64) -> HourlySample {
        HourlySample {
            time: hour_at(offset),
            temperature: 14.0,
            weather_code: 3,
            is_day: true,
            precipitation_probability: 10.0,
            relative_humidity: 60.0,
            dew_point: 7.0,
            precipitation: 0.0,
            wind_speed: 9.0,
            wind_gusts: 15.0,
            soil_temperature_6cm: 12.0,
            soil_moisture_3to9cm: 0.22,
            et0: 0.1,
            vapour_pressure_deficit: 0.6,
        }
    }

    fn summary() -> AgroSummary {
        AgroSummary {
            water: WaterBalance {
                precipitation: 10.0,
                evapotranspiration: 8.0,
                balance: 2.0,
                status: WaterStatus::Equilibre,
                irrigation_advice: 0.0,
            },
            soil: SoilCondition {
                moisture: 0.22,
                temperature: 12.0,
                state: SoilState::Ressuye,
                trafficable: true,
                sowable: true,
            },
            disease: DiseasePressure { level: DiseaseLevel::Faible, leaf_wetness_hours: 2 },
            frost: FrostRisk {
                severity: FrostSeverity::Aucun,
                min_temperature: 6.0,
                hoar_frost: false,
            },
            gdd: 120.0,
            next_spray: None,
        }
    }

    fn spray(start_offset: i64, end_offset: i64, score: i32) -> SprayOpportunity {
        SprayOpportunity { start: hour_at(start_offset), end: hour_at(end_offset), score }
    }

    fn kinds(alerts: &[Alert]) -> Vec<AlertKind> {
        alerts.iter().map(|alert| alert.kind).collect()
    }

    fn sent(kind: AlertKind, at: i64) -> AlertState {
        record_sent(
            &AlertState::default(),
            &[Alert {
                kind,
                at: NOW,
                title_key: "",
                body_key: "",
                params: Params::new(),
            }],
            at,
        )
    }

    /* ---- la plage de silence ---- */

    #[test]
    fn la_plage_de_silence_enjambe_minuit() {
        assert!(is_quiet(22, 21, 6));
        assert!(is_quiet(3, 21, 6));
        assert!(!is_quiet(6, 21, 6));
        assert!(!is_quiet(14, 21, 6));
    }

    #[test]
    fn hors_silence_lenvoi_part_tout_de_suite() {
        let send = 20_558 * DAY_MS + 14 * HOUR_MS;
        let event = 20_558 * DAY_MS + 17 * HOUR_MS;
        assert_eq!(defer_past_quiet_hours(send, event, 21, 6), Some(send));
    }

    #[test]
    fn la_nuit_lenvoi_attend_la_reprise_du_matin() {
        let send = 20_558 * DAY_MS + 23 * HOUR_MS + 30 * 60_000;
        let event = 20_559 * DAY_MS + 10 * HOUR_MS;
        assert_eq!(
            defer_past_quiet_hours(send, event, 21, 6),
            Some(20_559 * DAY_MS + 6 * HOUR_MS)
        );
    }

    #[test]
    fn avant_laube_la_reprise_est_le_matin_meme_pas_le_lendemain() {
        let send = 20_559 * DAY_MS + 3 * HOUR_MS;
        let event = 20_559 * DAY_MS + 10 * HOUR_MS;
        assert_eq!(
            defer_past_quiet_hours(send, event, 21, 6),
            Some(20_559 * DAY_MS + 6 * HOUR_MS)
        );
    }

    #[test]
    fn un_evenement_passe_avant_la_reprise_est_abandonne_pas_retarde() {
        // Mieux vaut se taire que raconter la veille.
        let send = 20_558 * DAY_MS + 23 * HOUR_MS + 30 * 60_000;
        let event = 20_559 * DAY_MS + 4 * HOUR_MS;
        assert_eq!(defer_past_quiet_hours(send, event, 21, 6), None);
    }

    /* ---- la fenêtre de traitement ---- */

    #[test]
    fn annoncee_quand_il_reste_le_temps_de_sortir_le_pulverisateur() {
        let mut summary = summary();
        summary.next_spray = Some(spray(4, 7, 90));
        let alerts = evaluate_alerts(&summary, &[], &AlertState::default(), &options());

        assert_eq!(kinds(&alerts), vec![AlertKind::Fenetre]);
        assert_eq!(alerts[0].params, params([("score", ParamValue::Number(90.0))]));
        assert_eq!(alerts[0].at, hour_at(4));
    }

    #[test]
    fn dans_une_heure_on_se_tait() {
        // On ne sort pas le pulvérisateur en une heure.
        let mut summary = summary();
        summary.next_spray = Some(spray(1, 4, 90));
        assert!(evaluate_alerts(&summary, &[], &AlertState::default(), &options()).is_empty());
    }

    #[test]
    fn dans_trois_jours_il_est_trop_tot_pour_en_parler() {
        let mut summary = summary();
        summary.next_spray = Some(spray(72, 75, 90));
        assert!(evaluate_alerts(&summary, &[], &AlertState::default(), &options()).is_empty());
    }

    #[test]
    fn le_domaine_renvoie_des_cles_jamais_des_phrases() {
        let mut summary = summary();
        summary.next_spray = Some(spray(4, 7, 90));
        let alerts = evaluate_alerts(&summary, &[], &AlertState::default(), &options());

        assert_eq!(alerts[0].title_key, "alert.fenetre.title");
        assert_eq!(alerts[0].body_key, "alert.fenetre.body");
        assert!(!alerts[0].title_key.contains(' '));
    }

    /* ---- le gel ---- */

    #[test]
    fn le_gel_est_annonce_avec_la_temperature_attendue() {
        let mut summary = summary();
        summary.frost =
            FrostRisk { severity: FrostSeverity::Modere, min_temperature: -2.4, hoar_frost: true };

        let mut froide = hour(0);
        froide.temperature = 3.0;
        let mut gelee = hour(10);
        gelee.temperature = -1.0;

        let alerts =
            evaluate_alerts(&summary, &[froide, gelee], &AlertState::default(), &options());

        assert_eq!(kinds(&alerts), vec![AlertKind::Gel]);
        assert_eq!(alerts[0].at, hour_at(10));
        assert_eq!(alerts[0].params, params([("temperature", ParamValue::Number(-2.4))]));
    }

    #[test]
    fn pas_de_gel_pas_dalerte() {
        assert!(
            evaluate_alerts(&summary(), &[], &AlertState::default(), &options()).is_empty()
        );
    }

    /* ---- le sol devenu portant ---- */

    #[test]
    fn cest_le_passage_qui_compte_pas_letat() {
        let state = record_soil(&AlertState::default(), SoilState::Sature);
        let alerts = evaluate_alerts(&summary(), &[], &state, &options());
        assert_eq!(kinds(&alerts), vec![AlertKind::Sol]);
    }

    #[test]
    fn sans_etat_precedent_on_se_tait() {
        // Au premier lancement on ne sait pas d'où l'on vient : annoncer « le
        // sol est ressuyé » à quelqu'un dont le sol l'est depuis un mois est
        // du bruit.
        assert!(
            evaluate_alerts(&summary(), &[], &AlertState::default(), &options()).is_empty()
        );
    }

    #[test]
    fn un_sol_deja_ressuye_la_veille_ne_redeclenche_rien() {
        let state = record_soil(&AlertState::default(), SoilState::Ressuye);
        assert!(evaluate_alerts(&summary(), &[], &state, &options()).is_empty());
    }

    /* ---- la pluie lavante ---- */

    #[test]
    fn la_pluie_est_annoncee_quand_elle_tombe_dans_les_six_heures_apres_la_fenetre() {
        let mut summary = summary();
        summary.next_spray = Some(spray(4, 7, 90));
        let mut averse = hour(8);
        averse.precipitation = 2.4;

        let alerts = evaluate_alerts(&summary, &[averse], &AlertState::default(), &options());

        assert_eq!(kinds(&alerts), vec![AlertKind::Fenetre, AlertKind::Pluie]);
        assert_eq!(alerts[1].params, params([("rain", ParamValue::Number(2.4))]));
    }

    #[test]
    fn une_bruine_sous_le_seuil_ne_compte_pas() {
        let mut summary = summary();
        summary.next_spray = Some(spray(4, 7, 90));
        let mut bruine = hour(8);
        bruine.precipitation = 0.05;

        let alerts = evaluate_alerts(&summary, &[bruine], &AlertState::default(), &options());
        assert_eq!(kinds(&alerts), vec![AlertKind::Fenetre]);
    }

    #[test]
    fn une_pluie_bien_apres_la_fenetre_ne_lave_rien() {
        let mut summary = summary();
        summary.next_spray = Some(spray(4, 7, 90));
        let mut averse = hour(20);
        averse.precipitation = 4.0;

        let alerts = evaluate_alerts(&summary, &[averse], &AlertState::default(), &options());
        assert_eq!(kinds(&alerts), vec![AlertKind::Fenetre]);
    }

    /* ---- le délai de garde ---- */

    #[test]
    fn une_alerte_deja_partie_ne_repart_pas_dans_la_foulee() {
        let state = sent(AlertKind::Fenetre, NOW - HOUR_MS);
        let mut summary = summary();
        summary.next_spray = Some(spray(4, 7, 90));

        assert!(evaluate_alerts(&summary, &[], &state, &options()).is_empty());
    }

    #[test]
    fn passe_le_delai_elle_repart() {
        let state = sent(AlertKind::Fenetre, NOW - 7 * HOUR_MS);
        let mut summary = summary();
        summary.next_spray = Some(spray(4, 7, 90));

        assert_eq!(evaluate_alerts(&summary, &[], &state, &options()).len(), 1);
    }

    #[test]
    fn le_delai_de_garde_dune_nature_nen_baillonne_pas_une_autre() {
        let state = sent(AlertKind::Fenetre, NOW);
        let mut summary = summary();
        summary.next_spray = Some(spray(4, 7, 90));
        summary.frost =
            FrostRisk { severity: FrostSeverity::Faible, min_temperature: -0.5, hoar_frost: false };

        let alerts = evaluate_alerts(&summary, &[], &state, &options());
        assert_eq!(kinds(&alerts), vec![AlertKind::Gel]);
    }

    #[test]
    fn letat_retenu_naltere_pas_lancien() {
        let before = record_soil(&AlertState::default(), SoilState::Sature);
        let after = record_sent(
            &before,
            &[Alert {
                kind: AlertKind::Gel,
                at: NOW,
                title_key: "",
                body_key: "",
                params: Params::new(),
            }],
            NOW,
        );

        assert_eq!(before.last_sent.get(&AlertKind::Gel), None);
        assert_eq!(after.last_soil_state, Some(SoilState::Sature));
        assert_eq!(after.last_sent.get(&AlertKind::Gel), Some(&NOW));
    }

    /* ---- le calme est le cas normal ---- */

    #[test]
    fn une_journee_sans_rien_a_dire_ne_dit_rien() {
        let hours = [hour(0), hour(1), hour(2)];
        assert_eq!(
            evaluate_alerts(&summary(), &hours, &AlertState::default(), &options()),
            vec![]
        );
    }

    #[test]
    fn les_quatre_natures_ont_un_code_distinct() {
        let mut codes: Vec<&str> = ALERT_KINDS.iter().map(|kind| kind.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), ALERT_KINDS.len());
    }
}
