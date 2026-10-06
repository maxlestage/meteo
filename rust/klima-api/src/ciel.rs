//! Les bulletins d'aéroport : l'adresse, et la lecture de la réponse.
//!
//! La règle — quel bulletin parle de la ville, ce qu'il voit tomber — vit dans
//! `klima_core::ciel`. Ici, seulement le fil : l'Aviation Weather Center
//! répond un tableau JSON, un bulletin par aéroport du cadre demandé.

use klima_core::ciel::{Metar, cadre, nom_lisible};
use klima_core::endpoints::{Endpoints, Transport};
use klima_core::providers::USER_AGENT;
use serde_json::Value;

use crate::readings::ReadingCall;

/// La demande des bulletins autour d'un point.
///
/// En direct, on demande le cadre à l'Aviation Weather Center en se nommant,
/// comme il le demande ; par le relais, on donne le point, et c'est le serveur
/// qui en fait un cadre — autour de sa maille, pour que sa mise en cache serve
/// à tous les voisins.
pub fn metar_call(endpoints: &Endpoints, latitude: f64, longitude: f64) -> ReadingCall {
    match endpoints.transport {
        Transport::Direct => {
            let (s, o, n, e) = cadre(latitude, longitude);
            ReadingCall {
                url: format!("{}?bbox={s},{o},{n},{e}&format=json", endpoints.aviation),
                user_agent: Some(USER_AGENT),
            }
        }
        Transport::Relais => ReadingCall {
            url: format!("{}?lat={latitude:.4}&lon={longitude:.4}", endpoints.aviation),
            user_agent: None,
        },
    }
}

/// Lit les bulletins. Les heures passent à celle de la ville
/// (`utc_offset_seconds`) ; un bulletin sans position ou sans heure est
/// écarté. Une réponse illisible ne donne aucun bulletin.
pub fn decode_metars(body: &str, utc_offset_seconds: i64) -> Vec<Metar> {
    let Ok(Value::Array(bulletins)) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };
    bulletins
        .iter()
        .filter_map(|b| {
            Some(Metar {
                station: b["icaoId"].as_str()?.to_owned(),
                nom: nom_lisible(b["name"].as_str().unwrap_or_default()),
                latitude: b["lat"].as_f64()?,
                longitude: b["lon"].as_f64()?,
                time: (b["obsTime"].as_i64()? + utc_offset_seconds) * 1000,
                temps_present: b["wxString"].as_str().unwrap_or_default().to_owned(),
            })
        })
        .collect()
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use klima_core::ciel::plus_proche;

    /// Trois bulletins autour de Brest, le 6 octobre 2026 à 14 h UTC.
    const BREST: &str = r#"[
      {"icaoId":"LFRJ","obsTime":1791295200,"wxString":null,"lat":48.527,"lon":-4.138,
       "name":"Landivisiau Arpt, BRE, FR","rawOb":"METAR LFRJ 061400Z AUTO 01008KT 9999 ///CB 18/16 Q1013"},
      {"icaoId":"LFRL","obsTime":1791295200,"wxString":"-RA","lat":48.279,"lon":-4.439,
       "name":"Lanveoc/Poulmic Arpt, BRE, FR","rawOb":"METAR LFRL 061400Z AUTO 35005KT 9000 -RA ///CB 17/16 Q1014"},
      {"icaoId":"LFRB","obsTime":1791295200,"lat":48.444,"lon":-4.412,
       "name":"Brest/Guipavas Intl Arpt, BRE, FR"},
      {"icaoId":"XXXX","obsTime":1791295200}
    ]"#;

    #[test]
    fn les_bulletins_se_lisent_a_l_heure_de_la_ville() {
        let bulletins = decode_metars(BREST, 7200);
        assert_eq!(bulletins.len(), 3, "sans position, écarté");
        let lanveoc = &bulletins[1];
        assert_eq!(lanveoc.station, "LFRL");
        assert_eq!(lanveoc.nom, "Lanveoc/Poulmic");
        assert_eq!(lanveoc.temps_present, "-RA");
        // 14 h UTC, 16 h à Brest.
        assert_eq!(lanveoc.time, (1_791_295_200 + 7200) * 1000);
        assert_eq!(bulletins[2].temps_present, "", "sans temps présent, rien ne tombe");
        assert!(decode_metars("pas du json", 0).is_empty());
        assert!(decode_metars("{}", 0).is_empty());
    }

    #[test]
    fn le_ciel_de_brest_est_celui_de_guipavas() {
        let bulletins = decode_metars(BREST, 7200);
        let maintenant = (1_791_295_200 + 7200 + 600) * 1000;
        let ciel = plus_proche(&bulletins, 48.39, -4.4861, maintenant).unwrap();
        assert_eq!(ciel.station, "LFRB");
        assert_eq!(ciel.tombe, None);
        // Au sud de la rade, c'est Lanvéoc qui parle — et il pleut.
        let ciel = plus_proche(&bulletins, 48.30, -4.45, maintenant).unwrap();
        assert_eq!((ciel.station.as_str(), ciel.tombe), ("LFRL", Some(61)));
    }

    #[test]
    fn en_direct_on_se_nomme_par_le_relais_on_donne_le_point() {
        let direct = metar_call(&Endpoints::direct(), 48.39, -4.4861);
        assert_eq!(
            direct.url,
            "https://aviationweather.gov/api/data/metar?bbox=48.03,-5.03,48.75,-3.94&format=json"
        );
        assert_eq!(direct.user_agent, Some(USER_AGENT));
        let relais = metar_call(&Endpoints::relais("https://relais.klima"), 48.39, -4.4861);
        assert_eq!(relais.url, "https://relais.klima/v1/aviation/metar?lat=48.3900&lon=-4.4861");
        assert_eq!(relais.user_agent, None);
    }
}
