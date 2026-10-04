//! Les appels qui partent du navigateur.
//!
//! Le décodage est dans `klima-api` et ne connaît pas le réseau ; ce module
//! est la moitié qui manque : il envoie, lit le corps, et traduit une panne en
//! `AgroApiError` — une clé de catalogue, pas une phrase.
//!
//! Ce que le navigateur ne fait pas : poser un `User-Agent`. C'est pour cela
//! que MET Norway n'est appelé d'ici qu'à travers le relais, et
//! `providers_for` le sait.

use gloo_net::http::Request;
use klima_api::open_meteo::{AgroApiError, AgroForecast, decode_forecast, decode_search};
use klima_api::plan::Verdict;
use klima_api::readings;
use klima_core::endpoints::Endpoints;
use klima_core::position::Parcelle;
use klima_core::providers::{Platform, ProviderOutcome, providers_for};

/// Récupère un corps de réponse, ou dit pourquoi on n'a rien.
async fn texte(url: &str) -> Result<String, AgroApiError> {
    let reponse = Request::get(url).send().await.map_err(|_| AgroApiError::Unreachable)?;

    if !(200..300).contains(&reponse.status()) {
        return Err(AgroApiError::Status(reponse.status()));
    }
    reponse.text().await.map_err(|_| AgroApiError::Malformed)
}

/// La prévision agricole d'une parcelle.
pub async fn forecast(
    endpoints: &Endpoints,
    parcelle: &Parcelle,
    days: u32,
    maintenant: i64,
) -> Result<AgroForecast, AgroApiError> {
    let url = klima_api::open_meteo::forecast_url(endpoints, parcelle, days);
    decode_forecast(parcelle.clone(), &texte(&url).await?, maintenant)
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
pub async fn search(endpoints: &Endpoints, requete: &str) -> Result<Vec<Parcelle>, AgroApiError> {
    let Some(url) = klima_api::open_meteo::search_url(endpoints, requete) else {
        return Ok(Vec::new());
    };
    decode_search(&texte(&url).await?)
}

/// Interroge tous les fournisseurs permis et rend ce que chacun a dit.
///
/// Les trois appels partent **ensemble**, comme le `Promise.allSettled` du
/// TypeScript : à la file, le recoupement attendrait trois allers-retours au
/// lieu d'un, et l'accord des modèles apparaîtrait trois fois plus tard que la
/// météo qu'il commente.
///
/// Chaque fournisseur est isolé : une panne, un refus ou une absence de
/// couverture en écarte un seul. Le recoupement se fait sur ce qui a répondu.
pub async fn readings(
    endpoints: &Endpoints,
    parcelle: &Parcelle,
    maintenant: i64,
) -> Vec<ProviderOutcome> {
    let mut appels = Vec::new();

    for provider in providers_for(Platform::Web, endpoints.transport) {
        let (appel, lire): (_, fn(&str, i64) -> _) = match provider.id {
            "open-meteo" => (
                readings::open_meteo_call(endpoints, parcelle.latitude, parcelle.longitude),
                |corps, now| readings::decode_open_meteo(corps, now),
            ),
            "met-norway" => (
                readings::met_norway_call(endpoints, parcelle.latitude, parcelle.longitude),
                |corps, now| readings::decode_met_norway(corps, now),
            ),
            "bright-sky" => (
                readings::bright_sky_call(endpoints, parcelle.latitude, parcelle.longitude),
                |corps, _| readings::decode_bright_sky(corps),
            ),
            _ => continue,
        };

        appels.push(async move {
            let readings = match texte(&appel.url).await {
                Ok(corps) => lire(&corps, maintenant),
                // Une absence n'est pas une panne : le fournisseur sort du
                // recoupement, et l'interface dit combien de sources ont parlé.
                Err(_) => Vec::new(),
            };
            ProviderOutcome { provider_id: provider.id.to_owned(), readings }
        });
    }

    futures::future::join_all(appels).await
}
