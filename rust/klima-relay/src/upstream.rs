//! Les interrogations qui partent vraiment.
//!
//! Le relais **ne décode rien** : il transmet la réponse du fournisseur telle
//! quelle. Les clients gardent donc leur code de décodage, et le jour où un
//! fournisseur ajoute un champ, il n'y a rien à changer ici.
//!
//! Deux choses ne peuvent se faire que de ce côté :
//!
//! - **La clé Open-Meteo.** Le plan gratuit est réservé à un usage non
//!   commercial ; dès que Klima se vend, il faut un plan payant, donc une clé.
//!   Une clé glissée dans une application est une clé publiée : elle vit ici,
//!   et nulle part ailleurs.
//! - **L'en-tête d'identification de MET Norway.** Leurs conditions l'exigent,
//!   et un navigateur n'a pas le droit de le poser.
//!
//! Construire l'adresse et l'appeler sont deux choses séparées : la première
//! est pure, donc testable sans réseau, et c'est elle qui porte les décisions
//! (quel hôte, quelle maille, quelle clé).

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use klima_core::providers::USER_AGENT;
use reqwest::Url;

/// Les paramètres d'une requête, nom par nom.
pub type Params = BTreeMap<String, String>;

const OPEN_METEO_PUBLIC: &str = "https://api.open-meteo.com";
const OPEN_METEO_CUSTOMER: &str = "https://customer-api.open-meteo.com";
const OPEN_METEO_GEOCODING: &str = "https://geocoding-api.open-meteo.com";
const OPEN_METEO_AIR_PUBLIC: &str = "https://air-quality-api.open-meteo.com";
const OPEN_METEO_AIR_CUSTOMER: &str = "https://customer-air-quality-api.open-meteo.com";
const MET_NORWAY: &str = "https://api.met.no";
const BRIGHT_SKY: &str = "https://api.brightsky.dev";
const AVIATION_WEATHER: &str = "https://aviationweather.gov";

/// Le fournisseur n'a pas donné ce qu'on lui demandait.
///
/// Le message n'y est pas, et c'est voulu : il peut contenir l'adresse
/// appelée, donc la clé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpstreamError {
    /// Le code renvoyé, ou `None` s'il n'a pas répondu du tout.
    pub status: Option<u16>,
}

/// Une interrogation prête à partir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub url: String,
    /// Posé seulement là où ses conditions l'exigent.
    pub user_agent: Option<&'static str>,
}

impl Call {
    fn plain(url: Url) -> Self {
        Call { url: url.into(), user_agent: None }
    }
}

/// Ce qui exécute l'interrogation.
///
/// Injectable, comme le `fetch` du relais TypeScript : les tests n'ont ni
/// réseau ni fournisseur, et l'essentiel de ce qu'il y a à vérifier — la
/// maille transmise, la clé, l'en-tête, le cache — se vérifie sans eux.
pub type Fetch = Arc<
    dyn Fn(Call) -> Pin<Box<dyn Future<Output = Result<String, UpstreamError>> + Send>>
        + Send
        + Sync,
>;

/// L'hôte à interroger : le client payant quand on a une clé, le public sinon.
fn open_meteo_host(key: Option<&str>) -> &'static str {
    if key.is_some() { OPEN_METEO_CUSTOMER } else { OPEN_METEO_PUBLIC }
}

/// Recopie les paramètres de la demande, en remplaçant le point par sa maille.
fn with_cell(mut url: Url, params: &Params, latitude: f64, longitude: f64) -> Url {
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in params {
            if key != "latitude" && key != "longitude" {
                pairs.append_pair(key, value);
            }
        }
        pairs.append_pair("latitude", &format!("{latitude:.3}"));
        pairs.append_pair("longitude", &format!("{longitude:.3}"));
    }
    url
}

pub fn open_meteo_forecast(
    key: Option<&str>,
    params: &Params,
    latitude: f64,
    longitude: f64,
) -> Call {
    let base = Url::parse(open_meteo_host(key)).expect("hôte");
    let mut url = with_cell(base.join("/v1/forecast").expect("chemin"), params, latitude, longitude);
    if let Some(key) = key {
        url.query_pairs_mut().append_pair("apikey", key);
    }
    Call::plain(url)
}

/// La qualité de l'air et les pollens, à la maille. La clé commerciale, quand
/// il y en a une, vaut aussi pour ce service — et passe donc par son hôte
/// payant.
pub fn open_meteo_air(key: Option<&str>, params: &Params, latitude: f64, longitude: f64) -> Call {
    let hote = if key.is_some() { OPEN_METEO_AIR_CUSTOMER } else { OPEN_METEO_AIR_PUBLIC };
    let base = Url::parse(hote).expect("hôte");
    let mut url = with_cell(base.join("/v1/air-quality").expect("chemin"), params, latitude, longitude);
    if let Some(key) = key {
        url.query_pairs_mut().append_pair("apikey", key);
    }
    Call::plain(url)
}

pub fn open_meteo_search(params: &Params) -> Call {
    let mut url =
        Url::parse(OPEN_METEO_GEOCODING).expect("hôte").join("/v1/search").expect("chemin");
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in params {
            pairs.append_pair(key, value);
        }
    }
    Call::plain(url)
}

pub fn met_norway_compact(latitude: f64, longitude: f64) -> Call {
    let mut url = Url::parse(MET_NORWAY)
        .expect("hôte")
        .join("/weatherapi/locationforecast/2.0/compact")
        .expect("chemin");
    url.query_pairs_mut()
        .append_pair("lat", &format!("{latitude:.3}"))
        .append_pair("lon", &format!("{longitude:.3}"));

    // Sans cet en-tête, MET Norway refuse la requête — c'est leur condition.
    Call { url: url.into(), user_agent: Some(USER_AGENT) }
}

pub fn bright_sky_current(latitude: f64, longitude: f64) -> Call {
    let mut url =
        Url::parse(BRIGHT_SKY).expect("hôte").join("/current_weather").expect("chemin");
    url.query_pairs_mut()
        .append_pair("lat", &format!("{latitude:.3}"))
        .append_pair("lon", &format!("{longitude:.3}"));
    Call::plain(url)
}

/// Les bulletins des aéroports autour de la maille. L'Aviation Weather Center
/// demande qu'on se nomme : le relais pose l'en-tête pour tout le monde.
pub fn aviation_metar(latitude: f64, longitude: f64) -> Call {
    let (sud, ouest, nord, est) = klima_core::ciel::cadre(latitude, longitude);
    let mut url = Url::parse(AVIATION_WEATHER).expect("hôte").join("/api/data/metar").expect("chemin");
    url.query_pairs_mut()
        .append_pair("bbox", &format!("{sud},{ouest},{nord},{est}"))
        .append_pair("format", "json");
    Call { url: url.into(), user_agent: Some(USER_AGENT) }
}

/// Le `Fetch` qui parle vraiment au réseau.
pub fn http_fetch(client: reqwest::Client) -> Fetch {
    Arc::new(move |call: Call| {
        let client = client.clone();
        Box::pin(async move {
            let mut requete = client.get(&call.url).header("Accept", "application/json");
            if let Some(agent) = call.user_agent {
                requete = requete.header("User-Agent", agent);
            }

            let reponse = requete.send().await.map_err(|erreur| {
                // On note de quoi diagnostiquer, et rien de plus : le message
                // de reqwest porte l'adresse appelée, donc la clé.
                eprintln!("fournisseur injoignable ({})", classer(&erreur));
                UpstreamError { status: None }
            })?;
            let status = reponse.status();
            if !status.is_success() {
                return Err(UpstreamError { status: Some(status.as_u16()) });
            }
            reponse.text().await.map_err(|_| UpstreamError { status: None })
        })
    })
}

/// Ce qui a cloché, en un mot — sans adresse, donc sans clé.
fn classer(erreur: &reqwest::Error) -> &'static str {
    if erreur.is_timeout() {
        "délai dépassé"
    } else if erreur.is_connect() {
        "connexion refusée"
    } else if erreur.is_decode() {
        "réponse illisible"
    } else {
        "autre"
    }
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    fn params(entries: &[(&str, &str)]) -> Params {
        entries.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect()
    }

    #[test]
    fn sans_cle_on_interroge_lhote_public() {
        let call = open_meteo_forecast(None, &Params::new(), 48.44, 1.48);
        assert!(call.url.starts_with("https://api.open-meteo.com/v1/forecast?"), "{}", call.url);
        assert!(!call.url.contains("apikey"));
    }

    #[test]
    fn avec_une_cle_on_passe_par_lhote_payant_et_la_cle_part_avec() {
        let call = open_meteo_forecast(Some("clé-secrète"), &Params::new(), 48.44, 1.48);
        assert!(call.url.starts_with("https://customer-api.open-meteo.com/v1/forecast?"));
        assert!(call.url.contains("apikey=cl%C3%A9-secr%C3%A8te"), "{}", call.url);
    }

    #[test]
    fn le_point_transmis_est_celui_de_la_maille_a_trois_decimales() {
        let call = open_meteo_forecast(None, &params(&[("latitude", "48.4468")]), 48.44, 1.48);
        assert!(call.url.contains("latitude=48.440"), "{}", call.url);
        assert!(call.url.contains("longitude=1.480"), "{}", call.url);
        // Le point demandé ne doit pas subsister à côté de la maille.
        assert!(!call.url.contains("48.4468"), "{}", call.url);
    }

    #[test]
    fn les_autres_parametres_de_la_demande_sont_recopies() {
        let call = open_meteo_forecast(
            None,
            &params(&[("hourly", "temperature_2m,precipitation"), ("models", "icon_seamless")]),
            48.44,
            1.48,
        );
        assert!(call.url.contains("models=icon_seamless"), "{}", call.url);
        assert!(call.url.contains("hourly=temperature_2m%2Cprecipitation"), "{}", call.url);
    }

    #[test]
    fn lair_passe_par_lhote_payant_quand_il_y_a_une_cle() {
        let public = open_meteo_air(None, &params(&[("current", "european_aqi")]), 48.86, 2.34);
        assert!(public.url.starts_with("https://air-quality-api.open-meteo.com/v1/air-quality?"));
        assert!(public.url.contains("current=european_aqi"), "{}", public.url);
        assert!(!public.url.contains("apikey"));

        let payant = open_meteo_air(Some("k"), &Params::new(), 48.86, 2.34);
        assert!(payant.url.starts_with("https://customer-air-quality-api.open-meteo.com/v1/air-quality?"));
        assert!(payant.url.contains("apikey=k"));
        assert_eq!(payant.user_agent, None);
    }

    #[test]
    fn met_norway_part_avec_len_tete_que_ses_conditions_exigent() {
        let call = met_norway_compact(48.4468, 1.4892);
        assert_eq!(call.user_agent, Some(USER_AGENT));
        assert!(call.user_agent.unwrap().contains("Klima/"));
        assert!(call.user_agent.unwrap().contains("http"));
        assert!(call.url.contains("lat=48.447"), "{}", call.url);
    }

    #[test]
    fn les_autres_fournisseurs_ne_se_nomment_pas() {
        // On ne pose un en-tête d'identification que là où il est exigé.
        assert_eq!(bright_sky_current(48.44, 1.48).user_agent, None);
    }

    #[test]
    fn les_aeroports_par_leur_cadre_en_se_nommant() {
        let call = aviation_metar(48.39, -4.49);
        assert_eq!(
            call.url,
            "https://aviationweather.gov/api/data/metar?bbox=48.03%2C-5.03%2C48.75%2C-3.95&format=json"
        );
        assert_eq!(call.user_agent, Some(USER_AGENT));
        assert_eq!(open_meteo_search(&Params::new()).user_agent, None);
    }

    #[test]
    fn le_geocodage_recopie_la_demande_telle_quelle() {
        let call = open_meteo_search(&params(&[("name", "Saint-Jean-de-Luz"), ("count", "8")]));
        assert!(call.url.starts_with("https://geocoding-api.open-meteo.com/v1/search?"));
        assert!(call.url.contains("name=Saint-Jean-de-Luz"), "{}", call.url);
        assert!(call.url.contains("count=8"), "{}", call.url);
    }

    #[test]
    fn une_erreur_de_fournisseur_ne_porte_pas_de_message() {
        // Le message pourrait contenir l'adresse appelée, donc la clé.
        let erreur = UpstreamError { status: Some(500) };
        assert_eq!(format!("{erreur:?}"), "UpstreamError { status: Some(500) }");
    }
}
