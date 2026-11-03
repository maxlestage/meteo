//! L'indice UV affiché est celui de l'heure en cours.
//!
//! La tuile « Indice UV » de la montre et des widgets montrait le maximum du
//! jour comme la valeur du moment : « 5 · modéré » à minuit. L'iPhone (#126)
//! et le web lisent l'heure en cours ; le maximum n'a sa place qu'en légende
//! (« Jusqu'à 5 aujourd'hui »).
//!
//! Comme `defilement.rs` et `voiceover.rs`, le test lit les vues SwiftUI :
//! autour de chaque intitulé `tile.uv`, rien ne tire sa valeur de
//! `uvIndexMax`, sauf une légende.

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

/// Lignes relues avant et après l'intitulé : la tuile se déclare là.
const AVANT: usize = 3;
const APRES: usize = 4;

#[test]
fn la_tuile_uv_dit_l_indice_du_moment() {
    let mut fichiers = Vec::new();
    swift(&racine().join("ios"), &mut fichiers);
    assert!(fichiers.len() > 10, "les vues SwiftUI sont introuvables");

    let mut tuiles = 0;
    let mut maxima = Vec::new();
    for fichier in &fichiers {
        let source = fs::read_to_string(fichier).expect("source lisible");
        let lignes: Vec<&str> = source.lines().collect();
        for (i, ligne) in lignes.iter().enumerate() {
            if !ligne.contains("Localized.text(\"tile.uv\")") {
                continue;
            }
            tuiles += 1;
            let debut = i.saturating_sub(AVANT);
            let fin = (i + APRES + 1).min(lignes.len());
            for (j, voisine) in lignes[debut..fin].iter().enumerate() {
                let code = voisine.trim_start();
                if code.starts_with("//") || code.starts_with("caption:") {
                    continue;
                }
                if code.contains("uvIndexMax") {
                    maxima.push(format!("{}:{}", fichier.display(), debut + j + 1));
                }
            }
        }
    }
    assert!(tuiles >= 5, "tuiles UV introuvables ({tuiles})");
    assert!(
        maxima.is_empty(),
        "le maximum du jour affiché comme indice du moment :\n{}",
        maxima.join("\n")
    );
}
