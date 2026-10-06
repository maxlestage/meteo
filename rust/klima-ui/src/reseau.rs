//! Les appels qui partent du navigateur.
//!
//! Le décodage est dans `klima-api` et ne connaît pas le réseau ; ce module
//! est la moitié qui manque : il envoie, lit le corps, et traduit une panne en
//! `ApiError` — une clé de catalogue, pas une phrase.
//!
//! Ce que le navigateur ne fait pas : poser un `User-Agent`. C'est pour cela
//! que MET Norway n'est appelé d'ici qu'à travers le relais, et
//! `providers_for` le sait.

use gloo_net::http::Request;
use klima_api::open_meteo::{ApiError, Forecast, decode_forecast, decode_search};
use klima_api::plan::Verdict;
use klima_api::{ciel, ensemble, readings};
use klima_core::ciel::{CielObserve, plus_proche};
use klima_core::fusion::SerieSource;
use klima_core::endpoints::Endpoints;
use klima_core::position::Parcelle;
use klima_core::providers::{Platform, ProviderOutcome, providers_for};

/// Récupère un corps de réponse, ou dit pourquoi on n'a rien.
async fn texte(url: &str) -> Result<String, ApiError> {
    let reponse = Request::get(url).send().await.map_err(|_| ApiError::Unreachable)?;

    if !(200..300).contains(&reponse.status()) {
        return Err(ApiError::Status(reponse.status()));
    }
    reponse.text().await.map_err(|_| ApiError::Malformed)
}

/// La prévision d'une ville.
pub async fn forecast(
    endpoints: &Endpoints,
    parcelle: &Parcelle,
    days: u32,
    maintenant: i64,
) -> Result<Forecast, ApiError> {
    let url = klima_api::open_meteo::forecast_url(endpoints, parcelle, days);
    decode_forecast(parcelle.clone(), &texte(&url).await?, maintenant)
}

/// L'air d'une ville : qualité et pollens. `None` en cas de panne : la tuile
/// se tait plutôt que d'afficher un air pur qu'on n'a pas mesuré.
pub async fn air(endpoints: &Endpoints, parcelle: &Parcelle) -> Option<klima_core::air::AirSample> {
    let url = klima_api::air::air_url(endpoints, parcelle.latitude, parcelle.longitude);
    klima_api::air::decode_air(&texte(&url).await.ok()?)
}

/// Ce que le radar voit au-dessus d'une ville, et prévoit pour deux heures.
/// Seulement par un relais : c'est lui qui lit la mosaïque. `None` sans
/// relais ou en cas de panne.
pub async fn radar(endpoints: &Endpoints, parcelle: &Parcelle) -> Option<klima_core::radar::Prevision> {
    let url = klima_api::radar::radar_url(endpoints, parcelle.latitude, parcelle.longitude)?;
    klima_api::radar::decoder(&texte(&url).await.ok()?)
}

/// La prévision au quart d'heure d'une ville, pour le guetteur. `None` en cas
/// de panne : il se tait plutôt que de dire « sec » sans avoir regardé.
pub async fn quarts(endpoints: &Endpoints, parcelle: &Parcelle) -> Option<klima_api::veille::Quarts> {
    let url = klima_api::veille::quarts_url(endpoints, parcelle);
    klima_api::veille::decode_quarts(&texte(&url).await.ok()?)
}

/// Ce que le relais répond sur cette adresse — ou le fait qu'il se taise.
///
/// Un relais injoignable n'ouvre rien et ne retire rien : seul un « libre »
/// dit franchement est un refus. Sans cette distinction, un creux de réseau
/// coûterait son palier à qui l'a obtenu.
pub async fn plan(url: &str) -> Verdict {
    match texte(url).await {
        Ok(corps) => klima_api::plan::decode_verdict(&corps),
        Err(_) => Verdict::Injoignable,
    }
}

/// Les communes qui répondent à une recherche. Une requête trop courte ne part
/// pas : deux lettres au moins.
pub async fn search(endpoints: &Endpoints, requete: &str) -> Result<Vec<Parcelle>, ApiError> {
    let Some(url) = klima_api::open_meteo::search_url(endpoints, requete) else {
        return Ok(Vec::new());
    };
    decode_search(&texte(&url).await?)
}

/// Ce que tous les fournisseurs ont dit : de quoi recouper la prévision, et
/// de quoi dire leur accord.
pub struct Recoupement {
    /// Ce que chaque fournisseur a dit pour l'heure en cours — l'accord.
    pub outcomes: Vec<ProviderOutcome>,
    /// Les séries de chaque source — la prévision recoupée.
    pub series: Vec<SerieSource>,
    /// La température mesurée par une station proche, s'il y en a une.
    pub observation: Option<f64>,
    /// Le ciel de l'aéroport le plus proche, s'il y en a un assez près.
    pub ciel: Option<CielObserve>,
}

/// Interroge tous les fournisseurs permis, en même temps.
///
/// Les appels partent **ensemble** : à la file, la prévision recoupée
/// attendrait trois allers-retours au lieu d'un. Chaque fournisseur est isolé :
/// une panne, un refus ou une absence de couverture en écarte un seul.
///
/// Open-Meteo répond pour sept modèles d'un coup, sur toute la période : la
/// même réponse fait leurs séries et leur relevé de l'heure. MET Norway aussi
/// sert deux fois. Bright Sky, une station, ne vote que pour l'instant présent ;
/// les aéroports disent ce qui tombe.
pub async fn recoupement(
    endpoints: &Endpoints,
    parcelle: &Parcelle,
    days: u32,
    utc_offset_seconds: i64,
    maintenant: i64,
) -> Recoupement {
    let permis: Vec<&str> =
        providers_for(Platform::Web, endpoints.transport).iter().map(|p| p.id).collect();
    let appel = |id: &'static str, url: String| {
        let actif = permis.contains(&id);
        async move {
            if !actif {
                return None;
            }
            // Une absence n'est pas une panne : le fournisseur sort du
            // recoupement, et l'interface dit combien de sources ont parlé.
            Some(texte(&url).await.ok())
        }
    };

    let (open_meteo, met, bright_sky, aviation) = futures::join!(
        appel("open-meteo", ensemble::ensemble_url(endpoints, parcelle, days)),
        appel(
            "met-norway",
            readings::met_norway_call(endpoints, parcelle.latitude, parcelle.longitude).url
        ),
        appel(
            "bright-sky",
            readings::bright_sky_call(endpoints, parcelle.latitude, parcelle.longitude).url
        ),
        appel(
            "aviation-weather",
            ciel::metar_call(endpoints, parcelle.latitude, parcelle.longitude).url
        ),
    );

    lire_recoupement(
        Corps {
            open_meteo: open_meteo.map(Option::unwrap_or_default).as_deref(),
            met: met.map(Option::unwrap_or_default).as_deref(),
            bright_sky: bright_sky.map(Option::unwrap_or_default).as_deref(),
            aviation: aviation.flatten().as_deref(),
        },
        (parcelle.latitude, parcelle.longitude),
        utc_offset_seconds,
        maintenant,
    )
}

/// Les corps reçus de chaque fournisseur.
///
/// `None` : le fournisseur n'a pas été interrogé (il n'est pas permis d'ici) ;
/// `Some("")` : il l'a été et n'a rien dit. Le second compte dans l'accord
/// comme une source muette, le premier n'y entre pas. Les aéroports ne votent
/// pas : pour eux, les deux reviennent au même.
#[derive(Default)]
pub struct Corps<'a> {
    pub open_meteo: Option<&'a str>,
    pub met: Option<&'a str>,
    pub bright_sky: Option<&'a str>,
    pub aviation: Option<&'a str>,
}

/// Lit ce que les fournisseurs ont répondu — par requête ou par le direct.
/// `point` : la ville, pour trouver l'aéroport le plus proche ; `maintenant`
/// est à son heure.
pub fn lire_recoupement(
    corps: Corps<'_>,
    point: (f64, f64),
    utc_offset_seconds: i64,
    maintenant: i64,
) -> Recoupement {
    let Corps { open_meteo, met, bright_sky, aviation } = corps;
    let mut outcomes = Vec::new();
    let mut series = Vec::new();
    let mut observation = None;

    if let Some(corps) = open_meteo {
        series.extend(ensemble::decode_ensemble(corps));
        outcomes.push(ProviderOutcome {
            provider_id: "open-meteo".to_owned(),
            readings: readings::decode_open_meteo(corps, maintenant),
        });
    }
    if let Some(corps) = met {
        series.extend(ensemble::serie_met_norway(corps, utc_offset_seconds));
        outcomes.push(ProviderOutcome {
            provider_id: "met-norway".to_owned(),
            readings: readings::decode_met_norway(corps, maintenant - utc_offset_seconds * 1000),
        });
    }
    if let Some(corps) = bright_sky {
        let releves = readings::decode_bright_sky(corps);
        observation = releves.first().map(|r| r.temperature);
        outcomes.push(ProviderOutcome { provider_id: "bright-sky".to_owned(), readings: releves });
    }

    let ciel = aviation.and_then(|corps| {
        plus_proche(&ciel::decode_metars(corps, utc_offset_seconds), point.0, point.1, maintenant)
    });

    Recoupement { outcomes, series, observation, ciel }
}
