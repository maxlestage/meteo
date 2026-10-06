//! Ce que chaque fournisseur répond, et comment on le lit.
//!
//! Trois services, trois formes de réponse, un seul type en sortie :
//! `SourceReading`. C'est ce qui permet au recoupement de comparer des choses
//! comparables — et de ne pas savoir d'où elles viennent.
//!
//! Comme pour la prévision, construire l'adresse et lire la réponse sont
//! séparés de l'appel. La conséquence pratique : ces décodages se testent sur
//! des réponses enregistrées, et le même code sert au relais et au navigateur.
//!
//! Une règle revient trois fois, et c'est la plus importante : **une source
//! sans température est écartée, pas comptée pour zéro.** Un modèle qui ne
//! couvre pas la parcelle, une station hors de portée, une colonne absente —
//! dans les trois cas, tirer la moyenne vers zéro inventerait un désaccord.

use klima_core::endpoints::{Endpoints, Transport};
use klima_core::providers::{
    BRIGHT_SKY_SOURCE, MET_NORWAY_SOURCE, OPEN_METEO_SOURCES, SourceReading, USER_AGENT,
};
use serde_json::Value;

const HOUR_MS: i64 = 3_600_000;

/// Les variables qu'on demande pour comparer : le strict nécessaire.
const READING_VARIABLES: &str = "temperature_2m,precipitation,wind_speed_10m";

/// Une requête de relevé : l'adresse, et l'en-tête à poser s'il y en a un.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadingCall {
    pub url: String,
    /// `Some` seulement en appel direct chez MET Norway : par le relais, c'est
    /// le serveur qui se nomme, et un navigateur qui ajouterait l'en-tête
    /// verrait sa requête refusée avant d'être envoyée.
    pub user_agent: Option<&'static str>,
}

fn point(url: &str, noms: (&str, &str), latitude: f64, longitude: f64) -> String {
    let (lat, lon) = noms;
    let separateur = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separateur}{lat}={latitude:.4}&{lon}={longitude:.4}")
}

/// Une seule requête suffit pour les sept modèles : le paramètre `models`
/// suffixe chaque colonne de l'identifiant du modèle.
pub fn open_meteo_call(endpoints: &Endpoints, latitude: f64, longitude: f64) -> ReadingCall {
    let modeles: Vec<&str> = OPEN_METEO_SOURCES.iter().map(|s| s.id).collect();
    let url = format!(
        "{}&hourly={READING_VARIABLES}&models={}&wind_speed_unit=kmh&timezone=auto&forecast_days=1",
        point(&endpoints.open_meteo_forecast, ("latitude", "longitude"), latitude, longitude),
        modeles.join(",")
    );
    ReadingCall { url, user_agent: None }
}

pub fn met_norway_call(endpoints: &Endpoints, latitude: f64, longitude: f64) -> ReadingCall {
    ReadingCall {
        url: point(&endpoints.met_norway, ("lat", "lon"), latitude, longitude),
        user_agent: match endpoints.transport {
            Transport::Direct => Some(USER_AGENT),
            Transport::Relais => None,
        },
    }
}

pub fn bright_sky_call(endpoints: &Endpoints, latitude: f64, longitude: f64) -> ReadingCall {
    ReadingCall {
        url: point(&endpoints.bright_sky, ("lat", "lon"), latitude, longitude),
        user_agent: None,
    }
}

/* -------------------- lecture des réponses -------------------- */

/// Lit les colonnes suffixées par modèle, à l'heure en cours.
pub fn decode_open_meteo(body: &str, now: i64) -> Vec<SourceReading> {
    let Ok(payload): Result<Value, _> = serde_json::from_str(body) else {
        return Vec::new();
    };
    let hourly = &payload["hourly"];
    let Some(times) = hourly["time"].as_array() else {
        return Vec::new();
    };

    let Some(index) = current_hour_index(times, now) else {
        return Vec::new();
    };

    OPEN_METEO_SOURCES
        .iter()
        .filter_map(|source| {
            // Une source sans température ne couvre pas la parcelle : on
            // l'écarte plutôt que de la compter pour zéro.
            let temperature = colonne(hourly, &format!("temperature_2m_{}", source.id), index)?;
            Some(SourceReading {
                source: source.clone(),
                temperature,
                precipitation: colonne(hourly, &format!("precipitation_{}", source.id), index)
                    .unwrap_or(0.0),
                wind_speed: colonne(hourly, &format!("wind_speed_10m_{}", source.id), index)
                    .unwrap_or(0.0),
            })
        })
        .collect()
}

/// Prend l'échéance la plus proche de maintenant et convertit le vent.
///
/// La série de MET Norway est horaire au début, trihoraire ensuite : « la plus
/// proche » est donc plus juste que « la première ».
pub fn decode_met_norway(body: &str, now: i64) -> Vec<SourceReading> {
    let Ok(payload): Result<Value, _> = serde_json::from_str(body) else {
        return Vec::new();
    };
    let Some(series) = payload["properties"]["timeseries"].as_array() else {
        return Vec::new();
    };

    let proche = series
        .iter()
        .filter_map(|entry| {
            let time = parse_rfc3339(entry["time"].as_str()?)?;
            Some(((time - now).abs(), entry))
        })
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, entry)| entry);

    let Some(entry) = proche else { return Vec::new() };
    let instant = &entry["data"]["instant"]["details"];

    let Some(temperature) = instant["air_temperature"].as_f64().filter(|t| t.is_finite()) else {
        return Vec::new();
    };

    vec![SourceReading {
        source: MET_NORWAY_SOURCE,
        temperature,
        precipitation: entry["data"]["next_1_hours"]["details"]["precipitation_amount"]
            .as_f64()
            .unwrap_or(0.0),
        // MET Norway donne le vent en m/s ; Klima raisonne en km/h.
        wind_speed: instant["wind_speed"].as_f64().unwrap_or(0.0) * 3.6,
    }]
}

/// Lit l'observation de la station.
///
/// Un corps vide — ce que renvoie un 404 — vaut absence de station à portée.
/// Ce n'est pas une panne : c'est une absence, et elle sort du recoupement
/// sans rien casser.
pub fn decode_bright_sky(body: &str) -> Vec<SourceReading> {
    let Ok(payload): Result<Value, _> = serde_json::from_str(body) else {
        return Vec::new();
    };
    let weather = &payload["weather"];

    let Some(temperature) = weather["temperature"].as_f64().filter(|t| t.is_finite()) else {
        return Vec::new();
    };

    vec![SourceReading {
        source: BRIGHT_SKY_SOURCE,
        temperature,
        precipitation: nombre_ou_zero(&weather["precipitation"]),
        wind_speed: nombre_ou_zero(&weather["wind_speed"]),
    }]
}

/// Première heure de la série postérieure ou égale à l'heure en cours ; la
/// dernière à défaut.
fn current_hour_index(times: &[Value], now: i64) -> Option<usize> {
    let debut = now.div_euclid(HOUR_MS) * HOUR_MS;
    let trouvee = times.iter().position(|stamp| {
        stamp.as_str().and_then(crate::open_meteo::parse_stamp).is_some_and(|t| t >= debut)
    });
    trouvee.or_else(|| times.len().checked_sub(1))
}

fn colonne(block: &Value, key: &str, index: usize) -> Option<f64> {
    block[key].as_array()?.get(index)?.as_f64().filter(|v| v.is_finite())
}

fn nombre_ou_zero(value: &Value) -> f64 {
    value.as_f64().filter(|v| v.is_finite()).unwrap_or(0.0)
}

/// Lit un horodatage RFC 3339 en UTC, tel que MET Norway les écrit
/// (« 2026-05-12T21:00:00Z »).
fn parse_rfc3339(stamp: &str) -> Option<i64> {
    let sans_zone = stamp.strip_suffix('Z').unwrap_or(stamp);
    crate::open_meteo::parse_stamp(sans_zone)
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use klima_core::calendar::civil;
    use klima_core::providers::attributions_for;

    fn maintenant() -> i64 {
        civil(2026, 5, 12, 14, 0)
    }

    /// Trois heures autour de maintenant, au format d'Open-Meteo.
    fn payload_open_meteo(colonnes: &[(&str, [Option<f64>; 3])]) -> String {
        let mut hourly = serde_json::Map::new();
        hourly.insert(
            "time".to_owned(),
            serde_json::json!(["2026-05-12T13:00", "2026-05-12T14:00", "2026-05-12T15:00"]),
        );
        for (id, valeurs) in colonnes {
            hourly.insert(format!("temperature_2m_{id}"), serde_json::json!(valeurs.to_vec()));
            hourly.insert(format!("precipitation_{id}"), serde_json::json!([0.0, 0.0, 0.0]));
            hourly.insert(format!("wind_speed_10m_{id}"), serde_json::json!([12.0, 12.0, 12.0]));
        }
        serde_json::json!({ "utc_offset_seconds": 7200, "hourly": hourly }).to_string()
    }

    /* ---- Open-Meteo ---- */

    #[test]
    fn lit_les_colonnes_suffixees_par_modele_a_lheure_en_cours() {
        let body = payload_open_meteo(&[
            ("meteofrance_seamless", [Some(10.0), Some(18.2), Some(19.0)]),
            ("ecmwf_ifs025", [Some(10.0), Some(18.6), Some(19.0)]),
            ("icon_seamless", [Some(10.0), Some(19.0), Some(19.0)]),
            ("gfs_seamless", [Some(10.0), Some(18.4), Some(19.0)]),
        ]);

        let readings = decode_open_meteo(&body, maintenant());
        assert_eq!(readings.len(), 4);

        let mut temperatures: Vec<f64> = readings.iter().map(|r| r.temperature).collect();
        temperatures.sort_by(f64::total_cmp);
        assert_eq!(temperatures, [18.2, 18.4, 18.6, 19.0]);
        assert!(readings.iter().all(|r| r.source.provider == "open-meteo"));
    }

    #[test]
    fn ecarte_un_modele_sans_valeur_plutot_que_de_compter_zero() {
        let body = payload_open_meteo(&[
            ("meteofrance_seamless", [Some(10.0), Some(18.0), Some(19.0)]),
            ("ecmwf_ifs025", [None, None, None]),
        ]);

        let readings = decode_open_meteo(&body, maintenant());
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].source.institution, "Météo-France");
    }

    #[test]
    fn une_reponse_illisible_ne_donne_aucun_releve() {
        assert!(decode_open_meteo("<html>502</html>", maintenant()).is_empty());
        assert!(decode_met_norway("", maintenant()).is_empty());
        assert!(decode_bright_sky("").is_empty());
    }

    /* ---- MET Norway ---- */

    #[test]
    fn prend_lecheance_la_plus_proche_et_convertit_le_vent_en_km_h() {
        let body = serde_json::json!({
            "properties": { "timeseries": [
                {
                    "time": "2026-05-12T20:00:00Z",
                    "data": { "instant": { "details": { "air_temperature": 30.0, "wind_speed": 1.0 } } }
                },
                {
                    "time": "2026-05-12T14:00:00Z",
                    "data": {
                        "instant": { "details": { "air_temperature": 18.3, "wind_speed": 5.0 } },
                        "next_1_hours": { "details": { "precipitation_amount": 0.4 } }
                    }
                }
            ]}
        })
        .to_string();

        let readings = decode_met_norway(&body, maintenant());
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].temperature, 18.3);
        assert_eq!(readings[0].precipitation, 0.4);
        // 5 m/s = 18 km/h
        assert!((readings[0].wind_speed - 18.0).abs() < 1e-9);
    }

    #[test]
    fn une_serie_vide_ne_donne_rien_a_comparer() {
        let body = r#"{"properties":{"timeseries":[]}}"#;
        assert!(decode_met_norway(body, maintenant()).is_empty());
    }

    #[test]
    fn en_direct_le_client_se_nomme_par_le_relais_il_sen_abstient() {
        let direct = met_norway_call(&Endpoints::direct(), 48.4468, 1.4892);
        assert_eq!(direct.user_agent, Some(USER_AGENT));
        assert!(direct.url.contains("lat=48.4468"), "{}", direct.url);

        // Un navigateur refuserait de poser cet en-tête, et le relais l'a déjà
        // mis.
        let relais = met_norway_call(&Endpoints::relais("https://relais.klima"), 48.44, 1.48);
        assert_eq!(relais.user_agent, None);
        assert!(relais.url.starts_with("https://relais.klima/v1/met-norway/compact?"));
    }

    /* ---- Bright Sky ---- */

    #[test]
    fn lit_lobservation_de_la_station() {
        let body = r#"{"weather":{"temperature":17.8,"precipitation":0.2,"wind_speed":14}}"#;
        let readings = decode_bright_sky(body);

        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].temperature, 17.8);
        assert_eq!(readings[0].source.name, "Observation DWD");
    }

    #[test]
    fn aucune_station_a_portee_absence_pas_panne() {
        // Ce que laisse un 404 : un corps vide, ou sans météo.
        assert!(decode_bright_sky("{}").is_empty());
    }

    #[test]
    fn station_sans_temperature_rien_a_comparer() {
        assert!(decode_bright_sky(r#"{"weather":{"temperature":null}}"#).is_empty());
    }

    /* ---- l'ensemble ---- */

    #[test]
    fn les_mentions_ne_sont_listees_quune_fois_par_licence() {
        let mut readings = decode_open_meteo(
            &payload_open_meteo(&[
                ("meteofrance_seamless", [Some(10.0), Some(18.0), Some(19.0)]),
                ("ecmwf_ifs025", [Some(10.0), Some(18.5), Some(19.0)]),
            ]),
            maintenant(),
        );
        readings.extend(decode_bright_sky(
            r#"{"weather":{"temperature":17.8,"precipitation":0,"wind_speed":14}}"#,
        ));

        let mentions = attributions_for(&readings);
        assert_eq!(mentions.len(), 2);
        assert!(mentions.iter().any(|m| m.contains("Open-Meteo")));
        assert!(mentions.iter().any(|m| m.contains("Bright Sky")));
    }

    #[test]
    fn les_trois_adresses_passent_par_le_relais_quand_il_y_en_a_un() {
        let relais = Endpoints::relais("https://relais.klima");
        for appel in [
            open_meteo_call(&relais, 48.44, 1.48),
            met_norway_call(&relais, 48.44, 1.48),
            bright_sky_call(&relais, 48.44, 1.48),
        ] {
            assert!(appel.url.starts_with("https://relais.klima/v1/"), "{}", appel.url);
        }
    }

    #[test]
    fn la_comparaison_ne_demande_que_trois_variables_et_un_seul_jour() {
        let appel = open_meteo_call(&Endpoints::direct(), 48.44, 1.48);
        assert!(appel.url.contains("hourly=temperature_2m,precipitation,wind_speed_10m"));
        assert!(appel.url.contains("forecast_days=1"));
        assert!(appel.url.contains(
            "models=meteofrance_seamless,ecmwf_ifs025,icon_seamless,gfs_seamless,ukmo_seamless,gem_seamless,jma_seamless"
        ));
    }
}
