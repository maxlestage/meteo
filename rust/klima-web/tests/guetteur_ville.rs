//! Le guetteur ne parle que de la ville affichée.
//!
//! La montre se cale sur la position du poignet. Arrivé à Lyon depuis Paris,
//! une lecture ratée gardait la série de Paris : le poignet disait « sec »
//! sous le nom de Lyon, d'après le ciel d'une autre ville. L'iPhone s'en
//! gardait déjà (`quartsDe`) ; la montre aussi désormais.
//!
//! Comme `uv.rs`, le test lit les vues SwiftUI : toute vue du guetteur reçoit
//! `quartsDeLaVille`, jamais la série brute.

use std::fs;
use std::path::{Path, PathBuf};

fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn swift(dossier: &Path, fichiers: &mut Vec<PathBuf>) {
    for entree in fs::read_dir(dossier).unwrap_or_else(|e| panic!("{} : {e}", dossier.display())) {
        let chemin = entree.expect("entrée lisible").path();
        if chemin.is_dir() {
            swift(&chemin, fichiers);
        } else if chemin.extension().is_some_and(|e| e == "swift") {
            fichiers.push(chemin);
        }
    }
}

#[test]
fn le_guetteur_ne_montre_que_la_serie_de_la_ville() {
    let mut fichiers = Vec::new();
    swift(&racine().join("ios"), &mut fichiers);
    assert!(fichiers.len() > 10, "les vues SwiftUI sont introuvables");

    let mut gardes = 0;
    let mut bruts = Vec::new();
    for fichier in &fichiers {
        if fichier.components().any(|c| c.as_os_str() == "KliimaTests") {
            continue;
        }
        let source = fs::read_to_string(fichier).expect("source lisible");
        for (i, ligne) in source.lines().enumerate() {
            let code = ligne.trim_start();
            if code.starts_with("//") {
                continue;
            }
            if code.contains("quarts: viewModel.quartsDeLaVille") {
                gardes += 1;
            } else if code.contains("quarts: viewModel.") {
                bruts.push(format!("{}:{}", fichier.display(), i + 1));
            }
        }
    }
    assert!(
        bruts.is_empty(),
        "un guetteur reçoit une série qui peut être d'une autre ville :\n{}",
        bruts.join("\n")
    );
    // L'iPhone et la montre.
    assert!(gardes >= 2, "guetteurs introuvables ({gardes})");
}
