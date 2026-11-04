//! « Maintenant » se relit à chaque affichage, partout.
//!
//! Le relevé d'une prévision (`forecast.current`) est celui de sa réception :
//! servie par le cache faute de réseau, ou gardée ouverte, elle montrait à
//! 17 h la mesure de 13 h 45. Les widgets, la complication et l'île passent
//! par `Horizon.conditions` (#131), l'en-tête et la montre par
//! `AgroForecast.maintenant()` ; la liste des villes lisait encore le relevé
//! brut.
//!
//! Comme `uv.rs` et `defilement.rs`, le test lit le Swift : hors des modèles et
//! des services, un relevé ne se lit que pour être confié à `Horizon` (les
//! arguments `courant:` et `releve:`).

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

/// Un relevé tiré d'une prévision : `forecast.current`, `prevision.current`…
fn lit_le_releve(code: &str) -> bool {
    ["forecast.current", "prevision.current", "base.current"].iter().any(|motif| {
        code.match_indices(motif).any(|(i, _)| {
            let suite = code[i + motif.len()..].chars().next();
            !suite.is_some_and(|c| c.is_alphanumeric() || c == '_')
        })
    })
}

#[test]
fn maintenant_se_relit_a_chaque_affichage() {
    let ios = racine().join("ios");
    let mut fichiers = Vec::new();
    swift(&ios, &mut fichiers);
    assert!(fichiers.len() > 10, "les vues SwiftUI sont introuvables");

    let mut bruts = Vec::new();
    for fichier in &fichiers {
        let relatif = fichier.strip_prefix(&ios).unwrap_or(fichier);
        let dossiers: Vec<_> = relatif.components().map(|c| c.as_os_str().to_string_lossy()).collect();
        // Les modèles font la règle, les services la prévision, les tests la
        // vérifient : le relevé y est chez lui.
        if dossiers.iter().any(|d| d == "Models" || d == "Services" || d == "KliimaTests") {
            continue;
        }
        let source = fs::read_to_string(fichier).expect("source lisible");
        for (i, ligne) in source.lines().enumerate() {
            let code = ligne.trim_start();
            if code.starts_with("//") || !lit_le_releve(code) {
                continue;
            }
            if code.starts_with("courant:") || code.contains("releve:") {
                continue;
            }
            bruts.push(format!("{}:{}", fichier.display(), i + 1));
        }
    }
    assert!(
        bruts.is_empty(),
        "un relevé affiché sans passer par `maintenant()` ni `Horizon` :\n{}",
        bruts.join("\n")
    );
}

#[test]
fn le_motif_reconnait_un_releve_et_lui_seul() {
    assert!(lit_le_releve("apercus[ville.id] = prevision.current"));
    assert!(lit_le_releve("Text(forecast.current.temperature)"));
    assert!(!lit_le_releve("let present = forecast.currentHour"));
    assert!(!lit_le_releve("forecast.maintenant()"));
}
