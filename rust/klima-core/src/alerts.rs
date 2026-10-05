//! Les alertes : ce que Klima dit sans qu'on ouvre l'application.
//!
//! Miroir Swift : `ios/Kliima/Models/Alerts.swift`.
//!
//! Ce module ne notifie rien. Il répond à une seule question — « qu'y a-t-il à
//! dire, maintenant ? » — à partir de la prévision et de ce qu'on a déjà dit.
//! L'acheminement (notification locale, APNs) est affaire de plateforme ; la
//! règle est affaire de domaine, et c'est ce qui permet de la tester des deux
//! côtés sans appareil.
//!
//! Ce qu'une ville a besoin de savoir avant que ça arrive : la pluie qui
//! approche, l'orage, le gel sur les trottoirs, la chaleur, le vent qui
//! arrache. Trois principes valent mieux que trente réglages :
//!
//! 1. **On ne dit que ce qui change quelque chose.** La pluie s'annonce quand
//!    elle approche et qu'il fait encore sec — pas quand il pleut déjà, ni
//!    pour demain soir.
//! 2. **On ne réveille personne.** Une alerte calculée la nuit attend le
//!    matin. Si l'événement est passé entre-temps, elle est abandonnée : mieux
//!    vaut se taire que raconter la veille.
//! 3. **On ne répète pas.** Une même nature d'alerte ne repart pas avant un
//!    délai de garde.
//!
//! Les horodatages sont ceux de la ville : « 21 h » est l'heure qu'il fait
//! là-bas, pas celle du serveur qui calcule. Le silence nocturne est donc une
//! simple division.

use std::collections::BTreeMap;

use crate::calendar::{at_midnight, hour_of};
use crate::i18n::{Params, params};
use crate::meteo::HourlySample;
use crate::ville::pluvieuse;

const HOUR_MS: i64 = 3_600_000;
const DAY_MS: i64 = 86_400_000;

/// Les seuils des alertes. Plus hauts que ceux des conseils : un conseil se
/// lit en ouvrant l'application, une alerte vient la chercher.
pub mod seuils {
    /// Une pluie s'annonce si elle commence dans les deux heures.
    pub const PLUIE_PREAVIS_H: i64 = 2;
    /// Orage, gel, chaleur, vent : on regarde les douze heures qui viennent.
    pub const HORIZON_H: usize = 12;
    /// Codes WMO d'orage.
    pub const ORAGE: [u16; 3] = [95, 96, 99];
    /// Gel des trottoirs (°C).
    pub const GEL: f64 = 0.0;
    /// Forte chaleur (°C).
    pub const CHALEUR: f64 = 33.0;
    /// Rafales dangereuses (km/h) : branches, tuiles, échafaudages.
    pub const RAFALES: f64 = 70.0;
}

use seuils::*;

/// La nature de ce qui est annoncé. Le délai de garde est tenu par nature :
/// une alerte de pluie ne bâillonne pas une alerte d'orage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AlertKind {
    Pluie,
    Orage,
    Gel,
    Chaleur,
    Vent,
}

pub const ALERT_KINDS: [AlertKind; 5] =
    [AlertKind::Pluie, AlertKind::Orage, AlertKind::Gel, AlertKind::Chaleur, AlertKind::Vent];

impl AlertKind {
    /// Nom stable, pour les journaux et l'état sérialisé.
    pub fn code(self) -> &'static str {
        match self {
            AlertKind::Pluie => "pluie",
            AlertKind::Orage => "orage",
            AlertKind::Gel => "gel",
            AlertKind::Chaleur => "chaleur",
            AlertKind::Vent => "vent",
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlertOptions {
    pub now: i64,
    /// Heure locale à partir de laquelle on se tait, et heure de reprise.
    pub quiet_from: i64,
    pub quiet_to: i64,
    /// Délai de garde entre deux alertes de même nature (heures).
    pub cooldown_hours: f64,
}

impl AlertOptions {
    /// Les réglages par défaut, à l'instant donné : silence de 22 h à 7 h,
    /// six heures de garde.
    pub fn at(now: i64) -> Self {
        AlertOptions { now, quiet_from: 22, quiet_to: 7, cooldown_hours: 6.0 }
    }
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

    let midnight = at_midnight(send);
    let day = if hour_of(send) >= to { midnight + DAY_MS } else { midnight };
    let resume = day + to * HOUR_MS;

    if resume <= event { Some(resume) } else { None }
}

/// Vrai si cette nature d'alerte est encore sous délai de garde.
fn held(kind: AlertKind, state: &AlertState, options: &AlertOptions) -> bool {
    match state.last_sent.get(&kind) {
        None => false,
        Some(last) => ((options.now - last) as f64) < options.cooldown_hours * HOUR_MS as f64,
    }
}

/// Ce qu'il y a à dire maintenant.
///
/// `hours` commence à l'heure en cours. Renvoie les alertes prêtes à partir,
/// dans l'ordre des natures. Une liste vide est le cas normal : la plupart
/// des heures n'ont rien à annoncer, et c'est ce qui rend les alertes
/// supportables.
pub fn evaluate_alerts(hours: &[HourlySample], state: &AlertState, options: &AlertOptions) -> Vec<Alert> {
    let mut alerts = Vec::new();
    let fenetre = &hours[..hours.len().min(HORIZON_H)];
    let libre = |kind| !held(kind, state, options);

    // 1. La pluie qui approche, tant qu'il fait sec.
    if libre(AlertKind::Pluie) && fenetre.first().is_some_and(|h| !pluvieuse(h)) {
        let limite = options.now + PLUIE_PREAVIS_H * HOUR_MS;
        if let Some(pluie) = fenetre.iter().find(|h| pluvieuse(h) && h.time <= limite) {
            alerts.push(Alert {
                kind: AlertKind::Pluie,
                at: pluie.time,
                title_key: "alert.pluie.title",
                body_key: "alert.pluie.body",
                params: params([("probability", pluie.precipitation_probability.into())]),
            });
        }
    }

    // 2. L'orage.
    if libre(AlertKind::Orage) {
        if let Some(orage) = fenetre.iter().find(|h| ORAGE.contains(&h.weather_code)) {
            alerts.push(Alert {
                kind: AlertKind::Orage,
                at: orage.time,
                title_key: "alert.orage.title",
                body_key: "alert.orage.body",
                params: Params::new(),
            });
        }
    }

    // 3. Le gel : trottoirs, pare-brise, plaques de verglas.
    if libre(AlertKind::Gel) {
        if let Some(premiere) = fenetre.iter().find(|h| h.temperature <= GEL) {
            let minimum = fenetre.iter().map(|h| h.temperature).fold(f64::INFINITY, f64::min);
            alerts.push(Alert {
                kind: AlertKind::Gel,
                at: premiere.time,
                title_key: "alert.gel.title",
                body_key: "alert.gel.body",
                params: params([("temperature", minimum.into())]),
            });
        }
    }

    // 4. La forte chaleur, annoncée à son heure la plus chaude.
    if libre(AlertKind::Chaleur) {
        let plus_chaude = fenetre
            .iter()
            .filter(|h| h.temperature >= CHALEUR)
            .fold(None, |m: Option<&HourlySample>, h| match m {
                Some(m) if m.temperature >= h.temperature => Some(m),
                _ => Some(h),
            });
        if let Some(h) = plus_chaude {
            alerts.push(Alert {
                kind: AlertKind::Chaleur,
                at: h.time,
                title_key: "alert.chaleur.title",
                body_key: "alert.chaleur.body",
                params: params([("temperature", h.temperature.into())]),
            });
        }
    }

    // 5. Le vent qui arrache.
    if libre(AlertKind::Vent) {
        if let Some(premiere) = fenetre.iter().find(|h| h.wind_gusts >= RAFALES) {
            let maximum = fenetre.iter().map(|h| h.wind_gusts).fold(0.0, f64::max);
            alerts.push(Alert {
                kind: AlertKind::Vent,
                at: premiere.time,
                title_key: "alert.vent.title",
                body_key: "alert.vent.body",
                params: params([("gusts", maximum.into())]),
            });
        }
    }

    alerts
}

/// Enregistre ce qui vient d'être dit, pour ne pas le redire.
pub fn record_sent(state: &AlertState, alerts: &[Alert], sent_at: i64) -> AlertState {
    let mut next = state.clone();
    for alert in alerts {
        next.last_sent.insert(alert.kind, sent_at);
    }
    next
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::ParamValue;

    /// Le 15 avril 2026, 9 h en ville.
    const NOW: i64 = 20_558 * DAY_MS + 9 * HOUR_MS;

    fn options() -> AlertOptions {
        AlertOptions::at(NOW)
    }

    fn hour(offset: i64) -> HourlySample {
        HourlySample {
            time: NOW + offset * HOUR_MS,
            weather_code: 3,
            is_day: true,
            precipitation_probability: 10.0,
            temperature: 14.0,
            apparent_temperature: 13.0,
            relative_humidity: 60.0,
            dew_point: 7.0,
            precipitation: 0.0,
            wind_speed: 9.0,
            wind_gusts: 15.0,
            uv_index: 2.0,
        }
    }

    fn hours() -> Vec<HourlySample> {
        (0..24).map(hour).collect()
    }

    fn kinds(alerts: &[Alert]) -> Vec<AlertKind> {
        alerts.iter().map(|alert| alert.kind).collect()
    }

    fn sent(kind: AlertKind, at: i64) -> AlertState {
        let alerte = Alert { kind, at: NOW, title_key: "", body_key: "", params: Params::new() };
        record_sent(&AlertState::default(), &[alerte], at)
    }

    /* ---- la plage de silence ---- */

    #[test]
    fn la_plage_de_silence_enjambe_minuit() {
        assert!(is_quiet(22, 22, 7));
        assert!(is_quiet(3, 22, 7));
        assert!(!is_quiet(7, 22, 7));
        assert!(!is_quiet(14, 22, 7));
    }

    #[test]
    fn hors_silence_lenvoi_part_tout_de_suite() {
        let send = 20_558 * DAY_MS + 14 * HOUR_MS;
        let event = 20_558 * DAY_MS + 17 * HOUR_MS;
        assert_eq!(defer_past_quiet_hours(send, event, 22, 7), Some(send));
    }

    #[test]
    fn la_nuit_lenvoi_attend_la_reprise_du_matin() {
        let send = 20_558 * DAY_MS + 23 * HOUR_MS + 30 * 60_000;
        let event = 20_559 * DAY_MS + 10 * HOUR_MS;
        assert_eq!(defer_past_quiet_hours(send, event, 22, 7), Some(20_559 * DAY_MS + 7 * HOUR_MS));
    }

    #[test]
    fn avant_laube_la_reprise_est_le_matin_meme_pas_le_lendemain() {
        let send = 20_559 * DAY_MS + 3 * HOUR_MS;
        let event = 20_559 * DAY_MS + 10 * HOUR_MS;
        assert_eq!(defer_past_quiet_hours(send, event, 22, 7), Some(20_559 * DAY_MS + 7 * HOUR_MS));
    }

    #[test]
    fn un_evenement_passe_avant_la_reprise_est_abandonne_pas_retarde() {
        let send = 20_558 * DAY_MS + 23 * HOUR_MS + 30 * 60_000;
        let event = 20_559 * DAY_MS + 4 * HOUR_MS;
        assert_eq!(defer_past_quiet_hours(send, event, 22, 7), None);
    }

    /* ---- la pluie ---- */

    #[test]
    fn la_pluie_dans_lheure_sannonce_tant_quil_fait_sec() {
        let mut h = hours();
        h[1].precipitation = 0.6;
        h[1].precipitation_probability = 80.0;
        let alerts = evaluate_alerts(&h, &AlertState::default(), &options());
        assert_eq!(kinds(&alerts), vec![AlertKind::Pluie]);
        assert_eq!(alerts[0].at, NOW + HOUR_MS);
        assert_eq!(alerts[0].params.get("probability"), Some(&ParamValue::Number(80.0)));
    }

    #[test]
    fn quand_il_pleut_deja_on_ne_lannonce_plus() {
        let mut h = hours();
        h[0].precipitation = 1.0;
        h[1].precipitation = 1.0;
        assert!(evaluate_alerts(&h, &AlertState::default(), &options()).is_empty());
    }

    #[test]
    fn la_pluie_de_ce_soir_attend_de_sapprocher() {
        let mut h = hours();
        h[3].precipitation = 2.0;
        assert!(evaluate_alerts(&h, &AlertState::default(), &options()).is_empty());
    }

    /* ---- orage, gel, chaleur, vent ---- */

    #[test]
    fn lorage_sannonce_a_son_heure() {
        let mut h = hours();
        h[5].weather_code = 95;
        let alerts = evaluate_alerts(&h, &AlertState::default(), &options());
        assert_eq!(kinds(&alerts), vec![AlertKind::Orage]);
        assert_eq!(alerts[0].at, NOW + 5 * HOUR_MS);
    }

    #[test]
    fn le_gel_dit_le_minimum_attendu() {
        let mut h = hours();
        h[8].temperature = -0.5;
        h[10].temperature = -3.0;
        let alerts = evaluate_alerts(&h, &AlertState::default(), &options());
        assert_eq!(kinds(&alerts), vec![AlertKind::Gel]);
        assert_eq!(alerts[0].at, NOW + 8 * HOUR_MS);
        assert_eq!(alerts[0].params.get("temperature"), Some(&ParamValue::Number(-3.0)));
    }

    #[test]
    fn la_chaleur_sannonce_a_lheure_la_plus_chaude() {
        let mut h = hours();
        h[4].temperature = 33.0;
        h[6].temperature = 36.0;
        h[7].temperature = 34.0;
        let alerts = evaluate_alerts(&h, &AlertState::default(), &options());
        assert_eq!(kinds(&alerts), vec![AlertKind::Chaleur]);
        assert_eq!(alerts[0].at, NOW + 6 * HOUR_MS);
        assert_eq!(alerts[0].params.get("temperature"), Some(&ParamValue::Number(36.0)));
    }

    #[test]
    fn le_vent_dit_sa_rafale_la_plus_forte() {
        let mut h = hours();
        h[2].wind_gusts = 72.0;
        h[3].wind_gusts = 90.0;
        let alerts = evaluate_alerts(&h, &AlertState::default(), &options());
        assert_eq!(kinds(&alerts), vec![AlertKind::Vent]);
        assert_eq!(alerts[0].at, NOW + 2 * HOUR_MS);
        assert_eq!(alerts[0].params.get("gusts"), Some(&ParamValue::Number(90.0)));
    }

    #[test]
    fn au_dela_de_douze_heures_il_est_trop_tot() {
        let mut h = hours();
        h[12].weather_code = 95;
        h[13].temperature = -2.0;
        h[14].wind_gusts = 100.0;
        assert!(evaluate_alerts(&h, &AlertState::default(), &options()).is_empty());
    }

    #[test]
    fn une_journee_sans_rien_a_dire_ne_dit_rien() {
        assert!(evaluate_alerts(&hours(), &AlertState::default(), &options()).is_empty());
        assert!(evaluate_alerts(&[], &AlertState::default(), &options()).is_empty());
    }

    #[test]
    fn le_domaine_renvoie_des_cles_jamais_des_phrases() {
        let mut h = hours();
        h[5].weather_code = 95;
        let alerts = evaluate_alerts(&h, &AlertState::default(), &options());
        assert_eq!(alerts[0].title_key, "alert.orage.title");
        assert_eq!(alerts[0].body_key, "alert.orage.body");
    }

    /* ---- le délai de garde ---- */

    #[test]
    fn une_alerte_deja_partie_ne_repart_pas_dans_la_foulee() {
        let mut h = hours();
        h[5].weather_code = 95;
        let state = sent(AlertKind::Orage, NOW - 2 * HOUR_MS);
        assert!(evaluate_alerts(&h, &state, &options()).is_empty());
    }

    #[test]
    fn passe_le_delai_elle_repart() {
        let mut h = hours();
        h[5].weather_code = 95;
        let state = sent(AlertKind::Orage, NOW - 7 * HOUR_MS);
        assert_eq!(kinds(&evaluate_alerts(&h, &state, &options())), vec![AlertKind::Orage]);
    }

    #[test]
    fn le_delai_de_garde_dune_nature_nen_baillonne_pas_une_autre() {
        let mut h = hours();
        h[5].weather_code = 95;
        h[8].temperature = -1.0;
        let state = sent(AlertKind::Orage, NOW - HOUR_MS);
        assert_eq!(kinds(&evaluate_alerts(&h, &state, &options())), vec![AlertKind::Gel]);
    }

    #[test]
    fn letat_retenu_naltere_pas_lancien() {
        let avant = AlertState::default();
        let apres = sent(AlertKind::Pluie, NOW);
        assert!(avant.last_sent.is_empty());
        assert_eq!(apres.last_sent.get(&AlertKind::Pluie), Some(&NOW));
    }

    #[test]
    fn les_cinq_natures_ont_un_code_distinct() {
        let codes: std::collections::BTreeSet<&str> = ALERT_KINDS.iter().map(|k| k.code()).collect();
        assert_eq!(codes.len(), 5);
    }
}
