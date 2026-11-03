//! Une alerte dit de quelle ville elle parle.
//!
//! « Gel annoncé — jusqu'à −2 °C » arrivait sans dire où. Au palier qui ouvre
//! les alertes, on suit plusieurs villes, et la notification tombe souvent
//! longtemps après qu'on a quitté l'écran de celle-ci. Le nom de la ville va
//! en sous-titre, et chaque ville a son fil dans le centre de notifications.
//!
//! Comme `defilement.rs`, `voiceover.rs` et `uv.rs`, le test lit le Swift :
//! la notification porte un sous-titre et un fil, et chaque appel au
//! planificateur nomme la ville.

use std::fs;
use std::path::{Path, PathBuf};

fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn lire(relatif: &str) -> String {
    let chemin = racine().join(relatif);
    fs::read_to_string(&chemin).unwrap_or_else(|e| panic!("{} : {e}", chemin.display()))
}

#[test]
fn la_notification_porte_le_nom_de_la_ville() {
    let source = lire("ios/Kliima/Services/AlertScheduler.swift");
    assert!(
        source.contains("content.subtitle = ville"),
        "la notification n'a plus la ville en sous-titre"
    );
    assert!(
        source.contains("content.threadIdentifier ="),
        "les alertes de deux villes ne sont plus rangées à part"
    );
}

#[test]
fn chaque_appel_nomme_la_ville() {
    let mut appels = 0;
    for fichier in [
        "ios/Kliima/Views/DashboardView.swift",
        "ios/Kliima/Services/BackgroundRefresh.swift",
    ] {
        let source = lire(fichier);
        for (i, _) in source.match_indices("AlertScheduler.schedule(") {
            appels += 1;
            let suite: String = source[i..].chars().take(200).collect();
            assert!(
                suite.contains("ville: forecast.parcelle.name"),
                "{fichier} : un appel au planificateur ne nomme pas la ville"
            );
        }
    }
    assert_eq!(appels, 2, "les appels au planificateur ont changé de place");
}
