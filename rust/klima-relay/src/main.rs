//! Le relais de Klima, en Axum.
//!
//! Il interroge les fournisseurs une fois pour tout le monde, pose l'en-tête
//! que MET Norway exige, détient la clé du plan commercial d'Open-Meteo et
//! sert la vitrine et l'application depuis la même origine que leurs appels.
//!
//! Ce fichier ne fait que le câblage : ce qui vient de l'environnement, et ce
//! qu'on en déduit. Les routes sont dans `routes`, les interrogations dans
//! `upstream`, les fichiers dans `site`, le cache dans `cache`.

mod cache;
mod pro;
mod routes;
mod site;
mod upstream;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8787);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("client HTTP");

    let mut etat = routes::etat(upstream::http_fetch(client), Arc::new(maintenant));
    etat.open_meteo_key = std::env::var("OPEN_METEO_KEY").ok().filter(|k| !k.is_empty());

    // Ce que ce déploiement accorde comme palier, en plus de la boutique. La
    // valeur vit ici et nulle part ailleurs : elle s'enlève en une commande,
    // là où une valeur glissée dans l'application demanderait une nouvelle
    // version pour être retirée.
    etat.accord_pro = pro::Accord::depuis(std::env::var("KLIMA_PRO").ok().as_deref());

    if let Some(origines) = std::env::var("KLIMA_ORIGINS").ok().filter(|o| !o.is_empty()) {
        etat.allowed_origins =
            origines.split(',').map(|o| o.trim().to_owned()).filter(|o| !o.is_empty()).collect();
    }

    // Le dossier est construit au déploiement. S'il n'est pas là — en
    // développement, par exemple — le relais ne fait que relayer.
    let racine = PathBuf::from(
        std::env::var("KLIMA_PUBLIC").unwrap_or_else(|_| "server/public".to_owned()),
    );
    etat.site = racine.is_dir().then_some(racine.clone());

    println!(
        "relais Klima (Rust) sur :{port} — clé Open-Meteo {}, palier accordé : {}{}",
        if etat.open_meteo_key.is_some() {
            "configurée"
        } else {
            "absente (plan gratuit, usage non commercial)"
        },
        etat.accord_pro.etiquette(),
        match &etat.site {
            Some(racine) => format!(", site servi depuis {}", racine.display()),
            None => ", sans site".to_owned(),
        }
    );

    balayer_regulierement(etat.clone());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.expect("écoute");
    axum::serve(listener, routes::router(etat)).await.expect("service");
}

/// Oublie les entrées que même le mode dépannage ne servirait plus.
///
/// Le relais TypeScript a la même méthode et ne l'appelle jamais : sa mémoire
/// ne redescend donc qu'au redémarrage. Une parcelle consultée une fois y
/// reste pour toujours — quelques kilo-octets, mais pour toujours.
fn balayer_regulierement(etat: routes::Etat) {
    tokio::spawn(async move {
        let mut horloge = tokio::time::interval(std::time::Duration::from_secs(3600));
        horloge.tick().await; // le premier top est immédiat
        loop {
            horloge.tick().await;
            etat.forecasts.sweep();
            etat.searches.sweep();
        }
    });
}

/// Millisecondes depuis l'époque.
fn maintenant() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}
