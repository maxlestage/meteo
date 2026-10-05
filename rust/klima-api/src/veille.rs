//! La prévision au quart d'heure, pour le guetteur : l'adresse, et la lecture.
//!
//! Un appel à part de la prévision horaire, et c'est voulu : celle-ci se
//! garde une heure, celle-là un quart d'heure. Les mêler ferait relire toute
//! la semaine pour savoir s'il pleut dans vingt minutes, ou laisser vieillir
//! d'une heure ce qui ne vaut que pour la demi-heure.
//!
//! On demande le quart entamé (`past_minutely_15=1`) et les dix suivants :
//! deux heures et une marge, quelle que soit la façon dont le service arrondit
//! « maintenant ».

use klima_core::endpoints::Endpoints;
use klima_core::position::Parcelle;
use klima_core::veille::QuartSample;
use serde_json::Value;

use crate::open_meteo::{column, parse_stamp, query};

/// Les variables lues au quart d'heure.
pub const QUART_VARIABLES: &str =
    "precipitation,weather_code,temperature_2m,apparent_temperature,wind_gusts_10m,is_day";

/// Ce que le service a répondu.
#[derive(Debug, Clone, PartialEq)]
pub struct Quarts {
    /// Décalage du fuseau de la ville, en secondes : de quoi savoir quel
    /// quart est « maintenant » là-bas.
    pub utc_offset_seconds: i64,
    pub quarts: Vec<QuartSample>,
}

/// L'adresse de la prévision au quart d'heure d'une ville.
pub fn quarts_url(endpoints: &Endpoints, parcelle: &Parcelle) -> String {
    let pairs = [
        ("latitude", format!("{:.4}", parcelle.latitude)),
        ("longitude", format!("{:.4}", parcelle.longitude)),
        ("minutely_15", QUART_VARIABLES.to_owned()),
        ("past_minutely_15", "1".to_owned()),
        ("forecast_minutely_15", "10".to_owned()),
        ("wind_speed_unit", "kmh".to_owned()),
        ("timezone", "auto".to_owned()),
    ];
    format!("{}?{}", endpoints.open_meteo_forecast, query(&pairs))
}

/// Lit la réponse. `None` si elle n'a pas de série au quart d'heure : le
/// guetteur se tait plutôt que de dire « sec » sur une réponse vide.
pub fn decode_quarts(body: &str) -> Option<Quarts> {
    let payload: Value = serde_json::from_str(body).ok()?;
    let block = &payload["minutely_15"];
    let times = block["time"].as_array()?;
    let length = times.len();
    let c = |key: &str| column(block, key, length);

    let precipitation = c("precipitation");
    let weather_code = c("weather_code");
    let temperature = c("temperature_2m");
    let apparent = c("apparent_temperature");
    let gusts = c("wind_gusts_10m");
    let is_day = c("is_day");

    let quarts: Vec<QuartSample> = times
        .iter()
        .enumerate()
        .filter_map(|(i, stamp)| {
            Some(QuartSample {
                time: stamp.as_str().and_then(parse_stamp)?,
                precipitation: precipitation[i],
                weather_code: weather_code[i] as u16,
                temperature: temperature[i],
                apparent_temperature: apparent[i],
                wind_gusts: gusts[i],
                is_day: is_day[i] != 0.0,
            })
        })
        .collect();
    if quarts.is_empty() {
        return None;
    }

    Some(Quarts { utc_offset_seconds: payload["utc_offset_seconds"].as_i64().unwrap_or(0), quarts })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use klima_core::calendar::civil;

    const REPONSE: &str = r#"{
      "latitude": 48.86, "longitude": 2.34, "utc_offset_seconds": 7200,
      "timezone": "Europe/Paris",
      "minutely_15": {
        "time": ["2026-05-12T16:00", "2026-05-12T16:15", "2026-05-12T16:30"],
        "precipitation": [0.0, 0.4, null],
        "weather_code": [3, 61, 95],
        "temperature_2m": [16.2, 15.8, 15.1],
        "apparent_temperature": [15.0, 14.1, 13.9],
        "wind_gusts_10m": [22.0, 31.0, 58.0],
        "is_day": [1, 1, 1]
      }
    }"#;

    #[test]
    fn lit_la_serie_au_quart_d_heure() {
        let lu = decode_quarts(REPONSE).unwrap();
        assert_eq!(lu.utc_offset_seconds, 7200);
        assert_eq!(lu.quarts.len(), 3);
        assert_eq!(lu.quarts[0].time, civil(2026, 5, 12, 16, 0));
        assert_eq!(lu.quarts[1].time - lu.quarts[0].time, 900_000);
        assert_eq!(lu.quarts[1].precipitation, 0.4);
        assert_eq!(lu.quarts[2].weather_code, 95);
        assert_eq!(lu.quarts[2].wind_gusts, 58.0);
        // Un `null` se lit 0, sans décaler la série.
        assert_eq!(lu.quarts[2].precipitation, 0.0);
        assert!(lu.quarts[0].is_day);
    }

    #[test]
    fn une_reponse_sans_serie_fait_taire_le_guetteur() {
        assert_eq!(decode_quarts(r#"{"latitude": 1}"#), None);
        assert_eq!(decode_quarts(r#"{"minutely_15": {"time": []}}"#), None);
        assert_eq!(decode_quarts("pas du json"), None);
    }

    #[test]
    fn l_adresse_ne_demande_que_les_quarts() {
        let paris = Parcelle {
            name: "Paris".into(),
            latitude: 48.8566,
            longitude: 2.3522,
            admin: None,
            country: None,
        };
        let url = quarts_url(&Endpoints::direct(), &paris);
        assert!(url.starts_with("https://api.open-meteo.com/v1/forecast?latitude=48.8566&longitude=2.3522"));
        assert!(url.contains("minutely_15=precipitation%2Cweather_code"));
        assert!(url.contains("past_minutely_15=1"));
        assert!(url.contains("forecast_minutely_15=10"));
        assert!(!url.contains("hourly="), "{url}");
        assert!(!url.contains("daily="), "{url}");
    }
}
