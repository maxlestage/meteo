//! Le relais de Klima, en Axum.
//!
//! Il interroge les fournisseurs une fois pour tout le monde, pose l'en-tête
//! que MET Norway exige, détient la clé du plan commercial d'Open-Meteo et
//! sert la vitrine et l'application depuis la même origine que leurs appels.
//!
//! Ce fichier ne fait que le câblage : ce qui vient de l'environnement, et ce
//! qu'on en déduit. Les routes sont dans `routes`, les interrogations dans
//! `upstream`, les fichiers dans `site`, le cache dans `cache`.

mod apns;
mod cache;
mod direct;
mod identite;
mod iles;
mod poussee;
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

    // Les comptes. Sans secret, ils sont désactivés : mieux vaut un relais qui
    // le dit qu'un secret tiré au hasard à chaque démarrage, qui déconnecterait
    // tout le monde au premier recyclage du dyno — chaque jour sur Heroku.
    etat.comptes = comptes_depuis_l_environnement();

    // La poussée vers les îles dynamiques. Sans clé, elles basculent quand même
    // seules à l'heure pile — elles ne reçoivent simplement rien de neuf.
    etat.apns = apns_depuis_l_environnement();

    if let Some(origines) = std::env::var("KLIMA_ORIGINS").ok().filter(|o| !o.is_empty()) {
        etat.allowed_origins =
            origines.split(',').map(|o| o.trim().to_owned()).filter(|o| !o.is_empty()).collect();
    }

    // Le dossier est construit au déploiement. S'il n'est pas là — en
    // développement, par exemple — le relais ne fait que relayer.
    let racine = PathBuf::from(
        std::env::var("KLIMA_PUBLIC").unwrap_or_else(|_| "public".to_owned()),
    );
    etat.site = racine.is_dir().then_some(racine.clone());

    println!(
        "relais Klima (Rust) sur :{port} — clé Open-Meteo {}, palier accordé : {}, comptes {}, poussée {}{}",
        if etat.open_meteo_key.is_some() {
            "configurée"
        } else {
            "absente (plan gratuit, usage non commercial)"
        },
        etat.accord_pro.etiquette(),
        if etat.comptes.is_some() { "activés" } else { "désactivés" },
        if etat.apns.is_some() { "activée" } else { "désactivée" },
        match &etat.site {
            Some(racine) => format!(", site servi depuis {}", racine.display()),
            None => ", sans site".to_owned(),
        }
    );

    balayer_regulierement(etat.clone());
    poussee::tourner(etat.clone());

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

/// Le secret des sessions doit être long : 32 octets au moins, ce que donne
/// `openssl rand -hex 32` (64 caractères). Plus court, HS256 se force.
const SECRET_MIN: usize = 32;

/// Ce qu'il faut pour reconnaître les comptes, d'après l'environnement.
fn comptes_depuis_l_environnement() -> Option<routes::Comptes> {
    let secret = std::env::var("KLIMA_SESSION_SECRET").ok()?;
    let secret = secret.trim();
    if secret.len() < SECRET_MIN {
        // Dit au journal, jamais la valeur : seulement qu'elle est trop courte.
        eprintln!(
            "KLIMA_SESSION_SECRET fait {} octets, il en faut {SECRET_MIN} : comptes désactivés",
            secret.len()
        );
        return None;
    }

    let mut comptes = routes::Comptes::new(secret.as_bytes().to_vec());
    // Pour une autre application, ou pour pointer les clés vers un faux Apple
    // dans un essai. Rien à régler en temps normal.
    if let Some(audience) = std::env::var("KLIMA_APPLE_AUDIENCE").ok().filter(|v| !v.is_empty()) {
        comptes.audience = audience;
    }
    if let Some(adresse) = std::env::var("KLIMA_APPLE_KEYS").ok().filter(|v| !v.is_empty()) {
        comptes.adresse_cles = adresse;
    }
    Some(comptes)
}

/// La clé APNs, d'après l'environnement : les trois valeurs ou rien.
///
/// `KLIMA_APNS_KEY` est le contenu du fichier `.p8` qu'Apple délivre une
/// fois, `KLIMA_APNS_KEY_ID` son identifiant à dix caractères,
/// `KLIMA_APNS_TEAM_ID` celui de l'équipe. Une valeur bancale désactive la
/// poussée et le journal dit laquelle — jamais ce qu'elle contient.
fn apns_depuis_l_environnement() -> Option<apns::Apns> {
    let lire = |nom: &str| std::env::var(nom).ok().filter(|v| !v.trim().is_empty());
    let (cle, id_cle, equipe) = match (
        lire("KLIMA_APNS_KEY"),
        lire("KLIMA_APNS_KEY_ID"),
        lire("KLIMA_APNS_TEAM_ID"),
    ) {
        (None, None, None) => return None,
        (Some(cle), Some(id), Some(equipe)) => (cle, id, equipe),
        _ => {
            eprintln!("poussée désactivée : il faut KLIMA_APNS_KEY, KLIMA_APNS_KEY_ID et KLIMA_APNS_TEAM_ID ensemble");
            return None;
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("client HTTP");
    match apns::Apns::new(&cle, &id_cle, &equipe, apns::http_envoyer(client)) {
        Ok(mut apns) => {
            // Le bac à sable, pour une compilation de développement.
            if let Some(hote) = lire("KLIMA_APNS_HOST") {
                apns.hote = hote;
            }
            Some(apns)
        }
        Err(motif) => {
            eprintln!("poussée désactivée : {motif:?} illisible");
            None
        }
    }
}

/// Millisecondes depuis l'époque.
fn maintenant() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}
