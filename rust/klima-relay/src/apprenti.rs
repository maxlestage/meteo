//! L'apprenti, côté relais : il va chercher de quoi noter les modèles.
//!
//! Deux interrogations, l'une après l'autre : une semaine de bulletins autour
//! de la ville — pour choisir l'aéroport qui jugera et lire ce qu'il a relevé —
//! puis, au point de cet aéroport, ce que chaque modèle avait annoncé la
//! veille pour chacune de ces heures. La note elle-même est pure
//! (`klima_core::apprenti`).
//!
//! Rien n'est gardé ailleurs qu'en mémoire, et rien n'a besoin de l'être :
//! Open-Meteo archive les prévisions passées, l'Aviation Weather Center garde
//! une semaine de bulletins. Un redémarrage du dyno ne fait rien oublier à
//! l'apprenti — il relit, c'est tout. La réponse se garde six heures : les
//! modèles tournent quatre fois par jour, et une semaine ne change pas d'une
//! heure à l'autre.

use klima_api::apprenti::{Apprentissage, JOURS, decode_annonces, encoder, station_juge};
use klima_core::apprenti::noter;

use crate::upstream::{self, Fetch, UpstreamError};

/// Six heures de fraîcheur, un jour de dépannage.
pub const TTL_MS: i64 = 6 * 3_600_000;
pub const STALE_MS: i64 = 24 * 3_600_000;

/// Ce que l'apprenti répond quand aucun aéroport ne peut juger.
pub const SANS_JUGE: &str = r#"{"station":null,"notes":[]}"#;

/// Une station n'est juge qu'avec deux jours de relevés au moins.
pub const RELEVES_MIN: usize = 48;

/// Note les modèles pour une maille. Sans aéroport assez proche ou assez
/// bavard, l'apprenti n'a rien à dire : une réponse sans juge ni notes — gardée
/// comme les autres, pour ne pas reposer la question à chaque visite — et les
/// interfaces votent sans correction.
pub async fn apprendre(
    fetch: Fetch,
    cle: Option<String>,
    latitude: f64,
    longitude: f64,
    maintenant_s: i64,
) -> Result<String, UpstreamError> {
    let bulletins = fetch(upstream::aviation_historique(latitude, longitude)).await?;
    let Some((station, nom, lat, lon, releves)) = station_juge(&bulletins, latitude, longitude, RELEVES_MIN)
    else {
        return Ok(SANS_JUGE.to_owned());
    };
    let veilles = fetch(upstream::open_meteo_veilles(cle.as_deref(), lat, lon)).await?;
    let notes = decode_annonces(&veilles)
        .iter()
        .map(|(id, annonces)| noter(id, annonces, &releves, maintenant_s))
        .filter(|note| note.n > 0 || note.pluie_n > 0)
        .collect();
    Ok(encoder(&Apprentissage { station, nom, jours: JOURS, notes }))
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upstream::Call;
    use std::sync::{Arc, Mutex};

    const H: i64 = 3600;
    const DEBUT: i64 = 1_793_000_000 / H * H;

    /// Orly relève 10 °C toutes les heures pendant trois jours, et GFS avait
    /// annoncé 12 °C : deux degrés de trop.
    fn fournisseur(vues: Arc<Mutex<Vec<String>>>) -> Fetch {
        Arc::new(move |call: Call| {
            vues.lock().unwrap().push(call.url.clone());
            let corps = if call.url.contains("aviationweather") {
                let bulletins: Vec<String> = (0..72)
                    .map(|i| {
                        format!(
                            r#"{{"icaoId":"LFPO","obsTime":{},"temp":10,"lat":48.717,"lon":2.384,"name":"Paris/Orly Fld, ID, FR"}}"#,
                            DEBUT + i * H
                        )
                    })
                    .collect();
                format!("[{}]", bulletins.join(","))
            } else {
                let temps: Vec<String> = (0..72).map(|i| (DEBUT + i * H).to_string()).collect();
                let douze = vec!["12.0"; 72].join(",");
                let secs = vec!["0.0"; 72].join(",");
                format!(
                    r#"{{"hourly":{{"time":[{}],"temperature_2m_previous_day1_gfs_seamless":[{douze}],"precipitation_previous_day1_gfs_seamless":[{secs}]}}}}"#,
                    temps.join(",")
                )
            };
            Box::pin(async move { Ok(corps) })
        })
    }

    #[tokio::test]
    async fn l_apprenti_note_au_point_de_l_aeroport() {
        let vues = Arc::new(Mutex::new(Vec::new()));
        let corps = apprendre(fournisseur(vues.clone()), None, 48.80, 2.40, DEBUT + 100 * H).await.unwrap();
        let appris = klima_api::apprenti::decoder(&corps).expect("relu");
        assert_eq!(appris.station, "LFPO");
        assert_eq!(appris.notes.len(), 1);
        assert_eq!(appris.notes[0].source_id, "gfs_seamless");
        assert_eq!((appris.notes[0].n, appris.notes[0].biais), (72, 2.0));
        assert_eq!(appris.notes[0].correction(), 1.5);

        let vues = vues.lock().unwrap();
        assert!(vues[0].contains("hours=168"), "{}", vues[0]);
        assert!(vues[1].starts_with("https://previous-runs-api.open-meteo.com/v1/forecast"), "{}", vues[1]);
        assert!(vues[1].contains("latitude=48.717") && vues[1].contains("longitude=2.384"), "au point de l'aéroport : {}", vues[1]);
        assert!(vues[1].contains("timeformat=unixtime"));
    }

    #[tokio::test]
    async fn sans_aeroport_l_apprenti_se_tait_sans_interroger_open_meteo() {
        let vues = Arc::new(Mutex::new(Vec::new()));
        let corps = apprendre(fournisseur(vues.clone()), None, 43.60, 1.44, DEBUT + 100 * H).await.unwrap();
        assert_eq!(corps, SANS_JUGE);
        assert!(klima_api::apprenti::decoder(&corps).is_none());
        assert_eq!(vues.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn la_cle_passe_par_l_hote_payant() {
        let vues = Arc::new(Mutex::new(Vec::new()));
        let _ = apprendre(fournisseur(vues.clone()), Some("secret".into()), 48.80, 2.40, DEBUT + 100 * H).await;
        let vues = vues.lock().unwrap();
        assert!(vues[1].starts_with("https://customer-previous-runs-api.open-meteo.com/"));
        assert!(vues[1].contains("apikey=secret"));
    }
}
