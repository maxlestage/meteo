//! Ce que VoiceOver lit à la place des flèches.
//!
//! L'amplitude du jour s'affiche « ↑ 13°  ↓ 9° » sur l'iPhone, la montre, les
//! widgets, l'île et la complication. Sans libellé, VoiceOver lisait « flèche
//! vers le haut 13 degrés » : chaque texte qui porte une flèche dit en mots ce
//! qu'il montre (`Localized.amplitude`).
//!
//! Comme `defilement.rs`, le test lit les vues SwiftUI : la règle tient en une
//! déclaration qu'on peut relire, sans simulateur dans l'intégration continue.

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

/// Le modificateur doit suivre le texte, avant le texte suivant.
const PORTEE: usize = 8;

#[test]
fn une_fleche_affichee_se_lit_en_mots() {
    let mut fichiers = Vec::new();
    swift(&racine().join("ios"), &mut fichiers);
    assert!(fichiers.len() > 10, "les vues SwiftUI sont introuvables");

    let mut muettes = Vec::new();
    let mut vues = 0;
    for fichier in &fichiers {
        let source = fs::read_to_string(fichier).expect("source lisible");
        let lignes: Vec<&str> = source.lines().collect();
        for (i, ligne) in lignes.iter().enumerate() {
            let code = ligne.trim_start();
            if code.starts_with("//") || !code.contains("Text(\"") {
                continue;
            }
            if !(ligne.contains('↑') || ligne.contains('↓')) {
                continue;
            }
            vues += 1;
            let lue = lignes
                .iter()
                .skip(i + 1)
                .take(PORTEE)
                .take_while(|suite| !suite.contains("Text("))
                .any(|suite| suite.contains(".accessibilityLabel("));
            if !lue {
                muettes.push(format!("{}:{}", fichier.display(), i + 1));
            }
        }
    }
    assert!(vues >= 5, "l'amplitude fléchée a disparu des vues ({vues}) : revoir ce test");
    assert_eq!(muettes, Vec::<String>::new());
}
