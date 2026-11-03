//! Les rafales se disent en mots.
//!
//! Sur la montre, la tuile « Vent » écrivait les rafales « ↗ 45 km/h » : la
//! flèche se lisait comme un vent de nord-est, et VoiceOver disait « flèche
//! nord-est ». Les widgets les posaient à nu sous la vitesse (« 45 km/h »), ou
//! les accolaient (« 20 km/h · 45 km/h ») : deux vitesses, sans dire laquelle
//! est la rafale. L'iPhone dit déjà « Rafales 45 km/h. ».
//!
//! Comme `uv.rs`, le test lit les vues SwiftUI : toute vue qui affiche
//! `current.windGusts` le fait à travers un libellé `tile.wind.*`, sans flèche.

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

/// Lignes relues avant l'affichage : le libellé s'ouvre là.
const AVANT: usize = 2;

#[test]
fn les_rafales_portent_leur_nom() {
    let mut fichiers = Vec::new();
    swift(&racine().join("ios"), &mut fichiers);
    assert!(fichiers.len() > 10, "les vues SwiftUI sont introuvables");

    let mut affichages = 0;
    let mut nues = Vec::new();
    for fichier in &fichiers {
        let source = fs::read_to_string(fichier).expect("source lisible");
        let lignes: Vec<&str> = source.lines().collect();
        for (i, ligne) in lignes.iter().enumerate() {
            if !ligne.contains("AgroFormat.unit(current.windGusts") {
                continue;
            }
            affichages += 1;
            let nommee = lignes[i.saturating_sub(AVANT)..=i]
                .iter()
                .any(|l| l.contains("\"tile.wind."));
            if !nommee || ligne.contains('↗') {
                nues.push(format!("{}:{}", fichier.display(), i + 1));
            }
        }
    }

    assert!(affichages >= 4, "iPhone, montre et widgets affichent les rafales ({affichages} trouvés)");
    assert!(nues.is_empty(), "rafales sans leur nom :\n{}", nues.join("\n"));
}
