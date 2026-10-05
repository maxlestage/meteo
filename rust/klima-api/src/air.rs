//! La qualité de l'air et les pollens : l'adresse, et la lecture.
//!
//! Le service est celui de qualité de l'air d'Open-Meteo, qui redistribue les
//! prévisions européennes de Copernicus (CAMS) sous licence CC BY 4.0. On ne
//! demande que l'instant présent (`current`) : la tuile dit l'air de
//! maintenant, pas celui de demain.
//!
//! Comme ailleurs, construire l'adresse et lire la réponse sont séparés de
//! l'appel, pour que le décodage se teste sur une réponse enregistrée.

use klima_core::air::{AirSample, POLLENS};
use klima_core::endpoints::Endpoints;
use serde_json::Value;

/// Ce qu'on demande : l'indice européen, les polluants qu'il agrège et qu'on
/// affiche, puis les six pollens.
pub fn air_variables() -> String {
    let mut variables = vec!["european_aqi", "pm2_5", "pm10", "nitrogen_dioxide", "ozone"];
    variables.extend(POLLENS.iter().map(|p| p.variable()));
    variables.join(",")
}

/// L'adresse de l'air d'un point.
pub fn air_url(endpoints: &Endpoints, latitude: f64, longitude: f64) -> String {
    format!(
        "{}?latitude={latitude:.4}&longitude={longitude:.4}&current={}&timezone=auto",
        endpoints.open_meteo_air,
        air_variables().replace(',', "%2C")
    )
}

/// Lit la réponse. `None` si elle n'a pas de bloc `current` : une réponse
/// sans mesure n'est pas un air pur.
pub fn decode_air(body: &str) -> Option<AirSample> {
    let payload: Value = serde_json::from_str(body).ok()?;
    let current = payload.get("current")?.as_object()?;
    let nombre = |nom: &str| current.get(nom).and_then(Value::as_f64).filter(|v| v.is_finite());

    Some(AirSample {
        european_aqi: nombre("european_aqi"),
        pm2_5: nombre("pm2_5"),
        pm10: nombre("pm10"),
        nitrogen_dioxide: nombre("nitrogen_dioxide"),
        ozone: nombre("ozone"),
        // Un pollen absent (hors d'Europe, `null`) n'est pas un pollen à zéro :
        // on l'écarte, comme une source sans température.
        pollens: POLLENS.iter().filter_map(|p| Some((*p, nombre(p.variable())?))).collect(),
    })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use klima_core::air::Pollen;

    const REPONSE: &str = r#"{
      "latitude": 48.86, "longitude": 2.34, "timezone": "Europe/Paris",
      "current": {
        "time": "2026-05-12T09:00", "interval": 3600,
        "european_aqi": 38, "pm2_5": 9.4, "pm10": 17.2,
        "nitrogen_dioxide": 21.0, "ozone": 64.0,
        "alder_pollen": 0.0, "birch_pollen": 12.5, "grass_pollen": 41.0,
        "mugwort_pollen": null, "olive_pollen": 0.2, "ragweed_pollen": 0.0
      }
    }"#;

    #[test]
    fn ladresse_demande_lindice_les_polluants_et_les_six_pollens() {
        let url = air_url(&Endpoints::direct(), 48.8566, 2.3522);
        assert_eq!(
            url,
            concat!(
                "https://air-quality-api.open-meteo.com/v1/air-quality",
                "?latitude=48.8566&longitude=2.3522",
                "&current=european_aqi%2Cpm2_5%2Cpm10%2Cnitrogen_dioxide%2Cozone",
                "%2Calder_pollen%2Cbirch_pollen%2Cgrass_pollen%2Cmugwort_pollen",
                "%2Colive_pollen%2Cragweed_pollen&timezone=auto"
            )
        );
    }

    #[test]
    fn par_le_relais_seul_lhote_change() {
        let url = air_url(&Endpoints::relais("https://relais.klima"), 48.8566, 2.3522);
        assert!(url.starts_with("https://relais.klima/v1/open-meteo/air-quality?latitude=48.8566"));
    }

    #[test]
    fn lit_lindice_les_polluants_et_les_pollens_presents() {
        let air = decode_air(REPONSE).unwrap();
        assert_eq!(air.european_aqi, Some(38.0));
        assert_eq!(air.pm2_5, Some(9.4));
        assert_eq!(air.ozone, Some(64.0));
        assert_eq!(air.pollens.len(), 5, "l'armoise à null est écartée");
        assert!(air.pollens.contains(&(Pollen::Graminees, 41.0)));
    }

    #[test]
    fn hors_deurope_pas_de_pollens_mais_un_indice() {
        let air = decode_air(r#"{"current":{"european_aqi":12,"pm2_5":3.1}}"#).unwrap();
        assert_eq!(air.european_aqi, Some(12.0));
        assert!(air.pollens.is_empty());
    }

    #[test]
    fn une_reponse_sans_mesure_nest_pas_un_air_pur() {
        assert_eq!(decode_air(r#"{"latitude":1}"#), None);
        assert_eq!(decode_air("pas du json"), None);
    }
}
