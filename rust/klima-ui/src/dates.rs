//! Les dates, telles que le navigateur les écrit.
//!
//! Ici — et seulement ici — Klima appelle `Intl`. Les nombres s'en passent :
//! `klima_core::format` écrit à la main les règles des trois langues, pour que
//! le cœur tourne aussi sur un serveur. Les noms de jours, eux, sont des
//! données de locale : les recopier pour trois langues serait une table à
//! maintenir alors que le navigateur en a déjà une juste.
//!
//! Un piège, et il vient de la convention du dépôt : les horodatages sont ceux
//! **de la parcelle**, pas des instants absolus. On les formate donc en UTC —
//! l'heure murale du champ est déjà dedans. Formater avec le fuseau de la
//! parcelle décalerait une deuxième fois.

use js_sys::{Array, Date, Intl, Object, Reflect};
use wasm_bindgen::JsValue;

/// Options d'un formateur : `{ hour: "numeric", timeZone: "UTC" }`.
fn options(paires: &[(&str, &str)]) -> Object {
    let options = Object::new();
    for (cle, valeur) in paires {
        let _ = Reflect::set(&options, &JsValue::from_str(cle), &JsValue::from_str(valeur));
    }
    let _ = Reflect::set(&options, &JsValue::from_str("timeZone"), &JsValue::from_str("UTC"));
    options
}

fn formate(ms: i64, locale: &str, paires: &[(&str, &str)]) -> String {
    let locales = Array::of1(&JsValue::from_str(locale));
    let formateur = Intl::DateTimeFormat::new(&locales, &options(paires));
    let date = Date::new(&JsValue::from_f64(ms as f64));

    formateur
        .format()
        .call1(&formateur, &date)
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

/// « 14 h » ou « 2 PM », selon la langue.
pub fn heure(ms: i64, locale: &str) -> String {
    formate(ms, locale, &[("hour", "numeric")])
}

/// « 14:05 » — l'heure et la minute, pour un lever de soleil.
pub fn heure_minute(ms: i64, locale: &str) -> String {
    formate(ms, locale, &[("hour", "2-digit"), ("minute", "2-digit")])
}

/// « lun. » — le jour de la semaine, en abrégé.
pub fn jour_court(ms: i64, locale: &str) -> String {
    formate(ms, locale, &[("weekday", "short")])
}

/// « lundi 12 mai » — la date en toutes lettres, pour une section du jour.
pub fn jour_complet(ms: i64, locale: &str) -> String {
    formate(ms, locale, &[("weekday", "long"), ("day", "numeric"), ("month", "long")])
}

/// « lun. 15 h » — le jour et l'heure, pour une fenêtre de traitement.
pub fn jour_et_heure(ms: i64, locale: &str) -> String {
    formate(ms, locale, &[("weekday", "short"), ("hour", "numeric")])
}

/// Met la première lettre en capitale.
///
/// Les noms de jours abrégés sortent en minuscule en français et en espagnol ;
/// en tête de ligne, ils se portent mieux avec une capitale.
pub fn capitale(valeur: &str) -> String {
    let mut lettres = valeur.chars();
    match lettres.next() {
        Some(premiere) => premiere.to_uppercase().collect::<String>() + lettres.as_str(),
        None => String::new(),
    }
}
