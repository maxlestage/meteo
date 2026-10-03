//! Le relais de Klima, en Axum.
//!
//! Il interroge les fournisseurs une fois pour tout le monde, pose l'en-tête
//! que MET Norway exige, et détient la clé du plan commercial d'Open-Meteo.
//! Le relais Bun (`server/`) sert toujours la production : celui-ci se
//! construit à côté, et on ne débranche qu'une fois le remplaçant prouvé.

mod cache;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8787);
    let app = axum::Router::new().route("/health", axum::routing::get(sante));
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.expect("écoute");
    println!("relais Klima (Rust) sur :{port}");
    axum::serve(listener, app).await.expect("service");
}

async fn sante() -> &'static str {
    "ok"
}
