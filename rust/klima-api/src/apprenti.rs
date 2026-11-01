//! L'apprenti, sur le fil.
//!
//! Deux lectures pour le relais — les bulletins d'une semaine, et ce que
//! chaque modèle avait annoncé la veille (`previous-runs-api` d'Open-Meteo) —
//! et la réponse de `/v1/apprenti`, que l'iPhone lit aussi
//! (`AgroWeatherService.decodeApprenti`) :
//!
//! ```json
//! {"station": "LFPO", "nom": "Paris/Orly", "jours": 7,
//!  "notes": [{"id": "gfs_seamless", "n": 160, "biais": 1.21, "ecart": 1.62,
//!             "pluie_n": 166, "pluie_justes": 150}]}
//! ```
//!
//! La règle — noter, retenir, corriger — vit dans `klima_core::apprenti`.

use klima_core::apprenti::{Annonce, Note, Releve};
use klima_core::ciel::{distance_km, nom_lisible, seuils::RAYON_KM, tombe_du_metar};
use klima_core::endpoints::Endpoints;
use klima_core::providers::OPEN_METEO_SOURCES;
use serde_json::{Value, json};

/// Jours d'histoire qu'on demande.
pub const JOURS: u32 = 7;

/// Les variables archivées qu'on compare : la prévision faite la veille.
pub const VARIABLES: &str = "temperature_2m_previous_day1,precipitation_previous_day1";

/// Ce que l'apprenti a appris pour une ville.
#[derive(Debug, Clone, PartialEq)]
pub struct Apprentissage {
    /// L'aéroport qui a servi de juge.
    pub station: String,
    pub nom: String,
    pub jours: u32,
    pub notes: Vec<Note>,
}

/// L'adresse de l'apprenti, quand il y a un relais.
pub fn apprenti_url(endpoints: &Endpoints, latitude: f64, longitude: f64) -> Option<String> {
    let radar = endpoints.radar_url()?;
    let base = radar.strip_suffix("/v1/radar")?;
    Some(format!("{base}/v1/apprenti?lat={latitude:.4}&lon={longitude:.4}"))
}

/// La station qui jugera : la plus proche du point, dans le rayon du ciel,
/// parmi celles qui ont au moins `min_releves` températures sur la période.
/// Rend son indicatif, son nom, sa position et ses relevés (secondes UTC).
pub fn station_juge(
    corps: &str,
    latitude: f64,
    longitude: f64,
    min_releves: usize,
) -> Option<(String, String, f64, f64, Vec<Releve>)> {
    let Ok(Value::Array(bulletins)) = serde_json::from_str::<Value>(corps) else {
        return None;
    };
    let mut stations: Vec<(String, String, f64, f64, Vec<Releve>)> = Vec::new();
    for b in &bulletins {
        let (Some(id), Some(lat), Some(lon), Some(time)) =
            (b["icaoId"].as_str(), b["lat"].as_f64(), b["lon"].as_f64(), b["obsTime"].as_i64())
        else {
            continue;
        };
        let releve = Releve {
            time,
            temperature: b["temp"].as_f64(),
            mouille: tombe_du_metar(b["wxString"].as_str().unwrap_or_default()).is_some(),
        };
        match stations.iter_mut().find(|s| s.0 == id) {
            Some(s) => s.4.push(releve),
            None => stations.push((
                id.to_owned(),
                nom_lisible(b["name"].as_str().unwrap_or_default()),
                lat,
                lon,
                vec![releve],
            )),
        }
    }
    stations
        .into_iter()
        .filter(|s| s.4.iter().filter(|r| r.temperature.is_some()).count() >= min_releves)
        .map(|s| (distance_km(latitude, longitude, s.2, s.3), s))
        .filter(|(d, _)| *d <= RAYON_KM)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, s)| s)
}

/// Ce que chaque modèle avait annoncé la veille, par source. Les clés de la
/// réponse portent le modèle en suffixe (`temperature_2m_previous_day1_gfs_seamless`).
/// L'heure doit être en secondes UTC (`timeformat=unixtime`).
pub fn decode_annonces(corps: &str) -> Vec<(String, Vec<Annonce>)> {
    let Ok(v) = serde_json::from_str::<Value>(corps) else { return Vec::new() };
    let heures = &v["hourly"];
    let Some(temps) = heures["time"].as_array() else { return Vec::new() };
    let serie = |cle: String| -> Vec<Option<f64>> {
        heures[&cle].as_array().map(|a| a.iter().map(Value::as_f64).collect()).unwrap_or_default()
    };
    OPEN_METEO_SOURCES
        .iter()
        .filter_map(|source| {
            let t = serie(format!("temperature_2m_previous_day1_{}", source.id));
            let p = serie(format!("precipitation_previous_day1_{}", source.id));
            let annonces: Vec<Annonce> = temps
                .iter()
                .enumerate()
                .filter_map(|(i, time)| {
                    let annonce = Annonce {
                        time: time.as_i64()?,
                        temperature: t.get(i).copied().flatten(),
                        precipitation: p.get(i).copied().flatten(),
                    };
                    (annonce.temperature.is_some() || annonce.precipitation.is_some()).then_some(annonce)
                })
                .collect();
            (!annonces.is_empty()).then(|| (source.id.to_owned(), annonces))
        })
        .collect()
}

fn arrondi(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Écrit ce que l'apprenti a appris.
pub fn encoder(a: &Apprentissage) -> String {
    let notes: Vec<Value> = a
        .notes
        .iter()
        .map(|n| {
            json!({
                "id": n.source_id, "n": n.n, "biais": arrondi(n.biais), "ecart": arrondi(n.ecart),
                "pluie_n": n.pluie_n, "pluie_justes": n.pluie_justes,
            })
        })
        .collect();
    json!({"station": a.station, "nom": a.nom, "jours": a.jours, "notes": notes}).to_string()
}

/// Relit ce que l'apprenti a appris ; `None` si la réponse n'en est pas une.
pub fn decoder(corps: &str) -> Option<Apprentissage> {
    let v: Value = serde_json::from_str(corps).ok()?;
    let notes = v["notes"]
        .as_array()?
        .iter()
        .filter_map(|n| {
            Some(Note {
                source_id: n["id"].as_str()?.to_owned(),
                n: u32::try_from(n["n"].as_u64()?).ok()?,
                biais: n["biais"].as_f64()?,
                ecart: n["ecart"].as_f64()?,
                pluie_n: u32::try_from(n["pluie_n"].as_u64().unwrap_or(0)).ok()?,
                pluie_justes: u32::try_from(n["pluie_justes"].as_u64().unwrap_or(0)).ok()?,
            })
        })
        .collect();
    Some(Apprentissage {
        station: v["station"].as_str()?.to_owned(),
        nom: v["nom"].as_str().unwrap_or_default().to_owned(),
        jours: u32::try_from(v["jours"].as_u64().unwrap_or(u64::from(JOURS))).ok()?,
        notes,
    })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    const BULLETINS: &str = r#"[
      {"icaoId":"LFPO","obsTime":1793559600,"temp":10,"wxString":null,"lat":48.717,"lon":2.384,"name":"Paris/Orly Fld, ID, FR"},
      {"icaoId":"LFPO","obsTime":1793556000,"temp":11,"wxString":"-RA","lat":48.717,"lon":2.384,"name":"Paris/Orly Fld, ID, FR"},
      {"icaoId":"LFPG","obsTime":1793559600,"temp":9,"lat":49.01,"lon":2.55,"name":"Paris/Charles De Gaulle Arpt, ID, FR"},
      {"icaoId":"LFOB","obsTime":1793559600,"temp":8,"lat":49.45,"lon":2.11,"name":"Beauvais/Tille Arpt, ID, FR"}
    ]"#;

    #[test]
    fn le_juge_est_la_station_proche_qui_a_assez_de_releves() {
        let (id, nom, _, _, releves) = station_juge(BULLETINS, 48.80, 2.40, 2).expect("une station");
        assert_eq!(id, "LFPO");
        assert_eq!(nom, "Paris/Orly Fld");
        assert_eq!(releves.len(), 2);
        assert!(releves.iter().any(|r| r.mouille));
        // Avec un seuil de deux relevés, Roissy (un seul) n'est pas retenue,
        // même plus proche de ce point-ci.
        let (id, ..) = station_juge(BULLETINS, 48.9, 2.5, 2).expect("une station");
        assert_eq!(id, "LFPO");
        let (id, ..) = station_juge(BULLETINS, 48.9, 2.5, 1).expect("une station");
        assert_eq!(id, "LFPG");
        assert!(station_juge(BULLETINS, 43.6, 1.44, 1).is_none(), "Toulouse : trop loin");
        assert!(station_juge("pas du json", 48.8, 2.4, 1).is_none());
    }

    #[test]
    fn les_annonces_se_lisent_par_modele() {
        let corps = r#"{"hourly":{"time":[1793552400,1793556000],
          "temperature_2m_previous_day1_gfs_seamless":[12.0,null],
          "precipitation_previous_day1_gfs_seamless":[0.0,0.4],
          "temperature_2m_previous_day1_jma_seamless":[null,null],
          "precipitation_previous_day1_jma_seamless":[null,null]}}"#;
        let annonces = decode_annonces(corps);
        assert_eq!(annonces.len(), 1, "un modèle muet n'a pas d'annonces");
        assert_eq!(annonces[0].0, "gfs_seamless");
        assert_eq!(annonces[0].1.len(), 2);
        assert_eq!(annonces[0].1[1].temperature, None);
        assert_eq!(annonces[0].1[1].precipitation, Some(0.4));
        assert!(decode_annonces("{}").is_empty());
    }

    #[test]
    fn l_apprentissage_fait_l_aller_retour() {
        let a = Apprentissage {
            station: "LFPO".into(),
            nom: "Paris/Orly".into(),
            jours: 7,
            notes: vec![Note { source_id: "gfs_seamless".into(), n: 160, biais: 1.214, ecart: 1.6, pluie_n: 166, pluie_justes: 150 }],
        };
        let relu = decoder(&encoder(&a)).expect("relu");
        assert_eq!(relu.station, "LFPO");
        assert_eq!(relu.notes[0].biais, 1.21);
        assert_eq!(relu.notes[0].pluie_justes, 150);
        assert!(decoder("[]").is_none());
    }

    #[test]
    fn l_adresse_n_existe_qu_avec_un_relais() {
        assert_eq!(apprenti_url(&Endpoints::direct(), 48.8, 2.3), None);
        assert_eq!(
            apprenti_url(&Endpoints::relais("https://k.example/"), 48.8, 2.35).as_deref(),
            Some("https://k.example/v1/apprenti?lat=48.8000&lon=2.3500")
        );
    }
}
