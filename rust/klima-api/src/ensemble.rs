//! Les séries de toutes les sources, pour le recoupement de la prévision.
//!
//! Deux lectures, une même forme en sortie (`SerieSource`) :
//!
//! - **Open-Meteo**, sept modèles en une requête : le paramètre `models`
//!   suffixe chaque colonne de l'identifiant du modèle. La même réponse sert
//!   aussi l'accord des sources (`readings::decode_open_meteo`) : elle porte
//!   la température, la pluie et le vent de chaque modèle.
//! - **MET Norway**, dont la série est horaire sur deux jours et demi, puis
//!   tous les six heures. On ne garde que les pas horaires — ceux qui ont une
//!   pluie sur l'heure —, à l'heure de la ville.
//!
//! Comme ailleurs, construire l'adresse et lire la réponse sont séparés de
//! l'appel : le décodage se teste sur une réponse enregistrée.

use klima_core::endpoints::Endpoints;
use klima_core::fusion::{HeureSource, JourSource, SerieSource};
use klima_core::position::Parcelle;
use klima_core::providers::{MET_NORWAY_SOURCE, OPEN_METEO_SOURCES};
use serde_json::Value;

use crate::open_meteo::{parse_stamp, query};

/// Ce que chaque modèle donne, heure par heure.
pub const HEURES: &str = "temperature_2m,apparent_temperature,precipitation,\
precipitation_probability,weather_code,wind_speed_10m,wind_gusts_10m";

/// Ce que chaque modèle donne, jour par jour.
pub const JOURS: &str = "temperature_2m_max,temperature_2m_min,precipitation_sum,\
precipitation_probability_max,wind_gusts_10m_max,weather_code";

/// L'adresse des sept modèles, sur `days` jours.
pub fn ensemble_url(endpoints: &Endpoints, parcelle: &Parcelle, days: u32) -> String {
    let modeles: Vec<&str> = OPEN_METEO_SOURCES.iter().map(|s| s.id).collect();
    let pairs = [
        ("latitude", format!("{:.4}", parcelle.latitude)),
        ("longitude", format!("{:.4}", parcelle.longitude)),
        ("hourly", HEURES.to_owned()),
        ("daily", JOURS.to_owned()),
        ("models", modeles.join(",")),
        ("wind_speed_unit", "kmh".to_owned()),
        ("timezone", "auto".to_owned()),
        ("forecast_days", days.to_string()),
    ];
    format!("{}?{}", endpoints.open_meteo_forecast, query(&pairs))
}

/// Une valeur d'une colonne suffixée, `None` si absente ou nulle.
fn valeur(bloc: &Value, variable: &str, modele: &str, index: usize) -> Option<f64> {
    bloc[format!("{variable}_{modele}")].as_array()?.get(index)?.as_f64().filter(|v| v.is_finite())
}

/// Lit la réponse des sept modèles : une série par modèle qui a répondu.
///
/// Un modèle dont toutes les températures sont nulles ne couvre pas le point :
/// il est écarté, pas compté pour zéro.
pub fn decode_ensemble(body: &str) -> Vec<SerieSource> {
    let Ok(payload) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };
    let heures = &payload["hourly"];
    let jours = &payload["daily"];
    let temps: Vec<Option<i64>> = heures["time"]
        .as_array()
        .map(|t| t.iter().map(|s| s.as_str().and_then(parse_stamp)).collect())
        .unwrap_or_default();
    let dates: Vec<Option<i64>> = jours["time"]
        .as_array()
        .map(|t| t.iter().map(|s| s.as_str().and_then(parse_stamp)).collect())
        .unwrap_or_default();

    OPEN_METEO_SOURCES
        .iter()
        .map(|source| {
            let m = source.id;
            let heures: Vec<HeureSource> = temps
                .iter()
                .enumerate()
                .filter_map(|(i, t)| {
                    Some(HeureSource {
                        time: (*t)?,
                        temperature: valeur(heures, "temperature_2m", m, i),
                        ressenti: valeur(heures, "apparent_temperature", m, i),
                        precipitation: valeur(heures, "precipitation", m, i),
                        probabilite: valeur(heures, "precipitation_probability", m, i),
                        vent: valeur(heures, "wind_speed_10m", m, i),
                        rafales: valeur(heures, "wind_gusts_10m", m, i),
                        code: valeur(heures, "weather_code", m, i).map(|c| c as u16),
                    })
                })
                .collect();
            let jours: Vec<JourSource> = dates
                .iter()
                .enumerate()
                .filter_map(|(i, d)| {
                    Some(JourSource {
                        date: (*d)?,
                        minimum: valeur(jours, "temperature_2m_min", m, i),
                        maximum: valeur(jours, "temperature_2m_max", m, i),
                        cumul: valeur(jours, "precipitation_sum", m, i),
                        probabilite: valeur(jours, "precipitation_probability_max", m, i),
                        rafales: valeur(jours, "wind_gusts_10m_max", m, i),
                        code: valeur(jours, "weather_code", m, i).map(|c| c as u16),
                    })
                })
                .collect();
            SerieSource { source_id: m.to_owned(), heures, jours }
        })
        .filter(|serie| serie.heures.iter().any(|h| h.temperature.is_some()))
        .collect()
}

/// Le code météo d'un symbole de MET Norway (« lightrainshowers_day »…).
///
/// La table suit leur liste de symboles ; l'orage l'emporte sur tout le reste,
/// comme dans le code de l'Organisation météorologique mondiale.
pub fn code_du_symbole(symbole: &str) -> Option<u16> {
    let nom = symbole.split('_').next()?;
    if nom.contains("thunder") {
        return Some(95);
    }
    Some(match nom {
        "clearsky" => 0,
        "fair" => 1,
        "partlycloudy" => 2,
        "cloudy" => 3,
        "fog" => 45,
        "lightrain" => 61,
        "rain" => 63,
        "heavyrain" => 65,
        "lightrainshowers" => 80,
        "rainshowers" => 81,
        "heavyrainshowers" => 82,
        "lightsleet" | "sleet" | "heavysleet" => 66,
        "lightsleetshowers" | "sleetshowers" | "heavysleetshowers" => 66,
        "lightsnow" => 71,
        "snow" => 73,
        "heavysnow" => 75,
        "lightsnowshowers" | "snowshowers" => 85,
        "heavysnowshowers" => 86,
        _ => return None,
    })
}

/// Lit la série de MET Norway, à l'heure de la ville (`utc_offset_seconds`).
///
/// Seuls les pas qui portent une pluie sur l'heure entrent : les pas de six
/// heures, plus loin, n'en ont pas, et une pluie de six heures n'est pas une
/// pluie horaire. MET Norway donne le vent en m/s ; Klima raisonne en km/h.
pub fn serie_met_norway(body: &str, utc_offset_seconds: i64) -> Option<SerieSource> {
    let payload: Value = serde_json::from_str(body).ok()?;
    let pas = payload["properties"]["timeseries"].as_array()?;
    let heures: Vec<HeureSource> = pas
        .iter()
        .filter_map(|entree| {
            let heure = &entree["data"]["next_1_hours"];
            let pluie = heure["details"]["precipitation_amount"].as_f64()?;
            let stamp = entree["time"].as_str()?;
            let utc = parse_stamp(stamp.strip_suffix('Z').unwrap_or(stamp))?;
            let instant = &entree["data"]["instant"]["details"];
            Some(HeureSource {
                time: utc + utc_offset_seconds * 1000,
                temperature: instant["air_temperature"].as_f64().filter(|t| t.is_finite()),
                precipitation: Some(pluie),
                vent: instant["wind_speed"].as_f64().map(|v| v * 3.6),
                code: heure["summary"]["symbol_code"].as_str().and_then(code_du_symbole),
                ..Default::default()
            })
        })
        .collect();
    heures.iter().any(|h| h.temperature.is_some()).then(|| SerieSource {
        source_id: MET_NORWAY_SOURCE.id.to_owned(),
        heures,
        jours: Vec::new(),
    })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use klima_core::calendar::civil;

    const REPONSE: &str = r#"{
      "timezone": "Europe/Paris", "utc_offset_seconds": 7200,
      "hourly": {
        "time": ["2026-05-12T12:00", "2026-05-12T13:00"],
        "temperature_2m_meteofrance_seamless": [19.5, 20.1],
        "precipitation_meteofrance_seamless": [0.0, 0.3],
        "weather_code_meteofrance_seamless": [2, 61],
        "precipitation_probability_meteofrance_seamless": [null, null],
        "temperature_2m_jma_seamless": [15.5, null],
        "wind_gusts_10m_jma_seamless": [null, null],
        "temperature_2m_gem_seamless": [null, null]
      },
      "daily": {
        "time": ["2026-05-12"],
        "temperature_2m_max_meteofrance_seamless": [22.0],
        "temperature_2m_min_meteofrance_seamless": [11.0],
        "precipitation_sum_meteofrance_seamless": [1.2]
      }
    }"#;

    #[test]
    fn une_serie_par_modele_qui_couvre_le_point() {
        let series = decode_ensemble(REPONSE);
        let ids: Vec<&str> = series.iter().map(|s| s.source_id.as_str()).collect();
        assert_eq!(ids, ["meteofrance_seamless", "jma_seamless"], "GEM, tout à null, est écarté");

        let mf = &series[0];
        assert_eq!(mf.heures[0].time, civil(2026, 5, 12, 12, 0));
        assert_eq!(mf.heures[1].temperature, Some(20.1));
        assert_eq!(mf.heures[1].code, Some(61));
        assert_eq!(mf.heures[0].probabilite, None, "un risque nul n'est pas un risque à zéro");
        assert_eq!(mf.jours[0].maximum, Some(22.0));
        assert_eq!(mf.jours[0].cumul, Some(1.2));

        let jma = &series[1];
        assert_eq!(jma.heures[1].temperature, None);
        assert_eq!(jma.heures[0].rafales, None);
    }

    #[test]
    fn une_reponse_illisible_ne_donne_rien() {
        assert!(decode_ensemble("pas du json").is_empty());
        assert!(decode_ensemble("{}").is_empty());
    }

    #[test]
    fn l_adresse_demande_les_sept_modeles_heures_et_jours() {
        let paris = Parcelle { name: "Paris".into(), latitude: 48.8566, longitude: 2.3522, admin: None, country: None };
        let url = ensemble_url(&Endpoints::direct(), &paris, 7);
        assert!(url.contains("models=meteofrance_seamless%2Cecmwf_ifs025%2Cicon_seamless%2Cgfs_seamless%2Cukmo_seamless%2Cgem_seamless%2Cjma_seamless"), "{url}");
        assert!(url.contains("hourly=temperature_2m%2Capparent_temperature"));
        assert!(url.contains("daily=temperature_2m_max"));
        assert!(url.contains("forecast_days=7"));
    }

    #[test]
    fn les_symboles_de_met_norway_deviennent_des_codes() {
        assert_eq!(code_du_symbole("clearsky_day"), Some(0));
        assert_eq!(code_du_symbole("lightrainshowers_night"), Some(80));
        assert_eq!(code_du_symbole("heavyrainandthunder"), Some(95));
        assert_eq!(code_du_symbole("snow"), Some(73));
        assert_eq!(code_du_symbole("inconnu"), None);
    }

    #[test]
    fn met_norway_a_l_heure_de_la_ville_et_en_km_h() {
        let corps = r#"{"properties":{"timeseries":[
          {"time":"2026-05-12T10:00:00Z","data":{"instant":{"details":{"air_temperature":18.2,"wind_speed":5.0}},
            "next_1_hours":{"summary":{"symbol_code":"lightrain"},"details":{"precipitation_amount":0.4}}}},
          {"time":"2026-05-15T12:00:00Z","data":{"instant":{"details":{"air_temperature":14.0,"wind_speed":2.0}},
            "next_6_hours":{"summary":{"symbol_code":"cloudy"},"details":{"precipitation_amount":0.0}}}}
        ]}}"#;
        let serie = serie_met_norway(corps, 7200).unwrap();
        assert_eq!(serie.source_id, MET_NORWAY_SOURCE.id);
        assert_eq!(serie.heures.len(), 1, "le pas de six heures n'entre pas");
        let h = &serie.heures[0];
        assert_eq!(h.time, civil(2026, 5, 12, 12, 0), "10 h UTC, midi à Paris");
        assert_eq!(h.temperature, Some(18.2));
        assert_eq!(h.vent, Some(18.0));
        assert_eq!(h.code, Some(61));
        assert_eq!(h.precipitation, Some(0.4));
        assert!(serie_met_norway("{}", 0).is_none());
    }
}
