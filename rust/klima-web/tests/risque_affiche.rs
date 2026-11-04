//! Le risque de pluie s'écrit au même seuil partout.
//!
//! Chaque écran avait le sien : 10 % sur l'iPhone et le web, 20 % sur la
//! montre et l'île dynamique, 30 % dans les widgets. Un « 25 % » lu dans
//! l'application manquait au widget de la même heure. Le seuil vit
//! maintenant dans `ville::seuils::RISQUE_AFFICHE` et son miroir
//! `VilleSeuils.risqueAffiche` ; les vues passent par `ville::risque_affiche`
//! et `Ville.risqueAffiche`.
//!
//! Comme `bleu_pluie.rs`, le test lit les vues SwiftUI — et celles du web.

use std::fs;
use std::path::{Path, PathBuf};

fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sources(dossier: &Path, extension: &str, fichiers: &mut Vec<PathBuf>) {
    for entree in fs::read_dir(dossier).unwrap_or_else(|e| panic!("{} : {e}", dossier.display())) {
        let chemin = entree.expect("entrée lisible").path();
        if chemin.is_dir() {
            sources(&chemin, extension, fichiers);
        } else if chemin.extension().is_some_and(|e| e == extension) {
            fichiers.push(chemin);
        }
    }
}

/// Vrai si la ligne compare un risque de pluie à un nombre écrit en dur.
fn seuil_en_dur(ligne: &str) -> bool {
    ["precipitationProbability", "precipitation_probability"].iter().any(|champ| {
        ligne.match_indices(champ).any(|(i, _)| {
            let suite = ligne[i + champ.len()..].trim_start_matches("Max").trim_start_matches("_max").trim_start();
            ["<=", ">=", "<", ">"].iter().any(|op| {
                suite
                    .strip_prefix(op)
                    .is_some_and(|reste| reste.trim_start().starts_with(|c: char| c.is_ascii_digit()))
            })
        })
    })
}

#[test]
fn le_detecteur_voit_un_seuil_en_dur() {
    assert!(seuil_en_dur("Text(hour.precipitationProbability >= 30 ? a : b)"));
    assert!(seuil_en_dur(".accessibilityHidden(jour.precipitationProbabilityMax < 30)"));
    assert!(seuil_en_dur("if day.precipitation_probability_max >= 10.0 {"));
    assert!(!seuil_en_dur("if ville::risque_affiche(hour.precipitation_probability) {"));
    assert!(!seuil_en_dur("heure.precipitationProbability >= VilleSeuils.pluieProbabilite"));
}

#[test]
fn le_risque_s_ecrit_au_meme_seuil_partout() {
    let mut fichiers = Vec::new();
    for dossier in ["ios/Kliima/Views", "ios/KliimaWidgets", "ios/KliimaWatch", "ios/KliimaWatchWidgets"] {
        sources(&racine().join(dossier), "swift", &mut fichiers);
    }
    sources(&racine().join("rust/klima-web/src"), "rs", &mut fichiers);
    assert!(fichiers.len() > 20, "les vues sont introuvables");

    let mut fixes = Vec::new();
    for fichier in &fichiers {
        let source = fs::read_to_string(fichier).expect("source lisible");
        for (i, ligne) in source.lines().enumerate() {
            if seuil_en_dur(ligne) {
                fixes.push(format!("{}:{}", fichier.display(), i + 1));
            }
        }
    }
    assert!(
        fixes.is_empty(),
        "risque de pluie comparé à un seuil en dur, à remplacer par `Ville.risqueAffiche` / `ville::risque_affiche` :\n{}",
        fixes.join("\n")
    );
}
