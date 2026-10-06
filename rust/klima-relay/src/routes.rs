//! Les routes du relais.
//!
//! Un seul rôle : interroger les fournisseurs météo une fois pour tout le
//! monde. Ce que ça change, dans l'ordre d'importance :
//!
//! 1. **Le prix tient.** Sans relais, la facture Open-Meteo suit le nombre
//!    d'utilisateurs ; avec lui, elle suit le nombre de villes distinctes et la
//!    durée de vie des caches. Une ville suivie coûte le même nombre
//!    d'interrogations, qu'elle soit ouverte par une personne ou par mille —
//!    par requête comme en direct (`direct.rs`), qui lit les mêmes caches.
//! 2. **MET Norway reste sous son plafond.** Leurs conditions plafonnent à
//!    vingt requêtes par seconde *par application*, pas par appareil.
//! 3. **La clé commerciale reste secrète.** Elle vit ici, jamais dans un
//!    binaire distribué.
//!
//! Le relais n'est pas un point de défaillance unique : quand il ne répond
//! pas, les clients retombent sur les fournisseurs qu'ils ont le droit
//! d'appeler seuls. On perd des sources, pas la météo.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use klima_core::grid::{cell_for, cell_key};
use klima_core::plan::Plan;
use reqwest::Url;

use crate::cache::{Clock, ForecastCache, Served};
use crate::apns::Apns;
use crate::identite::{self, Cles, Identite, Refus};
use crate::iles::{self, Iles};
use crate::pro::Accord;
use crate::site;
use crate::upstream::{self, Call, Fetch, Params, UpstreamError};

/// Une heure de fraîcheur. Les modèles ne tournent que quelques fois par
/// jour ; rafraîchir plus souvent ne change pas la réponse et multiplie la
/// facture.
pub const TTL_MS: i64 = 3_600_000;

/// Au-delà, une prévision périmée dépanne encore pendant deux heures.
pub const STALE_MS: i64 = 7_200_000;

/// La prévision au quart d'heure du guetteur ne vaut que pour la demi-heure :
/// cinq minutes de fraîcheur, et elle ne dépanne plus au-delà d'un quart
/// d'heure de retard. Un « sec » vieux d'une heure serait un mensonge.
pub const QUARTS_TTL_MS: i64 = 300_000;
pub const QUARTS_STALE_MS: i64 = 900_000;

/// L'instant présent (`current`) et l'observation de station : Open-Meteo
/// recalcule le premier tous les quarts d'heure. Dix minutes de fraîcheur, et
/// le direct le pousse dès qu'il a changé.
pub const COURANTS_TTL_MS: i64 = 600_000;
pub const COURANTS_STALE_MS: i64 = 1_800_000;

/// Le géocodage ne bouge pas d'un jour à l'autre.
pub const SEARCH_TTL_MS: i64 = 86_400_000;

/// L'origine autorisée par défaut : celle du site publié.
pub const DEFAULT_ORIGIN: &str = "https://maxlestage.github.io";

#[derive(Clone)]
pub struct Etat {
    pub forecasts: Arc<ForecastCache<String>>,
    /// Les prévisions au quart d'heure : un cache à part, plus court.
    pub quarts: Arc<ForecastCache<String>>,
    /// L'instant présent et la station : un cache à part aussi.
    pub courants: Arc<ForecastCache<String>>,
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
    /// L'horloge, en millisecondes : pour les échéances des jetons.
    pub now: Clock,
    /// Ce qu'il faut pour reconnaître un compte. Absent, les comptes sont
    /// désactivés — et `/v1/session` le dit plutôt que de faire semblant.
    pub comptes: Option<Comptes>,
    /// Les îles dynamiques suivies, à qui pousser l'heure qui commence.
    pub iles: Arc<Iles>,
    /// De quoi pousser. Absent, les îles basculent seules à l'heure pile, et
    /// `/v1/activites` répond qu'il n'y a pas de poussée.
    pub apns: Option<Apns>,
}

/// Les comptes : de quoi vérifier ce qu'Apple prouve, et signer ce qu'on en tire.
#[derive(Clone)]
pub struct Comptes {
    /// Le secret des sessions. Il vit dans l'environnement du relais et nulle
    /// part ailleurs ; le changer déconnecte tout le monde, d'un coup.
    pub secret: Arc<Vec<u8>>,
    /// L'identifiant de l'application, que les jetons d'Apple doivent viser.
    pub audience: String,
    /// Où lire les clés publiques d'Apple.
    pub adresse_cles: String,
    /// Les clés lues, et quand. Apple les fait tourner rarement : une heure de
    /// mémoire suffit, et une clé inconnue déclenche une relecture.
    pub cles: Arc<RwLock<Option<(i64, Cles)>>>,
}

impl Comptes {
    pub fn new(secret: Vec<u8>) -> Self {
        Comptes {
            secret: Arc::new(secret),
            audience: identite::AUDIENCE_PAR_DEFAUT.to_owned(),
            adresse_cles: identite::APPLE_CLES.to_owned(),
            cles: Arc::new(RwLock::new(None)),
        }
    }
}

/// Garder les clés d'Apple une heure.
const CLES_TTL_MS: i64 = 3_600_000;

/// Une clé inconnue ne relance la lecture qu'une fois toutes les cinq minutes :
/// sans ce frein, n'importe qui pourrait faire marteler Apple par le relais en
/// lui présentant des jetons à clé fantaisiste.
const CLES_RELECTURE_MIN_MS: i64 = 300_000;

/// Tout passe par un seul gestionnaire, comme dans le relais TypeScript : les
/// routes se lisent alors dans l'ordre où elles comptent, et la priorité de
/// l'API sur les fichiers du site se voit au lieu de se déduire.
///
/// Une exception : le direct (`/v1/direct`), qui monte en WebSocket et a donc
/// besoin de son propre extracteur.
pub fn router(etat: Etat) -> Router {
    Router::new()
        .route("/v1/direct", axum::routing::get(crate::direct::ouvrir))
        .fallback(handle)
        .with_state(etat)
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

    // Le porteur d'une session, s'il y en a un. Lu ici parce que la requête
    // peut être consommée plus bas, pour son corps.
    let porteur: Option<String> = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());

    if request.method() == Method::OPTIONS {
        let mut entetes = cors.clone();
        entetes.insert(
            HeaderName::from_static("access-control-allow-methods"),
            HeaderValue::from_static("GET, OPTIONS"),
        );
        return (StatusCode::NO_CONTENT, entetes).into_response();
    }

    // La seule route qui reçoit un corps : l'échange d'un jeton d'Apple contre
    // une session. Tout le reste est en lecture.
    if chemin == "/v1/session" {
        if request.method() != Method::POST {
            return texte(StatusCode::METHOD_NOT_ALLOWED, "méthode non permise", &cors);
        }
        return ouvrir_session(&etat, request.into_body(), &cors).await;
    }

    // Les îles dynamiques : l'iPhone inscrit celle qu'il ouvre, et la retire
    // quand il la ferme.
    if chemin == "/v1/activites" {
        let methode = request.method().clone();
        if methode != Method::POST && methode != Method::DELETE {
            return texte(StatusCode::METHOD_NOT_ALLOWED, "méthode non permise", &cors);
        }
        return activite(&etat, methode, request.into_body(), &cors).await;
    }

    if request.method() != Method::GET {
        return texte(StatusCode::METHOD_NOT_ALLOWED, "méthode non permise", &cors);
    }

    if chemin == "/health" {
        return json(StatusCode::OK, sante(&etat), &cors);
    }

    // Le palier que ce déploiement accorde, en plus de ce que dit la boutique.
    // La réponse ne porte que le verdict : jamais le code, jamais la liste,
    // jamais la règle. Elle ne dit pas non plus *pourquoi* c'est non : savoir
    // qu'une adresse est inconnue de la liste, c'est pouvoir énumérer la liste.
    if chemin == "/v1/plan" {
        // « courriel » d'abord, « email » ensuite : le second n'est là que pour
        // qu'un essai à la main ne réponde pas « libre » sur un nom de
        // paramètre, ce qui se cherche longtemps.
        //
        // Une session, quand il y en a une, passe avant tout : son adresse est
        // celle qu'Apple a prouvée, et une adresse en paramètre ne la remplace
        // pas. Une session illisible ou échue répond 401 — une réponse franche,
        // que l'application lit comme « reconnectez-vous », et non comme un
        // silence à ignorer.
        let verifiee: Option<String> = match (&porteur, &etat.comptes) {
            (None, _) => None,
            (Some(jeton), Some(comptes)) => {
                match identite::lire_session(jeton, &comptes.secret, (etat.now)() / 1000) {
                    Ok(identite) => Some(identite.courriel),
                    Err(_) => return json(StatusCode::UNAUTHORIZED, r#"{"erreur":"session"}"#.to_owned(), &cors),
                }
            }
            (Some(_), None) => {
                return json(StatusCode::UNAUTHORIZED, r#"{"erreur":"session"}"#.to_owned(), &cors);
            }
        };
        let courriel = verifiee
            .as_ref()
            .or_else(|| params.get("courriel"))
            .or_else(|| params.get("email"));
        let accorde = etat
            .accord_pro
            .accorde(params.get("code").map(String::as_str), courriel.map(String::as_str));
        let plan = if accorde { Plan::Pro } else { Plan::Libre };
        return json(StatusCode::OK, format!(r#"{{"plan":"{}"}}"#, plan.code()), &cors);
    }

    // Les fichiers du site, s'il y en a. L'API garde la priorité : elle est
    // tout entière sous « /v1/ », et « /health » vient d'être traité.
    //
    // Ce qui n'est pas sous « /v1/ » s'arrête ici, qu'il y ait un site ou non.
    // Sans cela la requête tomberait sur la lecture des coordonnées, et une
    // image manquante répondrait « coordonnées manquantes » — vrai, et
    // incompréhensible. Le cas sans site n'est pas théorique : c'est celui du
    // déploiement, où les interfaces sont publiées ailleurs, et où « / »
    // répondait donc 400 à qui ouvrait simplement l'adresse.
    if !chemin.starts_with("/v1/") {
        if let Some(racine) = &etat.site {
            if let Some(fichier) = site::servir(racine, &chemin).await {
                let mut entetes = cors.clone();
                inserer(&mut entetes, header::CONTENT_TYPE, fichier.content_type);
                inserer(&mut entetes, header::CACHE_CONTROL, fichier.cache_control);
                return (StatusCode::OK, entetes, fichier.contenu).into_response();
            }
        }
        return texte(StatusCode::NOT_FOUND, "introuvable", &cors);
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

    match lire(&etat, &chemin, &params).await {
        Lecture::HorsBornes => json(
            StatusCode::BAD_REQUEST,
            r#"{"erreur":"coordonnées manquantes ou hors bornes"}"#.to_owned(),
            &cors,
        ),
        Lecture::Inconnue => texte(StatusCode::NOT_FOUND, "inconnu", &cors),
        Lecture::Lue(lu) => servir(lu, &cors),
    }
}

/// Ce que donne une demande de fournisseur.
pub(crate) enum Lecture {
    /// Pas de point, ou un point hors du globe.
    HorsBornes,
    /// Une route que le relais ne connaît pas.
    Inconnue,
    Lue(Result<Served<String>, UpstreamError>),
}

/// Lit une donnée de fournisseur, par le cache de sa maille.
///
/// Le même chemin pour une requête HTTP et pour le direct (`direct.rs`) : les
/// deux partagent donc les mêmes entrées de cache, et une ville suivie en
/// direct ne coûte pas une interrogation de plus à qui la lit par HTTP.
pub(crate) async fn lire(etat: &Etat, chemin: &str, params: &Params) -> Lecture {
    let Some((latitude, longitude)) = point_from(params) else {
        return Lecture::HorsBornes;
    };
    let cell = cell_for(latitude, longitude);

    let (cle, appel) = match chemin {
        "/v1/open-meteo/forecast" => (
            cell_key(&format!("om:{}", empreinte(params)), &cell),
            upstream::open_meteo_forecast(etat.open_meteo_key.as_deref(), params, cell.latitude, cell.longitude),
        ),
        "/v1/open-meteo/air-quality" => (
            cell_key(&format!("air:{}", empreinte(params)), &cell),
            upstream::open_meteo_air(etat.open_meteo_key.as_deref(), params, cell.latitude, cell.longitude),
        ),
        "/v1/met-norway/compact" => (
            cell_key("met", &cell),
            upstream::met_norway_compact(cell.latitude, cell.longitude),
        ),
        "/v1/bright-sky/current" => (
            cell_key("brightsky", &cell),
            upstream::bright_sky_current(cell.latitude, cell.longitude),
        ),
        _ => return Lecture::Inconnue,
    };

    // Ce qui vieillit vite se garde moins longtemps : le quart d'heure du
    // guetteur, puis l'instant présent et la station ; la prévision horaire
    // des modèles, elle, ne bouge qu'à chaque passage des modèles.
    let cache = if params.contains_key("minutely_15") {
        &etat.quarts
    } else if params.contains_key("current") || chemin == "/v1/bright-sky/current" {
        &etat.courants
    } else {
        &etat.forecasts
    };
    Lecture::Lue(cache.serve(&cle, || (etat.fetch)(appel)).await)
}

/// L'état de santé. Ni la clé ni le code n'y apparaissent — seulement le fait
/// qu'ils soient là.
fn sante(etat: &Etat) -> String {
    format!(
        r#"{{"statut":"ok","cellules":{},"interrogations":{},"cleOpenMeteo":"{}","pro":"{}","comptes":"{}","poussee":"{}","iles":{}}}"#,
        etat.forecasts.size(),
        etat.forecasts.calls() + etat.quarts.calls() + etat.courants.calls() + etat.searches.calls(),
        if etat.open_meteo_key.is_some() { "configurée" } else { "absente" },
        etat.accord_pro.etiquette(),
        // L'état, jamais le secret.
        if etat.comptes.is_some() { "activés" } else { "désactivés" },
        // Le fait d'avoir une clé APNs, jamais la clé ni ses identifiants.
        if etat.apns.is_some() { "activée" } else { "désactivée" },
        etat.iles.taille()
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
pub(crate) fn empreinte(params: &Params) -> String {
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

/// Échange un jeton d'Apple contre une session du relais.
///
/// La réponse rend l'adresse prouvée : c'est celle de la personne qui vient de
/// se connecter, l'application la lui montre — y compris quand Apple a donné
/// une adresse relais à sa demande, qu'il faudra alors inviter telle quelle.
async fn ouvrir_session(etat: &Etat, corps: Body, cors: &HeaderMap) -> Response {
    let Some(comptes) = &etat.comptes else {
        return json(
            StatusCode::SERVICE_UNAVAILABLE,
            r#"{"erreur":"comptes désactivés"}"#.to_owned(),
            cors,
        );
    };

    let Ok(octets) = axum::body::to_bytes(corps, 16 * 1024).await else {
        return json(StatusCode::BAD_REQUEST, r#"{"erreur":"corps illisible"}"#.to_owned(), cors);
    };
    let jeton = serde_json::from_slice::<serde_json::Value>(&octets)
        .ok()
        .and_then(|v| v.get("jetonApple")?.as_str().map(str::to_owned));
    let Some(jeton) = jeton else {
        return json(StatusCode::BAD_REQUEST, r#"{"erreur":"corps illisible"}"#.to_owned(), cors);
    };

    let maintenant = (etat.now)() / 1000;
    match verifier_avec_cles(etat, comptes, &jeton, maintenant).await {
        Ok(identite) => {
            let session = identite::emettre_session(&identite, &comptes.secret, maintenant);
            let corps = serde_json::json!({ "session": session, "courriel": identite.courriel });
            json(StatusCode::OK, corps.to_string(), cors)
        }
        Err(motif) => {
            // Le motif va au journal, pas au client : détailler un refus
            // aiderait surtout qui fabrique des jetons.
            eprintln!("session refusée ({motif:?})");
            json(StatusCode::UNAUTHORIZED, r#"{"erreur":"refusé"}"#.to_owned(), cors)
        }
    }
}

/// Inscrit ou retire une île dynamique.
///
/// Inscrire : `{"jeton", "latitude", "longitude"}`. Retirer : `{"jeton"}`.
/// Retirer répond toujours 204 : un jeton inconnu est déjà retiré.
async fn activite(etat: &Etat, methode: Method, corps: Body, cors: &HeaderMap) -> Response {
    let illisible = || json(StatusCode::BAD_REQUEST, r#"{"erreur":"corps illisible"}"#.to_owned(), cors);
    let Ok(octets) = axum::body::to_bytes(corps, 4 * 1024).await else { return illisible() };
    let Ok(lu) = serde_json::from_slice::<serde_json::Value>(&octets) else { return illisible() };
    let Some(jeton) = lu.get("jeton").and_then(serde_json::Value::as_str) else { return illisible() };

    if methode == Method::DELETE {
        etat.iles.retirer(jeton);
        return (StatusCode::NO_CONTENT, cors.clone()).into_response();
    }

    // Sans clé, inutile de garder un jeton auquel on n'enverra rien : l'iPhone
    // l'apprend, et laisse l'île basculer seule.
    if etat.apns.is_none() {
        return json(
            StatusCode::SERVICE_UNAVAILABLE,
            r#"{"erreur":"poussée désactivée"}"#.to_owned(),
            cors,
        );
    }
    let nombre = |nom: &str| lu.get(nom).and_then(serde_json::Value::as_f64);
    let (Some(latitude), Some(longitude)) = (nombre("latitude"), nombre("longitude")) else {
        return illisible();
    };
    match etat.iles.inscrire(jeton, latitude, longitude, (etat.now)()) {
        Ok(()) => (StatusCode::NO_CONTENT, cors.clone()).into_response(),
        Err(iles::Refus::Plein) => {
            json(StatusCode::SERVICE_UNAVAILABLE, r#"{"erreur":"complet"}"#.to_owned(), cors)
        }
        Err(_) => illisible(),
    }
}

async fn verifier_avec_cles(
    etat: &Etat,
    comptes: &Comptes,
    jeton: &str,
    maintenant: i64,
) -> Result<Identite, Refus> {
    let (cles, relues) = cles_apple(etat, comptes, false).await.ok_or(Refus::CleInconnue)?;
    match identite::verifier_apple(jeton, &cles, &comptes.audience, maintenant) {
        // Apple fait tourner ses clés : une clé inconnue peut être une neuve.
        Err(Refus::CleInconnue) if !relues => {
            let (cles, _) = cles_apple(etat, comptes, true).await.ok_or(Refus::CleInconnue)?;
            identite::verifier_apple(jeton, &cles, &comptes.audience, maintenant)
        }
        autre => autre,
    }
}

/// Les clés d'Apple, de mémoire si elles sont fraîches. Le booléen dit si
/// elles viennent d'être relues — auquel cas une seconde lecture est inutile.
async fn cles_apple(etat: &Etat, comptes: &Comptes, forcer: bool) -> Option<(Cles, bool)> {
    let maintenant = (etat.now)();
    let memoire = comptes.cles.read().ok().and_then(|g| g.clone());
    if let Some((quand, cles)) = &memoire {
        let age = maintenant - quand;
        let fraiches = age < CLES_TTL_MS;
        let relecture_permise = age >= CLES_RELECTURE_MIN_MS;
        if fraiches && !(forcer && relecture_permise) {
            return Some((cles.clone(), false));
        }
    }

    let appel = Call { url: comptes.adresse_cles.clone(), user_agent: None };
    let Some(cles) = (etat.fetch)(appel).await.ok().and_then(|corps| Cles::lire(&corps)) else {
        // Apple injoignable : on se rabat sur ce qu'on avait, même vieux. Une
        // clé ne devient pas fausse en vieillissant ; elle peut seulement
        // manquer, et c'est ce que dira la vérification.
        return memoire.map(|(_, cles)| (cles, true));
    };
    if let Ok(mut garde) = comptes.cles.write() {
        *garde = Some((maintenant, cles.clone()));
    }
    Some((cles, true))
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
        quarts: Arc::new(ForecastCache::new(CacheOptions {
            ttl_ms: QUARTS_TTL_MS,
            stale_ms: QUARTS_STALE_MS,
            now: now.clone(),
        })),
        courants: Arc::new(ForecastCache::new(CacheOptions {
            ttl_ms: COURANTS_TTL_MS,
            stale_ms: COURANTS_STALE_MS,
            now: now.clone(),
        })),
        searches: Arc::new(ForecastCache::new(CacheOptions {
            ttl_ms: SEARCH_TTL_MS,
            stale_ms: SEARCH_TTL_MS,
            now: now.clone(),
        })),
        now,
        comptes: None,
        iles: Arc::new(Iles::new()),
        apns: None,
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

        fn repond_texte(mut self, corps: String) -> Self {
            self.reponse = Arc::new(move || Ok(corps.clone()));
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
    async fn le_direct_pousse_les_six_sujets_puis_seulement_ce_qui_change() {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message as Ws;

        let faux = Faux::new().repond(r#"{"timezone":"Europe/Paris"}"#);
        let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let adresse = ecoute.local_addr().unwrap();
        tokio::spawn(axum::serve(ecoute, router(relais(&faux))).into_future());

        let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{adresse}/v1/direct")).await.unwrap();
        let abonnement = r#"{"latitude":48.8566,"longitude":2.3522,"jours":7}"#;
        ws.send(Ws::text(abonnement)).await.unwrap();

        let mut sujets = Vec::new();
        while sujets.len() < 6 {
            match ws.next().await.unwrap().unwrap() {
                Ws::Text(texte) => {
                    let v: serde_json::Value = serde_json::from_str(&texte).unwrap();
                    assert_eq!(v["corps"]["timezone"], "Europe/Paris", "le corps passe tel quel");
                    assert_eq!(v["latitude"], 48.8566, "le point de l'abonnement revient");
                    sujets.push(v["sujet"].as_str().unwrap().to_owned());
                }
                _ => continue,
            }
        }
        assert_eq!(sujets, ["base", "ensemble", "quarts", "met", "station", "air"]);
        // Six interrogations, une par sujet : c'est tout.
        assert_eq!(faux.appels(), 6);

        // Le même abonnement, renvoyé : tout est poussé de nouveau, depuis les
        // caches — aucun fournisseur n'est réinterrogé.
        ws.send(Ws::text(abonnement)).await.unwrap();
        let mut encore = 0;
        while encore < 6 {
            if let Ws::Text(_) = ws.next().await.unwrap().unwrap() {
                encore += 1;
            }
        }
        assert_eq!(faux.appels(), 6);

        // Un abonnement absurde est ignoré, sans fermer la connexion.
        ws.send(Ws::text(r#"{"latitude":123,"longitude":0}"#)).await.unwrap();
        ws.send(Ws::text("pas du json")).await.unwrap();
        ws.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn ce_qui_vieillit_vite_se_garde_moins_longtemps() {
        let faux = Faux::new();
        let instant = Arc::new(Mutex::new(0_i64));
        let horloge = {
            let instant = instant.clone();
            Arc::new(move || *instant.lock().unwrap()) as crate::cache::Clock
        };
        let etat = etat(faux.fetch(), horloge);
        let quarts = "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48&minutely_15=precipitation";
        let courant = "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48&current=temperature_2m";
        let heures = "/v1/open-meteo/forecast?latitude=48.44&longitude=1.48&hourly=precipitation";

        let _ = get(etat.clone(), quarts).await;
        let _ = get(etat.clone(), courant).await;
        let _ = get(etat.clone(), heures).await;
        assert_eq!(faux.appels(), 3);

        // Six minutes plus tard : le quart d'heure est à refaire, le reste non.
        *instant.lock().unwrap() = 6 * 60_000;
        let (_, entetes, _) = get(etat.clone(), quarts).await;
        assert_eq!(entetes["x-klima-cache"], "frais");
        let (_, entetes, _) = get(etat.clone(), courant).await;
        assert_eq!(entetes["x-klima-cache"], "cache");
        assert_eq!(faux.appels(), 4);

        // Onze minutes : l'instant présent aussi ; l'heure, toujours pas.
        *instant.lock().unwrap() = 11 * 60_000;
        let (_, entetes, _) = get(etat.clone(), courant).await;
        assert_eq!(entetes["x-klima-cache"], "frais");
        let (_, entetes, _) = get(etat, heures).await;
        assert_eq!(entetes["x-klima-cache"], "cache");
        assert_eq!(faux.appels(), 5);
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
    async fn lair_se_relaie_a_la_maille_et_se_met_en_cache_a_part() {
        let faux = Faux::new().repond(r#"{"current":{"european_aqi":30}}"#);
        let etat = relais(&faux);
        let (status, _, corps) =
            get(etat.clone(), "/v1/open-meteo/air-quality?latitude=48.8566&longitude=2.3522&current=european_aqi").await;
        assert_eq!(status, StatusCode::OK);
        assert!(corps.contains("european_aqi"));
        assert!(faux.premiere_url().starts_with("https://air-quality-api.open-meteo.com/v1/air-quality?"));
        assert!(faux.premiere_url().contains("latitude=48.860"), "{}", faux.premiere_url());

        // La même maille, une seconde fois : le cache répond.
        get(etat.clone(), "/v1/open-meteo/air-quality?latitude=48.8567&longitude=2.3521&current=european_aqi").await;
        assert_eq!(faux.appels(), 1);
        // La prévision du même point n'est pas l'air : une entrée à part.
        get(etat, "/v1/open-meteo/forecast?latitude=48.8566&longitude=2.3522&current=european_aqi").await;
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
    async fn une_adresse_invitee_obtient_le_palier_et_les_autres_non() {
        let faux = Faux::new();
        let mut etat = relais(&faux);
        etat.accord_pro = Accord::depuis(Some("max@ferme.fr, ana@vina.es"));

        for invitee in ["max@ferme.fr", "MAX@FERME.FR", "ana@vina.es"] {
            let (status, _, corps) =
                get(etat.clone(), &format!("/v1/plan?courriel={invitee}")).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(corps, r#"{"plan":"pro"}"#, "{invitee}");
        }

        for etrangere in ["jo@farm.uk", "max@ferme.com", "max", ""] {
            let (_, _, corps) = get(etat.clone(), &format!("/v1/plan?courriel={etrangere}")).await;
            assert_eq!(corps, r#"{"plan":"libre"}"#, "{etrangere}");
        }

        // Rien présenté, rien accordé — et un code ne remplace pas une adresse.
        let (_, _, sans) = get(etat.clone(), "/v1/plan").await;
        assert_eq!(sans, r#"{"plan":"libre"}"#);
        let (_, _, par_code) = get(etat.clone(), "/v1/plan?code=max@ferme.fr").await;
        assert_eq!(par_code, r#"{"plan":"libre"}"#);
    }

    #[tokio::test]
    async fn email_marche_aussi_bien_que_courriel() {
        // Pour qu'un essai au curl ne réponde pas « libre » sur un nom de
        // paramètre : ça se cherche longtemps, et la réponse ne dit pas pourquoi.
        let faux = Faux::new();
        let mut etat = relais(&faux);
        etat.accord_pro = Accord::depuis(Some("max@ferme.fr"));

        for parametre in ["courriel", "email"] {
            let (_, _, corps) =
                get(etat.clone(), &format!("/v1/plan?{parametre}=max@ferme.fr")).await;
            assert_eq!(corps, r#"{"plan":"pro"}"#, "{parametre}");
        }
    }

    #[tokio::test]
    async fn une_adresse_a_etiquette_doit_arriver_encodee() {
        // Une query se lit en form-urlencoded : « + » y vaut une espace. Une
        // adresse « max+ferme@… » passée telle quelle arrive donc avec un trou
        // au milieu et ne correspond à rien. C'est au client d'encoder, et
        // l'application le fait (voir PlanGrantTests) ; ce test fixe la règle
        // côté relais pour qu'on ne la « corrige » pas ici par mégarde — lire
        // « + » comme un plus casserait tout formulaire conforme.
        let faux = Faux::new();
        let mut etat = relais(&faux);
        etat.accord_pro = Accord::depuis(Some("max+ferme@ferme.fr"));

        let (_, _, encodee) = get(etat.clone(), "/v1/plan?courriel=max%2Bferme@ferme.fr").await;
        assert_eq!(encodee, r#"{"plan":"pro"}"#);

        let (_, _, brute) = get(etat, "/v1/plan?courriel=max+ferme@ferme.fr").await;
        assert_eq!(brute, r#"{"plan":"libre"}"#, "un « + » brut vaut une espace");
    }

    #[tokio::test]
    async fn ni_la_reponse_ni_la_sante_ne_disent_les_adresses() {
        let faux = Faux::new();
        let mut etat = relais(&faux);
        etat.accord_pro = Accord::depuis(Some("max@ferme.fr, ana@vina.es"));

        // Un refus ne dit pas que l'adresse est inconnue : savoir cela, c'est
        // pouvoir énumérer la liste une adresse à la fois.
        let (_, _, refus) = get(etat.clone(), "/v1/plan?courriel=jo@farm.uk").await;
        assert_eq!(refus, r#"{"plan":"libre"}"#);

        let (_, _, sante) = get(etat, "/health").await;
        assert!(!sante.contains("max@"), "{sante}");
        assert!(!sante.contains("ana@"), "{sante}");
        assert!(!sante.contains("ferme"), "{sante}");
        // Le compte, en revanche, est dit : c'est ce qui rend visible du dehors
        // qu'une virgule oubliée a réduit la liste.
        assert!(sante.contains(r#""pro":"sur liste (2)""#), "{sante}");
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

    /* ---- les comptes ---- */

    use crate::identite::essai;

    /// L'instant des tests, en secondes — celui de `relais()`.
    const T: i64 = 1_700_000_000;
    const SECRET: &[u8] = b"un secret d'essai suffisamment long pour HS256";

    /// Un relais dont les comptes sont activés, et dont « Apple » répond avec
    /// les clés d'essai.
    fn avec_comptes(faux: &Faux, accord: &str) -> Etat {
        let mut etat = relais(faux);
        etat.accord_pro = Accord::depuis(Some(accord));
        etat.comptes = Some(Comptes::new(SECRET.to_vec()));
        etat
    }

    fn faux_apple() -> Faux {
        Faux::new().repond_texte(serde_json::to_string(&essai::cles()).unwrap())
    }

    async fn poste(etat: Etat, chemin: &str, corps: &str) -> (StatusCode, String) {
        let requete = Request::builder()
            .method(Method::POST)
            .uri(chemin)
            .header("Content-Type", "application/json")
            .body(Body::from(corps.to_owned()))
            .unwrap();
        let reponse = router(etat).oneshot(requete).await.expect("réponse");
        let status = reponse.status();
        let corps = to_bytes(reponse.into_body(), 1 << 20).await.unwrap();
        (status, String::from_utf8_lossy(&corps).into_owned())
    }

    async fn plan_avec_session(etat: Etat, session: &str, query: &str) -> (StatusCode, String) {
        let requete = Request::builder()
            .uri(format!("/v1/plan{query}"))
            .header("Authorization", format!("Bearer {session}"))
            .body(Body::empty())
            .unwrap();
        let reponse = router(etat).oneshot(requete).await.expect("réponse");
        let status = reponse.status();
        let corps = to_bytes(reponse.into_body(), 1 << 20).await.unwrap();
        (status, String::from_utf8_lossy(&corps).into_owned())
    }

    fn corps_session(jeton: &str) -> String {
        serde_json::json!({ "jetonApple": jeton }).to_string()
    }

    async fn ouvrir(etat: Etat, courriel: &str) -> (StatusCode, serde_json::Value) {
        let jeton = essai::jeton(essai::apple(courriel, T));
        let (status, corps) = poste(etat, "/v1/session", &corps_session(&jeton)).await;
        (status, serde_json::from_str(&corps).unwrap_or(serde_json::Value::Null))
    }

    #[tokio::test]
    async fn un_compte_invite_obtient_le_palier_sans_rien_saisir() {
        // Le trajet complet : Apple prouve l'adresse, le relais en tire une
        // session, la session suffit à obtenir le palier.
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");

        let (status, reponse) = ouvrir(etat.clone(), "Max@Ferme.FR").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(reponse["courriel"], "max@ferme.fr");
        let session = reponse["session"].as_str().expect("une session").to_owned();

        let (status, plan) = plan_avec_session(etat, &session, "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(plan, r#"{"plan":"pro"}"#);
    }

    #[tokio::test]
    async fn un_compte_non_invite_reste_au_palier_libre() {
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");

        let (_, reponse) = ouvrir(etat.clone(), "jo@farm.uk").await;
        let session = reponse["session"].as_str().unwrap().to_owned();
        let (status, plan) = plan_avec_session(etat, &session, "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(plan, r#"{"plan":"libre"}"#);
    }

    #[tokio::test]
    async fn la_session_passe_avant_ladresse_en_parametre() {
        // Un compte non invité ne se fait pas passer pour un autre en
        // ajoutant `?courriel=` : l'adresse prouvée l'emporte.
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");

        let (_, reponse) = ouvrir(etat.clone(), "jo@farm.uk").await;
        let session = reponse["session"].as_str().unwrap().to_owned();
        let (_, plan) = plan_avec_session(etat, &session, "?courriel=max@ferme.fr").await;
        assert_eq!(plan, r#"{"plan":"libre"}"#);
    }

    #[tokio::test]
    async fn retirer_quelquun_de_la_liste_lui_retire_le_palier_session_ou_pas() {
        // La session prouve une identité, elle n'accorde rien : c'est la liste
        // du moment qui décide, à chaque question.
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");
        let (_, reponse) = ouvrir(etat.clone(), "max@ferme.fr").await;
        let session = reponse["session"].as_str().unwrap().to_owned();

        let mut sans_max = etat.clone();
        sans_max.accord_pro = Accord::depuis(Some("ana@vina.es"));
        let (status, plan) = plan_avec_session(sans_max, &session, "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(plan, r#"{"plan":"libre"}"#);
    }

    #[tokio::test]
    async fn un_jeton_dapple_invalide_nouvre_pas_de_session() {
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");

        let mut revendications = essai::apple("max@ferme.fr", T);
        revendications["aud"] = serde_json::json!("com.autre.app");
        let jeton = essai::jeton(revendications);
        let (status, corps) = poste(etat, "/v1/session", &corps_session(&jeton)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        // Le refus ne dit pas pourquoi.
        assert_eq!(corps, r#"{"erreur":"refusé"}"#);
    }

    #[tokio::test]
    async fn une_session_illisible_ou_retouchee_repond_401() {
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");

        let (status, _) = plan_avec_session(etat.clone(), "nimporte.quoi.la", "").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        // Une session signée avec un autre secret : celle d'un autre relais,
        // ou d'avant un changement de secret.
        let autre = crate::identite::emettre_session(
            &Identite { sujet: "x".into(), courriel: "max@ferme.fr".into() },
            b"un autre secret, tout aussi long que le premier",
            T,
        );
        let (status, _) = plan_avec_session(etat, &autre, "").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn sans_secret_les_comptes_sont_desactives_et_le_disent() {
        let faux = faux_apple();
        let mut etat = relais(&faux);
        etat.accord_pro = Accord::depuis(Some("max@ferme.fr"));

        let jeton = essai::jeton(essai::apple("max@ferme.fr", T));
        let (status, corps) = poste(etat.clone(), "/v1/session", &corps_session(&jeton)).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(corps.contains("comptes désactivés"), "{corps}");
        // Et rien n'a été demandé à Apple.
        assert_eq!(faux.appels(), 0);

        let (_, _, sante) = get(etat, "/health").await;
        assert!(sante.contains(r#""comptes":"désactivés""#), "{sante}");
    }

    #[tokio::test]
    async fn la_sante_dit_que_les_comptes_sont_actives_sans_rien_reveler() {
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");
        let (_, _, sante) = get(etat, "/health").await;
        assert!(sante.contains(r#""comptes":"activés""#), "{sante}");
        assert!(!sante.contains("secret"), "{sante}");
    }

    #[tokio::test]
    async fn la_session_ne_se_demande_quen_post() {
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");
        let (status, _, _) = get(etat, "/v1/session").await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn un_corps_illisible_est_refuse_sans_interroger_apple() {
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");
        for corps in ["", "{}", r#"{"jeton":"x"}"#, "pas du json"] {
            let (status, _) = poste(etat.clone(), "/v1/session", corps).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{corps}");
        }
        assert_eq!(faux.appels(), 0);
    }

    #[tokio::test]
    async fn les_cles_dapple_se_lisent_une_fois_pour_plusieurs_connexions() {
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");
        for _ in 0..3 {
            let (status, _) = ouvrir(etat.clone(), "max@ferme.fr").await;
            assert_eq!(status, StatusCode::OK);
        }
        assert_eq!(faux.appels(), 1, "une seule lecture des clés pour trois connexions");
        assert_eq!(faux.premiere_url(), crate::identite::APPLE_CLES);
    }

    #[tokio::test]
    async fn une_cle_fantaisiste_ne_fait_pas_marteler_apple() {
        // Chaque jeton à clé inconnue pourrait relancer une lecture. Le frein :
        // pas plus d'une relecture toutes les cinq minutes.
        let faux = faux_apple();
        let etat = avec_comptes(&faux, "max@ferme.fr");
        ouvrir(etat.clone(), "max@ferme.fr").await; // première lecture
        for _ in 0..5 {
            let jeton = essai::jeton_avec_kid(essai::apple("max@ferme.fr", T), "fantaisie");
            let (status, _) = poste(etat.clone(), "/v1/session", &corps_session(&jeton)).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }
        assert_eq!(faux.appels(), 1, "l'horloge n'a pas bougé : aucune relecture");
    }

    #[tokio::test]
    async fn apple_injoignable_au_premier_appel_refuse_proprement() {
        let faux = Faux::new().echoue(503);
        let etat = avec_comptes(&faux, "max@ferme.fr");
        let (status, _) = ouvrir(etat, "max@ferme.fr").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
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
    async fn sans_site_la_racine_repond_404_et_non_une_erreur_de_coordonnees() {
        // Ce test disait « répond comme avant » : il gelait l'état du jour où
        // le relais a appris à servir un site, sans acter que cet état fût bon.
        // Il ne l'était pas. Le déploiement n'a pas de site — les interfaces
        // sont publiées ailleurs — donc « / » tombait sur la lecture des
        // coordonnées et répondait 400 « coordonnées manquantes » à qui ouvrait
        // simplement l'adresse. Vrai, et incompréhensible : la même raison qui
        // avait fait écrire le 404 juste au-dessus, pour le cas avec site.
        let faux = Faux::new();
        let (status, _, _) = get(relais(&faux), "/").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn sans_site_lapi_reste_entiere() {
        // Le 404 ci-dessus ne doit pas avaler ce qui est sous « /v1/ » : une
        // demande sans coordonnées doit toujours dire que ce sont elles qui
        // manquent, et non que le chemin n'existe pas.
        let faux = Faux::new();
        let (status, _, corps) = get(relais(&faux), "/v1/open-meteo/forecast").await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(corps.contains("coordonnées"), "{corps}");
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

    /* -------------------- îles dynamiques -------------------- */

    const JETON_ILE: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f90";

    fn avec_poussee(faux: &Faux) -> Etat {
        let mut etat = relais(faux);
        let (pem, _) = crate::apns::tests::cle_p8();
        etat.apns = Some(Apns::new(&pem, "ABC123DEFG", "TEAM123456", crate::apns::tests::muet()).unwrap());
        etat
    }

    async fn efface(etat: Etat, corps: &str) -> StatusCode {
        let requete = Request::builder()
            .method(Method::DELETE)
            .uri("/v1/activites")
            .body(Body::from(corps.to_owned()))
            .unwrap();
        router(etat).oneshot(requete).await.expect("réponse").status()
    }

    #[tokio::test]
    async fn une_ile_sinscrit_et_se_retire() {
        let faux = Faux::new();
        let etat = avec_poussee(&faux);
        let corps = format!(r#"{{"jeton":"{JETON_ILE}","latitude":48.45,"longitude":1.49}}"#);
        let (status, _) = poste(etat.clone(), "/v1/activites", &corps).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(etat.iles.taille(), 1);

        let (_, _, sante) = get(etat.clone(), "/health").await;
        assert!(sante.contains(r#""poussee":"activée","iles":1"#), "{sante}");

        assert_eq!(efface(etat.clone(), &format!(r#"{{"jeton":"{JETON_ILE}"}}"#)).await, StatusCode::NO_CONTENT);
        assert_eq!(etat.iles.taille(), 0);
        // Rien n'a été interrogé : l'inscription ne coûte rien au fournisseur.
        assert_eq!(faux.appels(), 0);
    }

    #[tokio::test]
    async fn sans_cle_apns_linscription_est_refusee_et_la_sante_le_dit() {
        let faux = Faux::new();
        let etat = relais(&faux);
        let corps = format!(r#"{{"jeton":"{JETON_ILE}","latitude":48.45,"longitude":1.49}}"#);
        let (status, reponse) = poste(etat.clone(), "/v1/activites", &corps).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(reponse.contains("poussée désactivée"));
        assert_eq!(etat.iles.taille(), 0);

        let (_, _, sante) = get(etat, "/health").await;
        assert!(sante.contains(r#""poussee":"désactivée","iles":0"#), "{sante}");
    }

    #[tokio::test]
    async fn une_inscription_bancale_est_refusee() {
        let faux = Faux::new();
        let etat = avec_poussee(&faux);
        for corps in [
            "pas du json".to_owned(),
            r#"{"latitude":48.45,"longitude":1.49}"#.to_owned(),
            format!(r#"{{"jeton":"{JETON_ILE}"}}"#),
            format!(r#"{{"jeton":"{JETON_ILE}","latitude":123,"longitude":1.49}}"#),
            r#"{"jeton":"pas-un-jeton","latitude":48.45,"longitude":1.49}"#.to_owned(),
        ] {
            let (status, _) = poste(etat.clone(), "/v1/activites", &corps).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{corps}");
        }
        assert_eq!(etat.iles.taille(), 0);
    }

    #[tokio::test]
    async fn la_sante_ne_dit_rien_de_la_cle_apns() {
        let faux = Faux::new();
        let (_, _, sante) = get(avec_poussee(&faux), "/health").await;
        assert!(!sante.contains("ABC123DEFG") && !sante.contains("TEAM123456"), "{sante}");
    }

    #[tokio::test]
    async fn les_iles_ne_se_lisent_pas() {
        let faux = Faux::new();
        let (status, _, _) = get(avec_poussee(&faux), "/v1/activites").await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    }
}
