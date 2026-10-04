//! Les routes du relais.
//!
//! Miroir de `server/src/index.ts`.
//!
//! Un seul rôle : interroger les fournisseurs météo une fois pour tout le
//! monde. Ce que ça change, dans l'ordre d'importance :
//!
//! 1. **Le prix tient.** Sans relais, la facture Open-Meteo suit le nombre
//!    d'utilisateurs ; avec lui, elle suit le nombre de parcelles distinctes et
//!    le rythme des modèles. Une cellule coûte vingt-quatre interrogations par
//!    jour, qu'elle soit ouverte par une personne ou par mille.
//! 2. **MET Norway reste sous son plafond.** Leurs conditions plafonnent à
//!    vingt requêtes par seconde *par application*, pas par appareil.
//! 3. **La clé commerciale reste secrète.** Elle vit ici, jamais dans un
//!    binaire distribué.
//!
//! Le relais n'est pas un point de défaillance unique : quand il ne répond
//! pas, les clients retombent sur les fournisseurs qu'ils ont le droit
//! d'appeler seuls. On perd des sources, pas la météo.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use klima_core::grid::{cell_for, cell_key};
use klima_core::plan::Plan;
use reqwest::Url;

use crate::cache::{ForecastCache, Served};
use crate::pro::Accord;
use crate::site;
use crate::upstream::{self, Fetch, Params, UpstreamError};

/// Une heure de fraîcheur. Les modèles ne tournent que quelques fois par
/// jour ; rafraîchir plus souvent ne change pas la réponse et multiplie la
/// facture.
pub const TTL_MS: i64 = 3_600_000;

/// Au-delà, une prévision périmée dépanne encore pendant deux heures.
pub const STALE_MS: i64 = 7_200_000;

/// Le géocodage ne bouge pas d'un jour à l'autre.
pub const SEARCH_TTL_MS: i64 = 86_400_000;

/// L'origine autorisée par défaut : celle du site publié.
pub const DEFAULT_ORIGIN: &str = "https://maxlestage.github.io";

#[derive(Clone)]
pub struct Etat {
    pub forecasts: Arc<ForecastCache<String>>,
    pub searches: Arc<ForecastCache<String>>,
    pub open_meteo_key: Option<String>,
    /// Ce que ce déploiement accorde comme palier, en plus de la boutique.
    pub accord_pro: Accord,
    /// Origines autorisées à appeler le relais depuis un navigateur.
    pub allowed_origins: Vec<String>,
    pub fetch: Fetch,
    /// Servir la vitrine et l'application depuis ce dossier, en plus de l'API.
    ///
    /// Absent, le relais ne fait que relayer. Présent, il devient aussi
    /// l'hébergement du site — et l'application se retrouve sur la même
    /// origine que ses appels.
    pub site: Option<PathBuf>,
}

/// Tout passe par un seul gestionnaire, comme dans le relais TypeScript : les
/// routes se lisent alors dans l'ordre où elles comptent, et la priorité de
/// l'API sur les fichiers du site se voit au lieu de se déduire.
pub fn router(etat: Etat) -> Router {
    Router::new().fallback(handle).with_state(etat)
}

async fn handle(State(etat): State<Etat>, request: Request) -> Response {
    let cors = cors_headers(
        request.headers().get(header::ORIGIN).and_then(|o| o.to_str().ok()),
        &etat.allowed_origins,
    );

    // L'URI d'une requête ne porte pas d'hôte : on en met un pour pouvoir
    // lire les paramètres avec le même décodage que les clients.
    let Ok(url) = Url::parse(&format!("http://relais{}", request.uri())) else {
        return texte(StatusCode::BAD_REQUEST, "requête illisible", &cors);
    };
    let chemin = url.path().to_owned();
    let params: Params = url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();

    if request.method() == Method::OPTIONS {
        let mut entetes = cors.clone();
        entetes.insert(
            HeaderName::from_static("access-control-allow-methods"),
            HeaderValue::from_static("GET, OPTIONS"),
        );
        return (StatusCode::NO_CONTENT, entetes).into_response();
    }

    if request.method() != Method::GET {
        return texte(StatusCode::METHOD_NOT_ALLOWED, "méthode non permise", &cors);
    }

    if chemin == "/health" {
        return json(StatusCode::OK, sante(&etat), &cors);
    }

    // Le palier que ce déploiement accorde, en plus de ce que dit la boutique.
    // La réponse ne porte que le verdict : jamais le code, jamais la règle.
    if chemin == "/v1/plan" {
        let accorde = etat.accord_pro.accorde(params.get("code").map(String::as_str));
        let plan = if accorde { Plan::Pro } else { Plan::Libre };
        return json(StatusCode::OK, format!(r#"{{"plan":"{}"}}"#, plan.code()), &cors);
    }

    // Les fichiers du site, s'il y en a. L'API garde la priorité : elle est
    // tout entière sous « /v1/ », et « /health » vient d'être traité.
    if let Some(racine) = &etat.site {
        if !chemin.starts_with("/v1/") {
            if let Some(fichier) = site::servir(racine, &chemin).await {
                let mut entetes = cors.clone();
                inserer(&mut entetes, header::CONTENT_TYPE, fichier.content_type);
                inserer(&mut entetes, header::CACHE_CONTROL, fichier.cache_control);
                return (StatusCode::OK, entetes, fichier.contenu).into_response();
            }
            // Un fichier absent est absent. Sans ce retour, la requête
            // tomberait sur la lecture des coordonnées et une image manquante
            // répondrait « coordonnées manquantes » — vrai, et incompréhensible.
            return texte(StatusCode::NOT_FOUND, "introuvable", &cors);
        }
    }

    // Le géocodage n'a pas de point : il se met en cache sur le texte cherché.
    if chemin == "/v1/open-meteo/search" {
        let nom = params.get("name").map(|n| n.trim().to_lowercase()).unwrap_or_default();
        if nom.chars().count() < 2 {
            return json(StatusCode::OK, "[]".to_owned(), &cors);
        }
        let appel = upstream::open_meteo_search(&params);
        let lu = etat
            .searches
            .serve(&format!("search:{nom}"), || (etat.fetch)(appel))
            .await;
        return servir(lu, &cors);
    }

    let Some((latitude, longitude)) = point_from(&params) else {
        return json(
            StatusCode::BAD_REQUEST,
            r#"{"erreur":"coordonnées manquantes ou hors bornes"}"#.to_owned(),
            &cors,
        );
    };
    let cell = cell_for(latitude, longitude);

    let (cle, appel) = match chemin.as_str() {
        "/v1/open-meteo/forecast" => {
            (
                cell_key(&format!("om:{}", empreinte(&params)), &cell),
                upstream::open_meteo_forecast(
                    etat.open_meteo_key.as_deref(),
                    &params,
                    cell.latitude,
                    cell.longitude,
                ),
            )
        }
        "/v1/met-norway/compact" => (
            cell_key("met", &cell),
            upstream::met_norway_compact(cell.latitude, cell.longitude),
        ),
        "/v1/bright-sky/current" => (
            cell_key("brightsky", &cell),
            upstream::bright_sky_current(cell.latitude, cell.longitude),
        ),
        _ => return texte(StatusCode::NOT_FOUND, "inconnu", &cors),
    };

    let lu = etat.forecasts.serve(&cle, || (etat.fetch)(appel)).await;
    servir(lu, &cors)
}

/// L'état de santé. Ni la clé ni le code n'y apparaissent — seulement le fait
/// qu'ils soient là.
fn sante(etat: &Etat) -> String {
    format!(
        r#"{{"statut":"ok","cellules":{},"interrogations":{},"cleOpenMeteo":"{}","pro":"{}"}}"#,
        etat.forecasts.size(),
        etat.forecasts.calls() + etat.searches.calls(),
        if etat.open_meteo_key.is_some() { "configurée" } else { "absente" },
        etat.accord_pro.etiquette()
    )
}

/// Les en-têtes de partage, s'ils sont dus.
///
/// Une étoile suffirait — le relais ne sert que des données publiques et ne lit
/// aucun cookie — mais nommer les origines évite qu'il serve de relais ouvert à
/// n'importe quel site.
fn cors_headers(origin: Option<&str>, allowed: &[String]) -> HeaderMap {
    let mut entetes = HeaderMap::new();
    let Some(origin) = origin else { return entetes };

    let etoile = allowed.iter().any(|o| o == "*");
    if !etoile && !allowed.iter().any(|o| o == origin) {
        return entetes;
    }

    let valeur = if etoile { "*" } else { origin };
    inserer(&mut entetes, header::ACCESS_CONTROL_ALLOW_ORIGIN, valeur);
    inserer(&mut entetes, header::VARY, "Origin");
    entetes
}

/// L'empreinte de ce qui change la réponse.
///
/// Le relais TypeScript ne mettait que `models` dans la clé : la vitrine, qui
/// demande deux jours, et l'application, qui en demande sept, partageaient
/// donc une entrée. Celle des deux qui arrivait la première servait l'autre —
/// une liste de sept jours qui n'en montre que deux, ou l'inverse.
///
/// Tout ce qui n'est pas le point entre donc dans la clé. On la condense
/// (FNV-1a, 64 bits) parce que les listes de variables font quatre cents
/// caractères, et qu'une clé de cache n'a pas à être lisible.
fn empreinte(params: &Params) -> String {
    let mut hachage: u64 = 0xcbf2_9ce4_8422_2325;
    for (nom, valeur) in params {
        if nom == "latitude" || nom == "longitude" || nom == "lat" || nom == "lon" {
            continue;
        }
        for octet in nom.bytes().chain(b"=".iter().copied()).chain(valeur.bytes()).chain(b"&".iter().copied())
        {
            hachage ^= u64::from(octet);
            hachage = hachage.wrapping_mul(0x1000_0000_01b3);
        }
    }
    format!("{hachage:016x}")
}

/// Lit et valide un point de la requête.
fn point_from(params: &Params) -> Option<(f64, f64)> {
    // Sans ce garde-fou, une requête sans coordonnées irait chercher la météo
    // du golfe de Guinée.
    let brut = |noms: [&str; 2]| noms.iter().find_map(|nom| params.get(*nom));
    let latitude: f64 = brut(["latitude", "lat"])?.parse().ok()?;
    let longitude: f64 = brut(["longitude", "lon"])?.parse().ok()?;

    if !latitude.is_finite() || !longitude.is_finite() {
        return None;
    }
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        return None;
    }
    Some((latitude, longitude))
}

/// Emballe une lecture de cache en réponse, en disant d'où elle vient.
fn servir(lu: Result<Served<String>, UpstreamError>, cors: &HeaderMap) -> Response {
    match lu {
        Ok(served) => {
            let mut entetes = cors.clone();
            inserer(&mut entetes, header::CONTENT_TYPE, "application/json; charset=utf-8");
            inserer(
                &mut entetes,
                HeaderName::from_static("x-klima-cache"),
                served.freshness.label(),
            );
            inserer(&mut entetes, header::AGE, &served.age_seconds.to_string());
            inserer(
                &mut entetes,
                header::CACHE_CONTROL,
                &format!("public, max-age={}", TTL_MS / 1000),
            );
            (StatusCode::OK, entetes, served.value).into_response()
        }
        Err(erreur) => {
            // On ne renvoie jamais l'erreur du fournisseur telle quelle : elle
            // peut contenir l'URL, donc la clé.
            let status = if erreur.status == Some(429) {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::BAD_GATEWAY
            };
            json(status, r#"{"erreur":"fournisseur indisponible"}"#.to_owned(), cors)
        }
    }
}

fn json(status: StatusCode, corps: String, cors: &HeaderMap) -> Response {
    let mut entetes = cors.clone();
    inserer(&mut entetes, header::CONTENT_TYPE, "application/json; charset=utf-8");
    (status, entetes, corps).into_response()
}

fn texte(status: StatusCode, corps: &str, cors: &HeaderMap) -> Response {
    (status, cors.clone(), Body::from(corps.to_owned())).into_response()
}

fn inserer<N: Into<HeaderName>>(entetes: &mut HeaderMap, nom: N, valeur: &str) {
    if let Ok(valeur) = HeaderValue::from_str(valeur) {
        entetes.insert(nom.into(), valeur);
    }
}

/// Un état de relais tout monté, pour les tests et pour `main`.
pub fn etat(fetch: Fetch, now: crate::cache::Clock) -> Etat {
    use crate::cache::CacheOptions;

    Etat {
        forecasts: Arc::new(ForecastCache::new(CacheOptions {
            ttl_ms: TTL_MS,
            stale_ms: STALE_MS,
            now: now.clone(),
        })),
        searches: Arc::new(ForecastCache::new(CacheOptions {
            ttl_ms: SEARCH_TTL_MS,
            stale_ms: SEARCH_TTL_MS,
            now,
        })),
        open_meteo_key: None,
        accord_pro: Accord::Aucun,
        allowed_origins: vec![DEFAULT_ORIGIN.to_owned()],
        fetch,
        site: None,
    }
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upstream::Call;
    use axum::body::to_bytes;
    use std::sync::Mutex;
    use tower::ServiceExt;

    /// Un fournisseur qui note ce qu'on lui demande et répond ce qu'on lui dit.
    #[derive(Clone)]
    struct Faux {
        vues: Arc<Mutex<Vec<Call>>>,
        reponse: Arc<dyn Fn() -> Result<String, UpstreamError> + Send + Sync>,
    }

    impl Faux {
        fn new() -> Self {
            Faux {
                vues: Arc::new(Mutex::new(Vec::new())),
                reponse: Arc::new(|| Ok(r#"{"ok":true}"#.to_owned())),
            }
        }

        fn repond(mut self, corps: &'static str) -> Self {
            self.reponse = Arc::new(move || Ok(corps.to_owned()));
            self
        }

        fn echoue(mut self, status: u16) -> Self {
            self.reponse = Arc::new(move || Err(UpstreamError { status: Some(status) }));
            self
        }

        fn fetch(&self) -> Fetch {
            let vues = self.vues.clone();
            let reponse = self.reponse.clone();
            Arc::new(move |call: Call| {
                vues.lock().unwrap().push(call);
                let resultat = reponse();
                Box::pin(async move { resultat })
            })
        }

        fn appels(&self) -> usize {
            self.vues.lock().unwrap().len()
        }

        fn premiere_url(&self) -> String {
            self.vues.lock().unwrap().first().expect("un appel").url.clone()
        }
    }

    fn relais(faux: &Faux) -> Etat {
        etat(faux.fetch(), Arc::new(|| 1_700_000_000_000))
    }

    async fn demande(etat: Etat, chemin: &str, origine: Option<&str>) -> (StatusCode, HeaderMap, String) {
        let mut requete = Request::builder().uri(chemin);
        if let Some(origine) = origine {
            requete = requete.header("Origin", origine);
        }
        let reponse = router(etat)
            .oneshot(requete.body(Body::empty()).unwrap())
            .await
            .expect("réponse");

        let status = reponse.status();
        let entetes = reponse.headers().clone();
        let corps = to_bytes(reponse.into_body(), 1 << 20).await.unwrap();
        (status, entetes, String::from_utf8_lossy(&corps).into_owned())
    }

    async fn get(etat: Etat, chemin: &str) -> (StatusCode, HeaderMap, String) {
        demande(etat, chemin, None).await
    }

    #[tokio::test]
    async fn sert_la_prevision_et_dit_quelle_est_fraiche() {
        let faux = Faux::new().repond(r#"{"timezone":"Europe/Paris"}"#);
        let (status, entetes, corps) = get(
            relais(&faux),
            "/v1/open-meteo/forecast?latitude=48.4468&longitude=1.4892",
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(corps, r#"{"timezone":"Europe/Paris"}"#);
        assert_eq!(entetes["x-klima-cache"], "frais");
        assert_eq!(faux.appels(), 1);
    }

    #[tokio::test]
    async fn deux_parcelles_de_la_meme_maille_ne_font_quune_interrogation() {
        let faux = Faux::new();
        let etat = relais(&faux);

        let _ = get(etat.clone(), "/v1/open-meteo/forecast?latitude=48.4468&longitude=1.4870").await;
        let (_, entetes, _) =
            get(etat, "/v1/open-meteo/forecast?latitude=48.4490&longitude=1.4885").await;

        assert_eq!(faux.appels(), 1);
        assert_eq!(entetes["x-klima-cache"], "cache");
    }

    #[tokio::test]
    async fn le_point_transmis_est_celui_de_la_maille_pas_celui_demande() {
        let faux = Faux::new();
        let _ = get(relais(&faux), "/v1/open-meteo/forecast?latitude=48.4468&longitude=1.4892")
            .await;

        let url = faux.premiere_url();
        assert!(url.contains("latitude=48.440"), "{url}");
        assert!(url.contains("longitude=1.480"), "{url}");
    }

    #[tokio::test]
    async fn deux_demandes_de_portee_differente_ne_partagent_pas_leur_entree() {
        // La vitrine demande deux jours, l'application sept. Avec une seule
        // entrée pour les deux, la première arrivée servirait l'autre.
        let faux = Faux::new();
        let etat = relais(&faux);

        let _ = get(
            etat.clone(),
            "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48&forecast_days=2",
        )
        .await;
        let _ = get(
            etat,
            "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48&forecast_days=7",
        )
        .await;

        assert_eq!(faux.appels(), 2);
    }

    #[tokio::test]
    async fn deux_demandes_identiques_a_la_maille_pres_partagent_la_leur() {
        let faux = Faux::new();
        let etat = relais(&faux);
        let demande = |lat: &str| {
            format!("/v1/open-meteo/forecast?latitude={lat}&longitude=1.48&forecast_days=7&hourly=temperature_2m")
        };

        let _ = get(etat.clone(), &demande("48.441")).await;
        let (_, entetes, _) = get(etat, &demande("48.444")).await;

        assert_eq!(faux.appels(), 1);
        assert_eq!(entetes["x-klima-cache"], "cache");
    }

    #[tokio::test]
    async fn la_comparaison_des_modeles_est_un_cache_distinct_de_la_prevision_seule() {
        let faux = Faux::new();
        let etat = relais(&faux);

        let _ = get(etat.clone(), "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48").await;
        let _ = get(
            etat,
            "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48&models=icon_seamless",
        )
        .await;

        assert_eq!(faux.appels(), 2);
    }

    #[tokio::test]
    async fn met_norway_part_avec_len_tete_que_ses_conditions_exigent() {
        let faux = Faux::new();
        let _ = get(relais(&faux), "/v1/met-norway/compact?lat=48.44&lon=1.48").await;

        let agent = faux.vues.lock().unwrap()[0].user_agent.expect("en-tête");
        assert!(agent.contains("Klima/"));
        assert!(agent.contains("http"));
    }

    #[tokio::test]
    async fn la_cle_commerciale_part_chez_open_meteo_et_jamais_vers_le_client() {
        let faux = Faux::new();
        let mut etat = relais(&faux);
        etat.open_meteo_key = Some("clé-secrète".to_owned());

        let (_, _, corps) =
            get(etat, "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48").await;

        let url = faux.premiere_url();
        assert!(url.starts_with("https://customer-api.open-meteo.com/"), "{url}");
        assert!(url.contains("apikey=cl%C3%A9-secr%C3%A8te"), "{url}");
        assert!(!corps.contains("clé-secrète"));
    }

    #[tokio::test]
    async fn une_panne_du_fournisseur_ne_fuit_ni_son_message_ni_son_url() {
        let faux = Faux::new().echoue(500);
        let mut etat = relais(&faux);
        etat.open_meteo_key = Some("clé-secrète".to_owned());

        let (status, _, corps) =
            get(etat, "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48").await;

        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert!(!corps.contains("clé-secrète"));
        assert!(!corps.contains("open-meteo"));
    }

    #[tokio::test]
    async fn un_fournisseur_qui_sature_garde_son_code() {
        // 429 se transmet : le client sait qu'il doit se calmer, pas réessayer.
        let faux = Faux::new().echoue(429);
        let (status, _, _) =
            get(relais(&faux), "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48").await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn des_coordonnees_absurdes_sont_refusees_avant_tout_appel() {
        let faux = Faux::new();
        let etat = relais(&faux);

        for query in ["latitude=95&longitude=1", "latitude=abc&longitude=1", ""] {
            let (status, _, _) =
                get(etat.clone(), &format!("/v1/open-meteo/forecast?{query}")).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{query}");
        }
        assert_eq!(faux.appels(), 0);
    }

    #[tokio::test]
    async fn le_geocodage_se_met_en_cache_sur_le_texte_quelle_que_soit_la_casse() {
        let faux = Faux::new().repond(r#"{"results":[]}"#);
        let etat = relais(&faux);

        let _ = get(etat.clone(), "/v1/open-meteo/search?name=Reims").await;
        let _ = get(etat.clone(), "/v1/open-meteo/search?name=reims").await;
        let _ = get(etat, "/v1/open-meteo/search?name=%20%20REIMS%20%20").await;

        assert_eq!(faux.appels(), 1);
    }

    #[tokio::test]
    async fn une_recherche_trop_courte_ne_part_pas() {
        let faux = Faux::new();
        let (_, _, corps) = get(relais(&faux), "/v1/open-meteo/search?name=r").await;

        assert_eq!(corps, "[]");
        assert_eq!(faux.appels(), 0);
    }

    #[tokio::test]
    async fn seules_les_origines_nommees_obtiennent_len_tete_de_partage() {
        let faux = Faux::new();
        let etat = relais(&faux);
        let chemin = "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48";

        let (_, ami, _) = demande(etat.clone(), chemin, Some(DEFAULT_ORIGIN)).await;
        let (_, inconnu, _) = demande(etat, chemin, Some("https://pirate.example")).await;

        assert_eq!(ami["access-control-allow-origin"], DEFAULT_ORIGIN);
        assert!(!inconnu.contains_key("access-control-allow-origin"));
    }

    #[tokio::test]
    async fn letat_de_sante_ne_revele_pas_la_cle() {
        let faux = Faux::new();
        let mut etat = relais(&faux);
        etat.open_meteo_key = Some("clé-secrète".to_owned());

        let (_, _, corps) = get(etat, "/health").await;
        assert!(corps.contains("configurée"));
        assert!(!corps.contains("clé-secrète"));
    }

    /* ---- le palier accordé par le déploiement ---- */

    #[tokio::test]
    async fn sans_rien_de_configure_le_relais_naccorde_aucun_palier() {
        // StoreKit décide seul, comme en production.
        let faux = Faux::new();
        let (_, _, corps) = get(relais(&faux), "/v1/plan").await;
        assert_eq!(corps, r#"{"plan":"libre"}"#);
    }

    #[tokio::test]
    async fn le_mode_tous_accorde_a_qui_demande() {
        let faux = Faux::new();
        let mut etat = relais(&faux);
        etat.accord_pro = Accord::Tous;

        let (status, _, corps) = get(etat, "/v1/plan").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(corps, r#"{"plan":"pro"}"#);
    }

    #[tokio::test]
    async fn un_code_naccorde_quau_code_et_le_relais_ne_le_repete_jamais() {
        let faux = Faux::new();
        let mut etat = relais(&faux);
        etat.accord_pro = Accord::SurCode("sillon-2026-dUx7".to_owned());

        let (_, _, juste) = get(etat.clone(), "/v1/plan?code=sillon-2026-dUx7").await;
        assert_eq!(juste, r#"{"plan":"pro"}"#);

        let (_, _, faux_code) = get(etat.clone(), "/v1/plan?code=sillon").await;
        assert_eq!(faux_code, r#"{"plan":"libre"}"#);

        let (_, _, sans) = get(etat.clone(), "/v1/plan").await;
        assert_eq!(sans, r#"{"plan":"libre"}"#);

        // Ni la réponse ni l'état de santé ne redisent le code.
        let (_, _, sante) = get(etat, "/health").await;
        assert!(!sante.contains("sillon"), "{sante}");
        assert!(sante.contains(r#""pro":"sur code""#), "{sante}");
    }

    #[tokio::test]
    async fn letat_de_sante_dit_la_situation_sans_la_regle() {
        let faux = Faux::new();
        let mut etat = relais(&faux);
        etat.accord_pro = Accord::Tous;

        let (_, _, corps) = get(etat, "/health").await;
        assert!(corps.contains(r#""pro":"tous""#), "{corps}");
    }

    #[tokio::test]
    async fn un_chemin_inconnu_repond_404_sans_rien_interroger() {
        let faux = Faux::new();
        let (status, _, _) = get(relais(&faux), "/v1/autre?latitude=48&longitude=1").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(faux.appels(), 0);
    }

    /* ---- le relais qui sert aussi le site ---- */

    /// Un site d'un seul fichier, dans un dossier à soi : deux tests qui
    /// partageraient le même pourraient lire un `index.html` tronqué par
    /// l'écriture de l'autre.
    async fn avec_site(faux: &Faux, nom: &str) -> Etat {
        let racine = std::env::temp_dir().join(format!("klima-routes-{nom}"));
        tokio::fs::create_dir_all(&racine).await.unwrap();
        tokio::fs::write(racine.join("index.html"), b"<!doctype html>").await.unwrap();

        let mut etat = relais(faux);
        etat.site = Some(racine);
        etat
    }

    #[tokio::test]
    async fn la_racine_rend_la_page_au_lieu_dune_erreur_de_coordonnees() {
        let faux = Faux::new();
        let etat = avec_site(&faux, "racine").await;

        let (status, _, corps) = get(etat, "/").await;
        assert_eq!(status, StatusCode::OK);
        assert!(corps.contains("doctype"));
    }

    #[tokio::test]
    async fn le_site_ne_prend_jamais_le_pas_sur_lapi() {
        // Sans ce partage net, un fichier nommé comme une route mangerait
        // l'API — ou l'inverse, plus silencieusement encore.
        let faux = Faux::new();
        let etat = avec_site(&faux, "priorite").await;

        let (_, entetes, _) =
            get(etat.clone(), "/v1/met-norway/compact?latitude=48.44&longitude=1.49").await;
        assert!(entetes[header::CONTENT_TYPE].to_str().unwrap().contains("json"));
        assert_eq!(faux.appels(), 1);

        let (_, _, sante) = get(etat, "/health").await;
        assert!(sante.contains(r#""statut":"ok""#));
    }

    #[tokio::test]
    async fn un_fichier_absent_repond_404_pas_une_erreur_de_coordonnees() {
        let faux = Faux::new();
        let etat = avec_site(&faux, "absent").await;

        let (status, _, _) = get(etat, "/inconnu.png").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn sans_site_la_racine_repond_comme_avant() {
        let faux = Faux::new();
        let (status, _, corps) = get(relais(&faux), "/").await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(corps.contains("erreur"));
    }

    #[tokio::test]
    async fn une_demande_de_permission_recoit_les_methodes_permises() {
        let faux = Faux::new();
        let reponse = router(relais(&faux))
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/v1/open-meteo/forecast")
                    .header("Origin", DEFAULT_ORIGIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(reponse.status(), StatusCode::NO_CONTENT);
        assert_eq!(reponse.headers()["access-control-allow-methods"], "GET, OPTIONS");
    }

    #[tokio::test]
    async fn une_ecriture_nest_pas_permise() {
        let faux = Faux::new();
        let reponse = router(relais(&faux))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/v1/open-meteo/forecast")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(reponse.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(faux.appels(), 0);
    }
}
