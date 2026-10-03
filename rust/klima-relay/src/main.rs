//! Le relais de Klima, en Axum.
//!
//! Il interroge les fournisseurs une fois pour tout le monde, pose l'en-tête
//! que MET Norway exige, et détient la clé du plan commercial d'Open-Meteo.
//!
//! Le relais Bun (`server/`) sert toujours la production : celui-ci se
//! construit à côté, route par route, et on ne débranche qu'une fois le
//! remplaçant complet. Pour l'instant il sait dire comment il va — ce qui
//! suffit à prouver la chaîne : Axum répond, et le cache qu'il porte est le
//! même que celui du relais TypeScript.

mod cache;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::response::IntoResponse;
use cache::{CacheOptions, ForecastCache};

/// Une entrée vit une heure, et dépanne encore deux heures si le fournisseur
/// se tait. Les mêmes durées que dans `server/src/index.ts`.
const TTL_MS: i64 = 3_600_000;
const STALE_MS: i64 = 7_200_000;

#[derive(Clone)]
struct Etat {
    forecasts: Arc<ForecastCache<String>>,
    open_meteo_key: Option<String>,
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8787);

    let etat = Etat {
        forecasts: Arc::new(ForecastCache::new(CacheOptions {
            ttl_ms: TTL_MS,
            stale_ms: STALE_MS,
            now: Arc::new(maintenant),
        })),
        open_meteo_key: std::env::var("OPEN_METEO_KEY").ok().filter(|k| !k.is_empty()),
    };

    let app = axum::Router::new()
        .route("/health", axum::routing::get(sante))
        .with_state(etat.clone());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.expect("écoute");
    println!(
        "relais Klima (Rust) sur :{port} — clé Open-Meteo {}",
        if etat.open_meteo_key.is_some() {
            "configurée"
        } else {
            "absente (plan gratuit, usage non commercial)"
        }
    );
    axum::serve(listener, app).await.expect("service");
}

/// Millisecondes depuis l'époque.
fn maintenant() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

/// L'état de santé, dans la même forme que celui du relais TypeScript — la
/// clé n'y apparaît jamais, seulement le fait qu'elle soit là.
async fn sante(State(etat): State<Etat>) -> impl IntoResponse {
    let corps = format!(
        r#"{{"statut":"ok","cellules":{},"interrogations":{},"cleOpenMeteo":"{}"}}"#,
        etat.forecasts.size(),
        etat.forecasts.calls(),
        if etat.open_meteo_key.is_some() { "configurée" } else { "absente" }
    );
    ([(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")], corps)
}
