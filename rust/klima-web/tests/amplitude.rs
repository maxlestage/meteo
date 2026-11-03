//! Le maximum du jour n'est jamais sous la température affichée.
//!
//! L'en-tête de l'iPhone et de la montre montre « maintenant » — le relevé,
//! puis l'heure prévue une fois son heure passée — au-dessus du minimum et du
//! maximum du jour. La fusion n'embrasse que le relevé : à 17 h, on lisait
//! « 22° » puis « ↑ 13° ». Les widgets, la complication et l'île passent par
//! `Horizon.jour(contenant:embrassant:dans:)` ; l'écran principal passe par
//! `AgroForecast.joursAutourDeMaintenant`, qui fait de même.
//!
//! Comme `uv.rs`, le test lit les vues SwiftUI : un jour tiré de
//! `aujourdhui()` ou de `joursAVenir()` n'affiche pas ses extrêmes.

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

/// Lignes relues après celle qui tire le jour : l'amplitude s'y affiche.
const APRES: usize = 6;

#[test]
fn l_en_tete_embrasse_la_temperature_affichee() {
    let mut fichiers = Vec::new();
    swift(&racine().join("ios"), &mut fichiers);
    assert!(fichiers.len() > 10, "les vues SwiftUI sont introuvables");

    let mut embrasses = 0;
    let mut nus = Vec::new();
    for fichier in &fichiers {
        if fichier.components().any(|c| c.as_os_str() == "KliimaTests") {
            continue;
        }
        let source = fs::read_to_string(fichier).expect("source lisible");
        let lignes: Vec<&str> = source.lines().collect();
        for (i, ligne) in lignes.iter().enumerate() {
            let code = ligne.trim_start();
            if code.starts_with("//") || code.starts_with("///") {
                continue;
            }
            if code.contains("joursAutourDeMaintenant(") && !code.starts_with("func ") {
                embrasses += 1;
            }
            let brut = code.contains("aujourdhui()") || code.contains("joursAVenir()");
            if !brut {
                continue;
            }
            // Une ligne d'amplitude dans les lignes qui suivent, ou la liste
            // des jours, dont la première ligne est aujourd'hui.
            let fin = (i + APRES + 1).min(lignes.len());
            let amplitude = lignes[i..fin].iter().any(|l| {
                let l = l.trim_start();
                !l.starts_with("//") && l.contains("Text(") && l.contains("temperatureMax")
            });
            let liste = lignes[i.saturating_sub(2)..i].iter().any(|l| l.contains("DailyListView("));
            if amplitude || liste {
                nus.push(format!("{}:{}", fichier.display(), i + 1));
            }
        }
    }
    assert!(
        nus.is_empty(),
        "un maximum du jour affiché sans embrasser la température du moment :\n{}",
        nus.join("\n")
    );
    assert!(embrasses >= 3, "en-têtes introuvables ({embrasses})");
}
