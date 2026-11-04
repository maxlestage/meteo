//! Le bleu de la pluie suit le thème.
//!
//! Sur l'iPhone, le risque de pluie du bandeau horaire et de la liste des
//! jours était écrit d'un azur pâle fixe (`0.498, 0.816, 0.961`) : lisible
//! sur le ciel sombre, il s'effaçait sur le ciel clair, à peine plus foncé
//! que le fond. `Color.bleuPluie` (`ios/Kliima/Views/Animations.swift`) prend
//! cet azur en sombre et un bleu soutenu en clair. Hors de sa définition,
//! l'azur pâle n'a donc rien à faire dans les vues de l'iPhone et de ses
//! widgets qui suivent le thème. La montre, toujours sombre, garde le sien.
//!
//! Comme `rafales.rs`, le test lit les vues SwiftUI.

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
fn l_azur_pale_ne_s_ecrit_pas_sur_le_ciel_clair() {
    let mut fichiers = Vec::new();
    swift(&racine().join("ios/Kliima/Views"), &mut fichiers);
    assert!(fichiers.len() > 10, "les vues SwiftUI sont introuvables");

    let definition = fs::read_to_string(racine().join("ios/Kliima/Views/Animations.swift")).expect("source lisible");
    assert!(definition.contains("static let bleuPluie"), "`Color.bleuPluie` a disparu");

    let mut fixes = Vec::new();
    for fichier in &fichiers {
        let source = fs::read_to_string(fichier).expect("source lisible");
        for (i, ligne) in source.lines().enumerate() {
            let azur = ligne.contains("0.498") && ligne.contains("0.816") && ligne.contains("0.961");
            if azur && ligne.contains("Color(red:") && !ligne.contains("UIColor(red:") {
                fixes.push(format!("{}:{}", fichier.display(), i + 1));
            }
        }
    }
    assert!(fixes.is_empty(), "azur pâle fixe, à remplacer par `Color.bleuPluie` :\n{}", fixes.join("\n"));
}
