//! Construire le site en même temps que le relais — chez Heroku seulement.
//!
//! Le buildpack Rust d'Heroku fait une chose : `cargo build --release`. Il n'a
//! aucun crochet pour lancer autre chose avant ou après, et la vitrine comme
//! l'application web demandent Trunk et une compilation en WebAssembly. Sans
//! ce fichier, le relais déployé ne servait que l'API, et l'adresse publique
//! répondait 404 à qui l'ouvrait.
//!
//! Ce script de construction comble ce trou, et seulement celui-là :
//!
//! - **Il ne fait rien ailleurs.** Ni en développement, ni en CI, ni sous
//!   `cargo test` : il ne s'éveille que si `STACK` dit `heroku-…` (Heroku la
//!   pose pendant la construction) ou si `KLIMA_CONSTRUIRE_SITE=1` le demande
//!   explicitement, pour l'essayer à la main.
//! - **Il appelle le script qui existe déjà**, `rust/scripts/construire.sh`,
//!   celui de GitHub Pages et du développement. Une seule façon de construire
//!   le site.
//! - **Il ne verrouille pas le cargo qui l'appelle.** Le buildpack fixe
//!   `CARGO_TARGET_DIR` pour toute la construction ; le cargo imbriqué en
//!   reçoit un autre, voisin, dans le même cache — sans quoi il attendrait
//!   pour toujours le verrou que tient son parent.
//! - **Il échoue franchement.** Un site qui ne se construit pas fait échouer le
//!   déploiement, et Heroku garde la version en service. Remplacer une version
//!   qui servait le site par une qui ne le sert plus serait pire.

use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    println!("cargo:rerun-if-env-changed=STACK");
    println!("cargo:rerun-if-env-changed=KLIMA_CONSTRUIRE_SITE");
    // Chaque déploiement porte un nouveau SOURCE_VERSION : le site est refait à
    // chaque fois, même si rien n'a bougé dans le relais. Sans cela, cargo
    // garderait le résultat du script en cache, et un dépôt neuf partirait
    // sans dossier `public`.
    println!("cargo:rerun-if-env-changed=SOURCE_VERSION");

    let sur_heroku = env::var("STACK").is_ok_and(|s| s.starts_with("heroku-"));
    let demande = env::var("KLIMA_CONSTRUIRE_SITE").is_ok_and(|v| v == "1");
    if !(sur_heroku || demande) {
        return;
    }

    let manifeste = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let espace = manifeste.parent().expect("rust/");
    let racine = espace.parent().expect("racine du dépôt");
    let script = espace.join("scripts").join("construire.sh");
    // Là où le relais le cherchera : `KLIMA_PUBLIC` vaut `public` par défaut,
    // relatif au dossier de lancement, qui est la racine chez Heroku.
    let dehors = racine.join("public");

    // Le cache du site, à côté de celui du relais — même durée de vie, autre
    // verrou. Hors Heroku, dans OUT_DIR, que cargo nettoie avec le reste.
    let cache = match env::var("CARGO_TARGET_DIR") {
        Ok(cible) => PathBuf::from(cible).with_file_name("klima-site-cache"),
        Err(_) => PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("site"),
    };

    eprintln!("construction du site dans {} (cache {})", dehors.display(), cache.display());

    let statut = Command::new("bash")
        .arg(&script)
        .arg(&dehors)
        .env("CARGO_TARGET_DIR", cache.join("target"))
        .env("KLIMA_OUTILS", cache.join("outils"))
        // Ce que cargo passe à un script de construction et qu'un cargo
        // imbriqué lirait comme un réglage : des drapeaux vides qui écraseraient
        // ceux du WebAssembly, un compilateur imposé au lieu de celui de
        // `rust-toolchain.toml`.
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("CARGO_BUILD_TARGET")
        // La sortie standard d'un script de construction est lue par cargo,
        // ligne à ligne, comme des directives. Celle de Trunk va sur l'erreur
        // standard, que cargo montre si quelque chose casse.
        .stdout(Stdio::from(std::io::stderr()))
        .status()
        .expect("lancer construire.sh");

    assert!(
        statut.success(),
        "la construction du site a échoué ({statut}) : le déploiement s'arrête, \
         et Heroku garde la version en service"
    );
    assert!(
        dehors.join("index.html").is_file() && dehors.join("app").join("index.html").is_file(),
        "construire.sh a fini sans produire public/index.html et public/app/index.html"
    );
}
