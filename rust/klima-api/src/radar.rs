//! Le radar, sur le fil : ce que le relais répond sur `/v1/radar`.
//!
//! Le relais lit la mosaïque européenne et en tire, pour une ville, une
//! `klima_core::radar::Prevision` ; ce module l'écrit et la relit, dans une
//! forme courte que l'iPhone lit aussi (`AgroWeatherService.decodeRadar`) :
//!
//! ```json
//! {"image": 1791327600, "maintenant": 0.8,
//!  "quarts": [[1791327600, 0.8], [1791328500, 1.4]],
//!  "vitesse": 42.0, "cap": 75.0}
//! ```
//!
//! Les heures sont en secondes UTC ; les débits en mm/h. Une ville que nul
//! radar ne voit a `maintenant` à `null` et aucun quart.

use klima_core::endpoints::Endpoints;
use klima_core::radar::Prevision;
use serde_json::{Value, json};

/// L'adresse du radar pour un point, quand il y a un relais.
pub fn radar_url(endpoints: &Endpoints, latitude: f64, longitude: f64) -> Option<String> {
    Some(format!("{}?lat={latitude:.4}&lon={longitude:.4}", endpoints.radar_url()?))
}

fn arrondi(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Écrit la prévision du radar.
pub fn encoder(p: &Prevision) -> String {
    let quarts: Vec<Value> = p.quarts.iter().map(|(t, d)| json!([t / 1000, arrondi(*d)])).collect();
    let mut objet = json!({
        "image": p.image / 1000,
        "maintenant": p.maintenant.map(arrondi),
        "quarts": quarts,
    });
    if let Some((vitesse, cap)) = p.deplacement {
        objet["vitesse"] = json!(arrondi(vitesse));
        objet["cap"] = json!(arrondi(cap));
    }
    objet.to_string()
}

/// Relit la prévision du radar ; `None` si la réponse n'en est pas une.
pub fn decoder(corps: &str) -> Option<Prevision> {
    let v: Value = serde_json::from_str(corps).ok()?;
    let image = v["image"].as_i64()? * 1000;
    let quarts = v["quarts"]
        .as_array()?
        .iter()
        .filter_map(|q| Some((q[0].as_i64()? * 1000, q[1].as_f64()?)))
        .collect();
    let deplacement = v["vitesse"].as_f64().zip(v["cap"].as_f64());
    Some(Prevision { image, maintenant: v["maintenant"].as_f64(), quarts, deplacement })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_prevision_du_radar_fait_l_aller_retour() {
        let p = Prevision {
            image: 1_791_327_600_000,
            maintenant: Some(0.8),
            quarts: vec![(1_791_327_600_000, 0.8), (1_791_328_500_000, 1.4)],
            deplacement: Some((42.0, 75.0)),
        };
        let fil = encoder(&p);
        assert_eq!(
            fil,
            r#"{"cap":75.0,"image":1791327600,"maintenant":0.8,"quarts":[[1791327600,0.8],[1791328500,1.4]],"vitesse":42.0}"#
        );
        assert_eq!(decoder(&fil), Some(p));

        let aveugle = Prevision { image: 1_791_327_600_000, maintenant: None, quarts: Vec::new(), deplacement: None };
        assert_eq!(decoder(&encoder(&aveugle)), Some(aveugle));
        assert_eq!(decoder("pas du json"), None);
    }

    #[test]
    fn le_radar_ne_se_demande_qu_au_relais() {
        assert_eq!(
            radar_url(&Endpoints::relais("https://relais.klima"), 48.39, -4.4861).as_deref(),
            Some("https://relais.klima/v1/radar?lat=48.3900&lon=-4.4861")
        );
        assert_eq!(radar_url(&Endpoints::direct(), 48.39, -4.4861), None);
    }
}
