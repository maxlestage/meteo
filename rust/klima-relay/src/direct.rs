//! Le direct : la météo poussée par WebSocket, dès qu'elle change.
//!
//! Le client ouvre `/v1/direct`, puis dit quelle ville il suit :
//!
//! ```json
//! {"latitude": 48.8566, "longitude": 2.3522, "jours": 7}
//! ```
//!
//! Le relais pousse alors six sujets, chacun dans la forme exacte que le
//! fournisseur renvoie — les clients ont déjà de quoi les lire :
//!
//! ```json
//! {"sujet": "base", "latitude": 48.8566, "longitude": 2.3522,
//!  "corps": { …réponse d'Open-Meteo… }}
//! ```
//!
//! Le point de l'abonnement revient avec chaque message : un client qui vient
//! de changer de ville écarte ce qui était déjà en route pour l'ancienne.
//!
//! | sujet      | contenu                                             |
//! | ---------- | --------------------------------------------------- |
//! | `base`     | la prévision et l'instant présent (`current`)       |
//! | `ensemble` | les sept modèles, heure par heure et jour par jour  |
//! | `quarts`   | le quart d'heure du guetteur                        |
//! | `met`      | MET Norway                                          |
//! | `station`  | l'observation de Bright Sky                         |
//! | `air`      | qualité de l'air et pollens                         |
//!
//! Toutes les trente secondes, le relais relit chaque sujet **par les mêmes
//! caches que les requêtes HTTP** (`routes::lire`), et ne pousse que ce qui a
//! changé depuis le dernier envoi à ce client. Mille personnes qui suivent la
//! même ville ne coûtent donc pas plus d'interrogations qu'une seule : c'est
//! la durée de vie des caches — cinq minutes pour le quart d'heure, dix pour
//! l'instant présent, une heure pour les modèles — qui fixe le rythme des
//! fournisseurs, et le direct qui fixe celui des écrans.
//!
//! Un battement part toutes les vingt-cinq secondes : sans lui, un routeur
//! (celui d'Heroku coupe à cinquante-cinq) fermerait une connexion qui n'a
//! rien eu à dire. Changer de ville, c'est renvoyer un abonnement sur la même
//! connexion : tout est poussé de nouveau.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use klima_api::{air, ensemble, open_meteo, readings, veille};
use klima_core::endpoints::Endpoints;
use klima_core::position::Parcelle;
use reqwest::Url;
use serde::Deserialize;
use tokio::task::JoinSet;

use crate::routes::{Etat, Lecture, lire};
use crate::upstream::Params;

/// Tous les combien le relais relit les sujets d'un client.
pub const RELECTURE: Duration = Duration::from_secs(30);

/// Tous les combien il fait signe, même sans rien à dire.
pub const BATTEMENT: Duration = Duration::from_secs(25);

/// Ce que le client demande.
#[derive(Debug, Deserialize)]
struct Abonnement {
    latitude: f64,
    longitude: f64,
    #[serde(default = "sept")]
    jours: u32,
}

fn sept() -> u32 {
    7
}

/// Un sujet : son nom, et la requête qui le lit — chemin et paramètres,
/// tels qu'un client les enverrait.
#[derive(Debug, Clone)]
pub(crate) struct Sujet {
    pub nom: &'static str,
    pub chemin: String,
    pub params: Params,
}

/// Les six sujets d'une ville.
///
/// Les adresses viennent des mêmes fonctions que celles des clients
/// (`klima-api`) : ce sont donc les mêmes paramètres, donc les mêmes entrées de
/// cache que leurs requêtes HTTP.
pub(crate) fn sujets(latitude: f64, longitude: f64, jours: u32) -> Vec<Sujet> {
    let relais = Endpoints::relais("http://relais");
    let ville = Parcelle { name: String::new(), latitude, longitude, admin: None, country: None };
    let jours = jours.clamp(1, 16);
    [
        ("base", open_meteo::forecast_url(&relais, &ville, jours)),
        ("ensemble", ensemble::ensemble_url(&relais, &ville, jours)),
        ("quarts", veille::quarts_url(&relais, &ville)),
        ("met", readings::met_norway_call(&relais, latitude, longitude).url),
        ("station", readings::bright_sky_call(&relais, latitude, longitude).url),
        ("air", air::air_url(&relais, latitude, longitude)),
    ]
    .into_iter()
    .filter_map(|(nom, adresse)| {
        let url = Url::parse(&adresse).ok()?;
        Some(Sujet {
            nom,
            chemin: url.path().to_owned(),
            params: url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect(),
        })
    })
    .collect()
}

/// Le message d'un sujet, avec le point de l'abonnement. `None` si le corps
/// n'est pas du JSON : on ne pousse pas ce qu'un client ne saurait pas lire.
pub(crate) fn message(sujet: &str, point: (f64, f64), corps: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(corps).ok()?;
    let (latitude, longitude) = point;
    Some(format!(
        r#"{{"sujet":"{sujet}","latitude":{latitude},"longitude":{longitude},"corps":{corps}}}"#
    ))
}

fn empreinte(corps: &str) -> u64 {
    let mut h = DefaultHasher::new();
    corps.hash(&mut h);
    h.finish()
}

/// `GET /v1/direct` : la montée en WebSocket.
pub async fn ouvrir(ws: WebSocketUpgrade, State(etat): State<Etat>) -> Response {
    ws.on_upgrade(move |socket| suivre(socket, etat))
}

async fn suivre(mut socket: WebSocket, etat: Etat) {
    let mut abonnement: Vec<Sujet> = Vec::new();
    let mut point = (0.0, 0.0);
    let mut envoyes: HashMap<&'static str, u64> = HashMap::new();
    let mut releve = tokio::time::interval(RELECTURE);
    let mut battement = tokio::time::interval(BATTEMENT);
    // Le premier battement de chaque horloge part tout de suite : on les
    // laisse passer, l'abonnement fera le premier envoi.
    releve.tick().await;
    battement.tick().await;

    loop {
        tokio::select! {
            recu = socket.recv() => match recu {
                Some(Ok(Message::Text(texte))) => {
                    let Ok(demande) = serde_json::from_str::<Abonnement>(&texte) else { continue };
                    let valide = demande.latitude.is_finite()
                        && demande.longitude.is_finite()
                        && demande.latitude.abs() <= 90.0
                        && demande.longitude.abs() <= 180.0;
                    if !valide {
                        continue;
                    }
                    abonnement = sujets(demande.latitude, demande.longitude, demande.jours);
                    point = (demande.latitude, demande.longitude);
                    envoyes.clear();
                    if pousser(&mut socket, &etat, &abonnement, point, &mut envoyes).await.is_err() {
                        break;
                    }
                }
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
            _ = releve.tick(), if !abonnement.is_empty() => {
                if pousser(&mut socket, &etat, &abonnement, point, &mut envoyes).await.is_err() {
                    break;
                }
            }
            _ = battement.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

/// Relit tous les sujets — en même temps, par les caches — et pousse ceux
/// qui ont changé depuis le dernier envoi, toujours dans le même ordre.
async fn pousser(
    socket: &mut WebSocket,
    etat: &Etat,
    sujets: &[Sujet],
    point: (f64, f64),
    envoyes: &mut HashMap<&'static str, u64>,
) -> Result<(), axum::Error> {
    let mut lectures = JoinSet::new();
    for (rang, sujet) in sujets.iter().cloned().enumerate() {
        let etat = etat.clone();
        lectures.spawn(async move {
            let corps = match lire(&etat, &sujet.chemin, &sujet.params).await {
                Lecture::Lue(Ok(servi)) => Some(servi.value),
                _ => None,
            };
            (rang, sujet.nom, corps)
        });
    }
    let mut lus: Vec<(usize, &'static str, Option<String>)> = lectures.join_all().await;
    lus.sort_by_key(|(rang, _, _)| *rang);

    for (_, nom, corps) in lus {
        // Un fournisseur muet ne pousse rien : le client garde ce qu'il a.
        let Some(corps) = corps else { continue };
        let signe = empreinte(&corps);
        if envoyes.get(nom) == Some(&signe) {
            continue;
        }
        if let Some(texte) = message(nom, point, &corps) {
            socket.send(Message::Text(texte.into())).await?;
            envoyes.insert(nom, signe);
        }
    }
    Ok(())
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_sujets_par_les_routes_du_relais() {
        let s = sujets(48.8566, 2.3522, 7);
        let noms: Vec<&str> = s.iter().map(|s| s.nom).collect();
        assert_eq!(noms, ["base", "ensemble", "quarts", "met", "station", "air"]);
        assert_eq!(s[0].chemin, "/v1/open-meteo/forecast");
        assert!(s[0].params.contains_key("current"));
        assert_eq!(s[1].params["models"].split(',').count(), 7);
        assert!(s[2].params.contains_key("minutely_15"));
        assert_eq!(s[3].chemin, "/v1/met-norway/compact");
        assert_eq!(s[4].chemin, "/v1/bright-sky/current");
        assert_eq!(s[5].chemin, "/v1/open-meteo/air-quality");
        // Le point voyage tel quel : c'est le relais qui l'arrondit à sa maille.
        assert_eq!(s[0].params["latitude"], "48.8566");
        assert_eq!(s[3].params["lat"], "48.8566");
    }

    #[test]
    fn les_jours_restent_dans_ce_qu_open_meteo_sait_faire() {
        assert_eq!(sujets(0.0, 0.0, 0)[0].params["forecast_days"], "1");
        assert_eq!(sujets(0.0, 0.0, 99)[0].params["forecast_days"], "16");
    }

    #[test]
    fn un_message_porte_le_corps_tel_quel() {
        assert_eq!(
            message("base", (48.8566, 2.3522), r#"{"timezone":"Europe/Paris"}"#).as_deref(),
            Some(r#"{"sujet":"base","latitude":48.8566,"longitude":2.3522,"corps":{"timezone":"Europe/Paris"}}"#)
        );
        assert_eq!(message("base", (0.0, 0.0), "pas du json"), None);
    }
}
