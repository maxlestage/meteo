//! Ce que le déploiement promet, et que rien n'exécutait.
//!
//! Le relais a tourné sans que personne ne vérifie que la chaîne de
//! construction d'Heroku disait vrai. Elle ne disait pas vrai : le `RustConfig`
//! portait deux clés que le buildpack ne lit pas, une troisième dont la valeur
//! faisait *sauter* la construction, et un `BUILD_PATH` qui posait le binaire
//! un dossier à côté de celui que le `Procfile` lançait. Rien ne tombait,
//! puisque rien ne regardait.
//!
//! Ces tests regardent. Ils ne remplacent pas un déploiement — ils attrapent la
//! catégorie d'erreur qui ne se voit qu'en production, et qui ne dit alors rien
//! de plus qu'un 503.
//!
//! La référence est `bin/compile` du buildpack `emk/heroku-buildpack-rust`, qui
//! déclare ses variables en tête de script et n'en lit aucune autre.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Les seules clés que `bin/compile` déclare. Tout le reste est décoratif.
const CLES_RECONNUES: &[&str] = &[
    "VERSION",
    "RUST_SKIP_BUILD",
    "BUILD_PATH",
    "RUST_CARGO_BUILD_FLAGS",
    "RUST_INSTALL_DIESEL",
    "DIESEL_FLAGS",
    "CARGO_URL",
    "CARGO_TARGET_DIR",
];

fn racine() -> PathBuf {
    // `CARGO_MANIFEST_DIR` vaut `…/rust/klima-relay`.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn lire(nom: &str) -> String {
    let chemin = racine().join(nom);
    std::fs::read_to_string(&chemin)
        .unwrap_or_else(|e| panic!("{} illisible : {e}", chemin.display()))
}

/// `RustConfig` est un fragment de shell sourcé : `CLE=valeur`, un par ligne.
fn rust_config() -> BTreeMap<String, String> {
    lire("RustConfig")
        .lines()
        .map(str::trim)
        .filter(|ligne| !ligne.is_empty() && !ligne.starts_with('#'))
        .filter_map(|ligne| ligne.split_once('='))
        .map(|(cle, valeur)| (cle.trim().to_owned(), valeur.trim().trim_matches('"').to_owned()))
        .collect()
}

#[test]
fn chaque_cle_du_rust_config_en_est_vraiment_une() {
    for cle in rust_config().keys() {
        assert!(
            CLES_RECONNUES.contains(&cle.as_str()),
            "« {cle} » n'est pas une variable que le buildpack lit : elle ne fera rien, \
             et ne le dira pas. Clés reconnues : {CLES_RECONNUES:?}"
        );
    }
}

#[test]
fn rust_skip_build_sil_est_la_est_un_nombre() {
    // Le buildpack écrit `[ $RUST_SKIP_BUILD -ne 1 ]`, qui attend un entier.
    // Avec « no », le test échoue, la branche n'est pas prise, et la
    // construction est sautée sans que rien ne signale l'erreur.
    if let Some(valeur) = rust_config().get("RUST_SKIP_BUILD") {
        assert!(
            valeur == "0" || valeur == "1",
            "RUST_SKIP_BUILD vaut « {valeur} » ; le buildpack le compare avec « -ne », \
             qui attend 0 ou 1. Toute autre valeur fait sauter la construction."
        );
    }
}

#[test]
fn la_version_du_rust_config_est_celle_du_rust_toolchain() {
    // Le buildpack ne lit que l'ancien `rust-toolchain` en texte brut : il
    // ignore le `.toml`. La version doit donc être répétée, et concorder.
    let config = rust_config();
    let declaree = config.get("VERSION").expect("RustConfig doit dire VERSION");

    let toolchain = lire("rust-toolchain.toml");
    let channel = toolchain
        .lines()
        .find_map(|ligne| ligne.trim().strip_prefix("channel"))
        .and_then(|reste| reste.split('=').nth(1))
        .map(|v| v.trim().trim_matches('"').to_owned())
        .expect("rust-toolchain.toml doit dire channel");

    assert_eq!(
        declaree, &channel,
        "RustConfig compile sur {declaree} et rust-toolchain.toml sur {channel}"
    );
}

#[test]
fn le_procfile_lance_le_binaire_la_ou_le_buildpack_le_pose() {
    // C'est le test qui aurait attrapé la panne : le buildpack se place dans
    // `BUILD_PATH`, construit, puis recopie les exécutables dans
    // `target/release` relatif à ce dossier. Le `Procfile` doit viser là.
    let config = rust_config();
    let build_path = config.get("BUILD_PATH").map(String::as_str).unwrap_or("");
    let attendu = if build_path.is_empty() {
        "target/release/klima-relay".to_owned()
    } else {
        format!("{build_path}/target/release/klima-relay")
    };

    let procfile = lire("Procfile");
    let commande = procfile
        .lines()
        .find_map(|ligne| ligne.trim().strip_prefix("web:"))
        .map(str::trim)
        .expect("le Procfile doit déclarer un processus web");

    assert_eq!(
        commande, attendu,
        "le Procfile lance « {commande} » alors que le buildpack pose le binaire \
         dans « {attendu} » (BUILD_PATH={build_path})"
    );
}

#[test]
fn le_binaire_vise_par_le_procfile_est_bien_celui_de_cette_caisse() {
    // Un `Procfile` qui lancerait un autre nom passerait les tests précédents
    // et échouerait au démarrage. Le nom vient du `Cargo.toml`.
    let procfile = lire("Procfile");
    assert!(
        procfile.contains(env!("CARGO_PKG_NAME")),
        "le Procfile ne nomme pas {} :\n{procfile}",
        env!("CARGO_PKG_NAME")
    );
}
