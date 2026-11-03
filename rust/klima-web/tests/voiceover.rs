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

/// Un symbole du ciel (ou d'un conseil) dit ce qu'il montre, ou se tait.
///
/// Sans libellé, VoiceOver lisait le nom anglais du symbole SF : « cloud sun
/// rain » sur le widget, la complication, l'île. Chaque `Image` dessinée
/// d'après le ciel ou un conseil porte donc `.accessibilityLabel(…)` — le
/// ciel en mots — ou `.accessibilityHidden(true)` quand le libellé s'affiche
/// déjà à côté.
#[test]
fn un_symbole_du_ciel_se_lit_en_mots() {
    let mut fichiers = Vec::new();
    swift(&racine().join("ios"), &mut fichiers);

    let mut muets = Vec::new();
    let mut vues = 0;
    for fichier in &fichiers {
        let source = fs::read_to_string(fichier).expect("source lisible");
        let lignes: Vec<&str> = source.lines().collect();
        for (i, ligne) in lignes.iter().enumerate() {
            let code = ligne.trim_start();
            if code.starts_with("//") || !code.contains("Image(systemName:") {
                continue;
            }
            if !(code.contains("symbolName") || code.contains("symbole)")) {
                continue;
            }
            vues += 1;
            // Les modificateurs qui suivent l'image (et leurs commentaires),
            // jusqu'à la vue suivante.
            let dit = lignes
                .iter()
                .skip(i + 1)
                .take(PORTEE)
                .take_while(|suite| {
                    let suite = suite.trim_start();
                    suite.starts_with('.') || suite.starts_with("//")
                })
                .any(|suite| {
                    suite.contains(".accessibilityLabel(") || suite.contains(".accessibilityHidden(")
                });
            if !dit {
                muets.push(format!("{}:{}", fichier.display(), i + 1));
            }
        }
    }
    assert!(vues >= 20, "les symboles du ciel ont disparu des vues ({vues}) : revoir ce test");
    assert_eq!(muets, Vec::<String>::new());
}

/// « Maint. » et « Auj. » se lisent en mots.
///
/// Abrégés pour tenir dans une colonne, ils étaient lus « maint point » et
/// « auj point » partout sauf dans le bandeau et la liste des jours de
/// l'iPhone : la carte de pluie, les widgets, la montre. Chaque texte qui
/// affiche l'abréviation porte, avant le texte suivant, le libellé en toutes
/// lettres (`hourly.now.a11y`, `daily.today.a11y`).
#[test]
fn une_abreviation_affichee_se_lit_en_mots() {
    let mut fichiers = Vec::new();
    swift(&racine().join("ios"), &mut fichiers);

    let mut muettes = Vec::new();
    let mut vues = 0;
    for fichier in &fichiers {
        let source = fs::read_to_string(fichier).expect("source lisible");
        let lignes: Vec<&str> = source.lines().collect();
        for (i, ligne) in lignes.iter().enumerate() {
            let code = ligne.trim_start();
            if code.starts_with("//") || !code.contains("Text(") {
                continue;
            }
            for cle in ["hourly.now", "daily.today"] {
                if !code.contains(&format!("Localized.text(\"{cle}\")")) {
                    continue;
                }
                vues += 1;
                let en_mots = format!("\"{cle}.a11y\"");
                let lue = lignes
                    .iter()
                    .skip(i + 1)
                    .take(PORTEE)
                    .take_while(|suite| !suite.contains("Text("))
                    .any(|suite| suite.contains(".accessibilityLabel(") && suite.contains(&en_mots));
                if !lue {
                    muettes.push(format!("{}:{} ({cle})", fichier.display(), i + 1));
                }
            }
        }
    }
    assert!(vues >= 7, "les abréviations ont disparu des vues ({vues}) : revoir ce test");
    assert_eq!(muettes, Vec::<String>::new());
}
