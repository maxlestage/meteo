//! Client de l'API de prévision d'Open-Meteo : les adresses, et la lecture
//! des réponses.
//!
//! On n'interroge que ce qu'une ville regarde avant de sortir : température
//! et ressenti, pluie et son risque, vent et rafales, indice UV, pression.
//!
//! ## Les horodatages sont ceux de la ville
//!
//! Open-Meteo renvoie des heures locales sans décalage (« 2026-05-12T21:00 »).
//! On garde l'heure de la ville telle quelle plutôt que d'en faire un instant
//! absolu : c'est la convention du cœur, celle qui fait que « 21 h » veut dire
//! 21 h là-bas dans `alerts` comme dans `ville`, quel que soit le fuseau du
//! serveur.
//!
//! Le décalage n'est pas perdu pour autant : `Forecast` le porte, et
//! `instant()` rend l'instant absolu pour qui en a besoin — un minuteur
//! d'écran verrouillé, par exemple, qui compte dans le temps du téléphone.

use klima_core::ciel::CielObserve;
use klima_core::fusion::{Observation, SerieSource, recouper};
use klima_core::meteo::{CurrentSample, DailySample, HourlySample};
use klima_core::calendar::civil;
use klima_core::endpoints::Endpoints;
use klima_core::i18n::{Params, params};
use serde_json::Value;

const CURRENT_VARIABLES: &str = "temperature_2m,apparent_temperature,relative_humidity_2m,\
weather_code,is_day,wind_speed_10m,wind_gusts_10m,pressure_msl";

const HOURLY_VARIABLES: &str = "temperature_2m,apparent_temperature,weather_code,is_day,\
precipitation_probability,relative_humidity_2m,dew_point_2m,precipitation,wind_speed_10m,\
wind_gusts_10m,uv_index";

const DAILY_VARIABLES: &str = "weather_code,sunrise,sunset,temperature_2m_min,temperature_2m_max,\
precipitation_sum,precipitation_probability_max,wind_gusts_10m_max,uv_index_max";

const HOUR_MS: i64 = 3_600_000;

pub use klima_core::position::Parcelle;

#[derive(Debug, Clone, PartialEq)]
pub struct Forecast {
    pub parcelle: Parcelle,
    /// Fuseau retenu par l'API pour cette parcelle.
    pub timezone: String,
    /// Décalage de ce fuseau, en secondes.
    pub utc_offset_seconds: i64,
    /// Altitude du point de grille (m).
    pub elevation: f64,
    pub current: CurrentSample,
    pub hourly: Vec<HourlySample>,
    pub daily: Vec<DailySample>,
    pub fetched_at: i64,
    /// Les sources qui ont fait cette prévision, quand elle est recoupée
    /// (`recoupee`) ; vide pour la prévision de base, d'un seul modèle.
    pub sources: Vec<String>,
    /// Le ciel observé à l'aéroport le plus proche, quand il y en a un assez
    /// près et assez récent (`klima_core::ciel`).
    pub ciel: Option<CielObserve>,
}

impl Forecast {
    /// L'instant absolu d'un horodatage de parcelle.
    ///
    /// Les séries sont en heure locale du champ ; cette fonction rend l'instant
    /// que connaît l'horloge du téléphone, pour un minuteur ou une
    /// notification programmée.
    pub fn instant(&self, local_ms: i64) -> i64 {
        local_ms - self.utc_offset_seconds * 1000
    }

    /// La même prévision, refaite avec toutes les sources reçues
    /// (`klima_core::fusion`). `observation` : la température d'une station
    /// proche, qui ne vote que pour l'instant présent ; `ciel` : ce que voit
    /// l'aéroport le plus proche — ce qu'il voit tomber s'impose à l'instant
    /// et à l'heure en cours. Sans rien de tout cela, la prévision revient
    /// telle quelle.
    ///
    /// `radar` : le débit que le radar voit au-dessus de la ville (mm/h),
    /// `None` s'il ne la voit pas. Quand il la voit, c'est lui qui dit ce qui
    /// tombe — il regarde la ville même, l'aéroport regarde à quelques
    /// kilomètres ; sinon, l'aéroport.
    pub fn recoupee(
        self,
        series: &[SerieSource],
        observation: Option<f64>,
        ciel: Option<CielObserve>,
        radar: Option<f64>,
    ) -> Forecast {
        let tombe = match radar {
            Some(_) => klima_core::radar::tombe(radar),
            None => ciel.as_ref().and_then(|c| c.tombe).map(|t| t.code),
        };
        if series.is_empty() && observation.is_none() && tombe.is_none() {
            return Forecast { ciel, ..self };
        }
        let vue = Observation { temperature: observation, tombe };
        let r = recouper(&self.hourly, &self.daily, &self.current, series, vue);
        let mut sources = r.sources;
        if observation.is_some() {
            sources.push(klima_core::providers::BRIGHT_SKY_SOURCE.id.to_owned());
        }
        Forecast { hourly: r.heures, daily: r.jours, current: r.courant, sources, ciel, ..self }
    }
}

/// Panne côté service.
///
/// L'erreur porte une clé de catalogue, pas une phrase : c'est l'interface qui
/// la formule dans la langue de l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// Le service n'a pas répondu — réseau coupé, DNS, délai dépassé.
    Unreachable,
    /// Le service a répondu, mais par un code d'erreur.
    Status(u16),
    /// Le service a répondu, mais pas ce qu'on attendait.
    Malformed,
}

impl ApiError {
    pub fn message_key(&self) -> &'static str {
        match self {
            ApiError::Unreachable => "api.unreachable",
            ApiError::Status(_) => "api.status",
            ApiError::Malformed => "api.malformed",
        }
    }

    pub fn params(&self) -> Params {
        match self {
            ApiError::Status(status) => params([("status", i32::from(*status).into())]),
            _ => Params::new(),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message_key())
    }
}

impl std::error::Error for ApiError {}

/* -------------------- les adresses -------------------- */

/// Encode une valeur de paramètre comme le fait `URLSearchParams`.
///
/// Les virgules des listes de variables deviennent `%2C` : le TypeScript les
/// envoie ainsi, et deux clients qui n'écrivent pas la même adresse se
/// retrouvent avec deux entrées de cache pour une seule prévision.
fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

pub(crate) fn query(pairs: &[(&str, String)]) -> String {
    pairs
        .iter()
        .map(|(name, value)| format!("{}={}", encode(name), encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

/// L'adresse de la prévision d'une ville sur `days` jours.
pub fn forecast_url(endpoints: &Endpoints, parcelle: &Parcelle, days: u32) -> String {
    let pairs = [
        ("latitude", format!("{:.4}", parcelle.latitude)),
        ("longitude", format!("{:.4}", parcelle.longitude)),
        ("current", CURRENT_VARIABLES.to_owned()),
        ("hourly", HOURLY_VARIABLES.to_owned()),
        ("daily", DAILY_VARIABLES.to_owned()),
        ("wind_speed_unit", "kmh".to_owned()),
        ("timezone", "auto".to_owned()),
        ("forecast_days", days.to_string()),
    ];
    format!("{}?{}", endpoints.open_meteo_forecast, query(&pairs))
}

/// L'adresse d'une recherche de commune, ou `None` si la requête est trop
/// courte pour valoir un appel.
pub fn search_url(endpoints: &Endpoints, requete: &str) -> Option<String> {
    let trimmed = requete.trim();
    if trimmed.chars().count() < 2 {
        return None;
    }

    let pairs = [
        ("name", trimmed.to_owned()),
        ("count", "8".to_owned()),
        ("language", "fr".to_owned()),
        ("format", "json".to_owned()),
    ];
    Some(format!("{}?{}", endpoints.open_meteo_search, query(&pairs)))
}

/* -------------------- décodage des réponses -------------------- */

/// Lit une réponse de prévision.
pub fn decode_forecast(
    parcelle: Parcelle,
    body: &str,
    fetched_at: i64,
) -> Result<Forecast, ApiError> {
    let payload: Value = serde_json::from_str(body).map_err(|_| ApiError::Malformed)?;

    let timezone = payload["timezone"].as_str().unwrap_or("UTC").to_owned();
    let utc_offset_seconds = payload["utc_offset_seconds"].as_i64().unwrap_or(0);
    let elevation = payload["elevation"].as_f64().unwrap_or(0.0);

    let current = decode_current(&payload["current"], fetched_at)?;
    let hourly = decode_hourly(&payload["hourly"]);
    let daily = decode_daily(&payload["daily"]);

    Ok(Forecast {
        parcelle,
        timezone,
        utc_offset_seconds,
        elevation,
        // L'API renvoie la journée entière depuis minuit : on repart de l'heure
        // en cours, pour que « maintenant » soit le premier élément des séries.
        hourly: from_current_hour(hourly, current.time),
        current,
        daily,
        fetched_at,
        sources: Vec::new(),
        ciel: None,
    })
}

/// Lit une réponse de géocodage.
pub fn decode_search(body: &str) -> Result<Vec<Parcelle>, ApiError> {
    let payload: Value = serde_json::from_str(body).map_err(|_| ApiError::Malformed)?;

    let Some(results) = payload["results"].as_array() else {
        // Aucun résultat : l'API omet la clé plutôt que d'envoyer un tableau
        // vide. Ce n'est pas une panne.
        return Ok(Vec::new());
    };

    Ok(results
        .iter()
        .filter_map(|result| {
            Some(Parcelle {
                name: result["name"].as_str()?.to_owned(),
                latitude: result["latitude"].as_f64()?,
                longitude: result["longitude"].as_f64()?,
                admin: result["admin1"].as_str().map(str::to_owned),
                country: result["country"].as_str().map(str::to_owned),
            })
        })
        .collect())
}

fn from_current_hour(hours: Vec<HourlySample>, now: i64) -> Vec<HourlySample> {
    let start = now.div_euclid(HOUR_MS) * HOUR_MS;
    let trimmed: Vec<HourlySample> =
        hours.iter().filter(|hour| hour.time >= start).cloned().collect();
    // Si l'heure courante sort de la série, on garde la série telle quelle.
    if trimmed.is_empty() { hours } else { trimmed }
}

/// Lit une valeur scalaire, 0 si absente ou illisible.
fn scalar(block: &Value, key: &str) -> f64 {
    block[key].as_f64().filter(|value| value.is_finite()).unwrap_or(0.0)
}

/// Une colonne de nombres, ramenée à `length` valeurs.
///
/// Open-Meteo renvoie des tableaux parallèles indexés par `time`, avec des
/// `null` quand une variable manque sur le point de grille : on les ramène à 0
/// pour garder des séries de longueur homogène.
pub(crate) fn column(block: &Value, key: &str, length: usize) -> Vec<f64> {
    let raw = block[key].as_array();
    (0..length)
        .map(|index| {
            raw.and_then(|values| values.get(index))
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite())
                .unwrap_or(0.0)
        })
        .collect()
}

/// Une colonne de dates : les valeurs illisibles deviennent `None`.
fn date_column(block: &Value, key: &str, length: usize) -> Vec<Option<i64>> {
    let raw = block[key].as_array();
    (0..length)
        .map(|index| {
            raw.and_then(|values| values.get(index))
                .and_then(Value::as_str)
                .and_then(parse_stamp)
        })
        .collect()
}

/// Lit un horodatage d'Open-Meteo, en heure de la parcelle.
///
/// Deux formes : « 2026-05-12 » pour une journée, « 2026-05-12T21:00 » pour une
/// heure. Les secondes, quand il y en a, ne nous intéressent pas.
pub fn parse_stamp(stamp: &str) -> Option<i64> {
    let bytes = stamp.as_bytes();
    if bytes.len() < 10 {
        return None;
    }

    let number = |from: usize, to: usize| stamp.get(from..to)?.parse::<u32>().ok();

    let year = stamp.get(0..4)?.parse::<i32>().ok()?;
    let month = number(5, 7)?;
    let day = number(8, 10)?;
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let (hour, minute) = if bytes.len() >= 16 {
        if bytes[10] != b'T' && bytes[10] != b' ' || bytes[13] != b':' {
            return None;
        }
        (number(11, 13)?, number(14, 16)?)
    } else {
        (0, 0)
    };
    if hour > 23 || minute > 59 {
        return None;
    }

    Some(civil(year, month, day, hour, minute))
}

fn decode_current(block: &Value, fallback: i64) -> Result<CurrentSample, ApiError> {
    if !block.is_object() {
        return Err(ApiError::Malformed);
    }

    Ok(CurrentSample {
        time: block["time"].as_str().and_then(parse_stamp).unwrap_or(fallback),
        temperature: scalar(block, "temperature_2m"),
        apparent_temperature: scalar(block, "apparent_temperature"),
        weather_code: scalar(block, "weather_code") as u16,
        is_day: scalar(block, "is_day") == 1.0,
        relative_humidity: scalar(block, "relative_humidity_2m"),
        wind_speed: scalar(block, "wind_speed_10m"),
        wind_gusts: scalar(block, "wind_gusts_10m"),
        pressure: scalar(block, "pressure_msl"),
    })
}

fn decode_hourly(block: &Value) -> Vec<HourlySample> {
    let Some(times) = block["time"].as_array() else {
        return Vec::new();
    };
    let length = times.len();
    let c = |key: &str| column(block, key, length);

    let temperature = c("temperature_2m");
    let apparent = c("apparent_temperature");
    let weather_code = c("weather_code");
    let is_day = c("is_day");
    let rain_probability = c("precipitation_probability");
    let humidity = c("relative_humidity_2m");
    let dew_point = c("dew_point_2m");
    let precipitation = c("precipitation");
    let wind_speed = c("wind_speed_10m");
    let wind_gusts = c("wind_gusts_10m");
    let uv = c("uv_index");

    times
        .iter()
        .enumerate()
        .filter_map(|(i, stamp)| {
            let time = stamp.as_str().and_then(parse_stamp)?;
            Some(HourlySample {
                time,
                weather_code: weather_code[i] as u16,
                is_day: is_day[i] != 0.0,
                precipitation_probability: rain_probability[i],
                temperature: temperature[i],
                apparent_temperature: apparent[i],
                relative_humidity: humidity[i],
                dew_point: dew_point[i],
                precipitation: precipitation[i],
                wind_speed: wind_speed[i],
                wind_gusts: wind_gusts[i],
                uv_index: uv[i],
            })
        })
        .collect()
}

fn decode_daily(block: &Value) -> Vec<DailySample> {
    let Some(times) = block["time"].as_array() else {
        return Vec::new();
    };
    let length = times.len();
    let c = |key: &str| column(block, key, length);

    let weather_code = c("weather_code");
    let sunrise = date_column(block, "sunrise", length);
    let sunset = date_column(block, "sunset", length);
    let t_min = c("temperature_2m_min");
    let t_max = c("temperature_2m_max");
    let rain = c("precipitation_sum");
    let rain_probability = c("precipitation_probability_max");
    let gusts = c("wind_gusts_10m_max");
    let uv = c("uv_index_max");

    times
        .iter()
        .enumerate()
        .filter_map(|(i, stamp)| {
            let date = stamp.as_str().and_then(parse_stamp)?;
            Some(DailySample {
                date,
                weather_code: weather_code[i] as u16,
                temperature_min: t_min[i],
                temperature_max: t_max[i],
                precipitation_sum: rain[i],
                precipitation_probability_max: rain_probability[i],
                wind_gusts_max: gusts[i],
                uv_index_max: uv[i],
                sunrise: sunrise[i],
                sunset: sunset[i],
            })
        })
        .collect()
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use klima_core::calendar::civil;

    /// Une réponse d'Open-Meteo réduite à trois heures et deux jours, avec les
    /// pièges qu'on rencontre pour de vrai : un `null` au milieu d'une colonne,
    /// une variable absente du bloc, et un horodatage illisible.
    const PAYLOAD: &str = r#"{
      "timezone": "Europe/Paris",
      "utc_offset_seconds": 7200,
      "elevation": 12.0,
      "current": {
        "time": "2026-05-12T09:00",
        "temperature_2m": 17.4,
        "apparent_temperature": 16.1,
        "relative_humidity_2m": 62,
        "weather_code": 3,
        "is_day": 1,
        "wind_speed_10m": 11.2,
        "wind_gusts_10m": 18.5,
        "pressure_msl": 1016.2
      },
      "hourly": {
        "time": ["2026-05-12T08:00", "2026-05-12T09:00", "2026-05-12T10:00", "pas-une-date"],
        "temperature_2m": [15.0, 17.4, 19.1, 20.0],
        "apparent_temperature": [14.0, 16.1, 18.0, 19.2],
        "weather_code": [3, 3, 61, 61],
        "is_day": [1, 1, 1, 1],
        "precipitation_probability": [10, 20, null, 40],
        "relative_humidity_2m": [70, 62, 58, 55],
        "dew_point_2m": [9.0, 9.5, 10.0, 10.2],
        "precipitation": [0.0, 0.0, 1.4, 0.2],
        "wind_speed_10m": [9.0, 11.2, 13.0, 14.0],
        "wind_gusts_10m": [15.0, 18.5, 22.0, 24.0]
      },
      "daily": {
        "time": ["2026-05-12", "2026-05-13"],
        "weather_code": [61, 3],
        "sunrise": ["2026-05-12T06:32", "pas-une-date"],
        "sunset": ["2026-05-12T21:14", "2026-05-13T21:15"],
        "temperature_2m_min": [11.2, 10.4],
        "temperature_2m_max": [21.0, 22.3],
        "precipitation_sum": [3.4, 0.0],
        "precipitation_probability_max": [80, 10],
        "wind_gusts_10m_max": [42.0, 31.0],
        "uv_index_max": [5.2, 6.1]
      }
    }"#;

    fn parcelle() -> Parcelle {
        Parcelle {
            name: "Le Clos".to_owned(),
            latitude: 43.4832,
            longitude: -1.5586,
            admin: None,
            country: None,
        }
    }

    fn forecast() -> Forecast {
        decode_forecast(parcelle(), PAYLOAD, civil(2026, 5, 12, 9, 7)).unwrap()
    }

    /* ---- les adresses ---- */

    #[test]
    fn ladresse_de_prevision_est_celle_du_typescript_caractere_pour_caractere() {
        // Deux clients qui n'écrivent pas la même adresse se retrouvent avec
        // deux entrées de cache pour une seule prévision.
        assert_eq!(
            forecast_url(&Endpoints::direct(), &parcelle(), 7),
            concat!(
                "https://api.open-meteo.com/v1/forecast",
                "?latitude=43.4832&longitude=-1.5586",
                "&current=temperature_2m%2Capparent_temperature%2Crelative_humidity_2m",
                "%2Cweather_code%2Cis_day%2Cwind_speed_10m%2Cwind_gusts_10m%2Cpressure_msl",
                "&hourly=temperature_2m%2Capparent_temperature%2Cweather_code%2Cis_day",
                "%2Cprecipitation_probability%2Crelative_humidity_2m%2Cdew_point_2m",
                "%2Cprecipitation%2Cwind_speed_10m%2Cwind_gusts_10m%2Cuv_index",
                "&daily=weather_code%2Csunrise%2Csunset%2Ctemperature_2m_min",
                "%2Ctemperature_2m_max%2Cprecipitation_sum",
                "%2Cprecipitation_probability_max%2Cwind_gusts_10m_max%2Cuv_index_max",
                "&wind_speed_unit=kmh&timezone=auto&forecast_days=7",
            )
        );
    }

    #[test]
    fn ladresse_de_recherche_est_celle_du_typescript() {
        assert_eq!(
            search_url(&Endpoints::direct(), "Saint-Jean-de-Luz").as_deref(),
            Some(concat!(
                "https://geocoding-api.open-meteo.com/v1/search",
                "?name=Saint-Jean-de-Luz&count=8&language=fr&format=json"
            ))
        );
    }

    #[test]
    fn une_lettre_ne_vaut_pas_un_appel() {
        assert_eq!(search_url(&Endpoints::direct(), "L"), None);
        assert_eq!(search_url(&Endpoints::direct(), " a "), None);
        assert!(search_url(&Endpoints::direct(), "Oô").is_some());
    }

    #[test]
    fn un_espace_dans_un_nom_de_commune_ne_casse_pas_ladresse() {
        let url = search_url(&Endpoints::direct(), "Saint Jean de Luz").unwrap();
        assert!(url.contains("name=Saint+Jean+de+Luz"), "{url}");
    }

    #[test]
    fn par_le_relais_ladresse_change_dhote_pas_de_parametres() {
        let direct = forecast_url(&Endpoints::direct(), &parcelle(), 7);
        let relais = forecast_url(&Endpoints::relais("https://relais.klima"), &parcelle(), 7);

        assert_eq!(
            relais.split_once('?').unwrap().1,
            direct.split_once('?').unwrap().1
        );
        assert!(relais.starts_with("https://relais.klima/v1/open-meteo/forecast?"));
    }

    /* ---- le décodage ---- */

    #[test]
    fn lit_le_fuseau_laltitude_et_linstant() {
        let f = forecast();

        assert_eq!(f.timezone, "Europe/Paris");
        assert_eq!(f.utc_offset_seconds, 7200);
        assert_eq!(f.elevation, 12.0);
        assert_eq!(f.current.time, civil(2026, 5, 12, 9, 0));
        assert_eq!(f.current.temperature, 17.4);
        assert_eq!(f.current.apparent_temperature, 16.1);
        assert!(f.current.is_day);
        assert_eq!(f.current.weather_code, 3);
    }

    #[test]
    fn les_heures_sont_celles_de_la_parcelle_pas_celles_du_serveur() {
        // 21 h 14 au coucher du soleil veut dire 21 h 14 au champ. Le
        // TypeScript stocke l'instant absolu et rhabille à l'affichage ; ici
        // c'est l'heure locale qui est stockée, et le décalage est à côté.
        let f = forecast();
        let coucher = f.daily[0].sunset.unwrap();

        assert_eq!(coucher, civil(2026, 5, 12, 21, 14));
        // L'instant absolu reste calculable : 21 h 14 en UTC+2, c'est 19 h 14 UTC.
        assert_eq!(f.instant(coucher), civil(2026, 5, 12, 19, 14));
    }

    #[test]
    fn la_serie_horaire_repart_de_lheure_en_cours() {
        // L'API renvoie la journée depuis minuit ; « maintenant » doit être le
        // premier élément, sinon le bandeau horaire s'ouvre sur le passé.
        let f = forecast();

        assert_eq!(f.hourly.len(), 2);
        assert_eq!(f.hourly[0].time, civil(2026, 5, 12, 9, 0));
        assert_eq!(f.hourly[1].time, civil(2026, 5, 12, 10, 0));
    }

    #[test]
    fn un_horodatage_illisible_fait_sauter_la_ligne_pas_la_reponse() {
        let f = forecast();
        // « pas-une-date » était la quatrième heure et le second lever.
        assert!(f.hourly.iter().all(|hour| hour.time > 0));
        assert_eq!(f.daily.len(), 2);
        assert_eq!(f.daily[0].sunrise, Some(civil(2026, 5, 12, 6, 32)));
        assert_eq!(f.daily[1].sunrise, None);
    }

    #[test]
    fn un_null_au_milieu_dune_colonne_vaut_zero() {
        // Une série trouée serait pire : les colonnes sont parallèles, et un
        // décalage d'un cran attribuerait la pluie de 10 h à 9 h.
        let f = forecast();
        assert_eq!(f.hourly[1].precipitation_probability, 0.0);
        assert_eq!(f.hourly[0].precipitation_probability, 20.0);
    }

    #[test]
    fn une_variable_absente_du_bloc_vaut_zero_sur_toute_la_colonne() {
        // L'indice UV horaire manque à la réponse enregistrée.
        let f = forecast();
        assert!(f.hourly.iter().all(|hour| hour.uv_index == 0.0));
        // Et les autres colonnes ne s'en trouvent pas décalées.
        assert_eq!(f.hourly[0].temperature, 17.4);
        assert_eq!(f.hourly[0].apparent_temperature, 16.1);
    }

    #[test]
    fn les_journees_portent_leurs_cumuls() {
        let f = forecast();
        let premier = &f.daily[0];

        assert_eq!(premier.date, civil(2026, 5, 12, 0, 0));
        assert_eq!(premier.precipitation_sum, 3.4);
        assert_eq!(premier.uv_index_max, 5.2);
        assert_eq!(premier.wind_gusts_max, 42.0);
        assert_eq!(premier.temperature_min, 11.2);
        assert_eq!(premier.weather_code, 61);
    }

    #[test]
    fn une_reponse_qui_nest_pas_du_json_est_une_panne_pas_un_plantage() {
        assert_eq!(
            decode_forecast(parcelle(), "<html>502 Bad Gateway</html>", 0),
            Err(ApiError::Malformed)
        );
    }

    #[test]
    fn une_reponse_sans_bloc_courant_est_une_panne() {
        assert_eq!(
            decode_forecast(parcelle(), r#"{"timezone":"UTC"}"#, 0),
            Err(ApiError::Malformed)
        );
    }

    #[test]
    fn lerreur_porte_une_cle_de_catalogue_jamais_une_phrase() {
        assert_eq!(ApiError::Unreachable.message_key(), "api.unreachable");
        assert_eq!(ApiError::Status(503).message_key(), "api.status");
        assert_eq!(
            ApiError::Status(503).params(),
            params([("status", 503.into())])
        );
        for erreur in [ApiError::Unreachable, ApiError::Status(503), ApiError::Malformed]
        {
            assert!(!erreur.message_key().contains(' '));
        }
    }

    /* ---- le géocodage ---- */

    #[test]
    fn lit_les_communes_trouvees() {
        let body = r#"{"results":[
          {"name":"Saint-Jean-de-Luz","latitude":43.3883,"longitude":-1.6594,
           "admin1":"Nouvelle-Aquitaine","country":"France"},
          {"name":"Luz","latitude":42.8742,"longitude":-0.0053,"country":"France"}
        ]}"#;
        let parcelles = decode_search(body).unwrap();

        assert_eq!(parcelles.len(), 2);
        assert_eq!(parcelles[0].name, "Saint-Jean-de-Luz");
        assert_eq!(parcelles[0].admin.as_deref(), Some("Nouvelle-Aquitaine"));
        assert_eq!(parcelles[1].admin, None);
        assert_eq!(parcelles[1].country.as_deref(), Some("France"));
    }

    #[test]
    fn aucun_resultat_nest_pas_une_panne() {
        // L'API omet la clé plutôt que d'envoyer un tableau vide.
        assert_eq!(decode_search(r#"{"generationtime_ms":0.3}"#).unwrap(), Vec::new());
    }

    #[test]
    fn une_commune_sans_coordonnees_est_ecartee_pas_inventee() {
        let body = r#"{"results":[{"name":"Nulle part"},
          {"name":"Luz","latitude":42.8742,"longitude":-0.0053}]}"#;
        let parcelles = decode_search(body).unwrap();

        assert_eq!(parcelles.len(), 1);
        assert_eq!(parcelles[0].name, "Luz");
    }

    /* ---- ce qui tombe maintenant ---- */

    #[test]
    fn le_radar_fait_foi_pour_ce_qui_tombe_quand_il_voit_la_ville() {
        use klima_core::ciel::{CielObserve, Tendance, Tombe};
        use klima_core::veille::Intensite;
        let sec = forecast();
        assert!(sec.current.weather_code < 51, "la base est sèche");
        let aeroport = CielObserve {
            station: "LFBZ".into(),
            nom: "Biarritz".into(),
            distance_km: 4.0,
            time: sec.current.time,
            tombe: Some(Tombe { code: 61, intensite: Intensite::Faible }),
            tendance: Tendance::Inconnue,
        };
        // Sans radar, l'aéroport dit qu'il pleut.
        let f = forecast().recoupee(&[], None, Some(aeroport.clone()), None);
        assert_eq!(f.current.weather_code, 61);
        assert_eq!(f.ciel.as_ref().map(|c| c.station.as_str()), Some("LFBZ"));
        // Le radar voit la ville au sec : il fait foi.
        let f = forecast().recoupee(&[], None, Some(aeroport), Some(0.0));
        assert_eq!(f.current.weather_code, sec.current.weather_code);
        // Le radar voit 5 mm/h sur la ville : pluie modérée.
        let f = forecast().recoupee(&[], None, None, Some(5.0));
        assert_eq!(f.current.weather_code, 63);
    }

    /* ---- les horodatages ---- */

    #[test]
    fn lit_les_deux_formes_dhorodatage() {
        assert_eq!(parse_stamp("2026-05-12"), Some(civil(2026, 5, 12, 0, 0)));
        assert_eq!(parse_stamp("2026-05-12T21:00"), Some(civil(2026, 5, 12, 21, 0)));
        assert_eq!(parse_stamp("2026-05-12T21:00:00"), Some(civil(2026, 5, 12, 21, 0)));
    }

    #[test]
    fn refuse_ce_qui_nest_pas_un_horodatage() {
        for faux in ["", "2026", "2026/05/12", "2026-05-12T25:00", "2026-13-01", "pas-une-date"] {
            assert_eq!(parse_stamp(faux), None, "{faux}");
        }
    }
}
