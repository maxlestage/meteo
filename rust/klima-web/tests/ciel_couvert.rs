//! Un ciel couvert est peint en gris, partout.
//!
//! Le fond de l'iPhone et de la montre passait au gris au code 45
//! (brouillard) : sous un ciel « Couvert » (code 3), l'écran montrait un azur
//! de beau temps, quand le web peignait déjà du gris. La règle vit désormais
//! dans `ConditionIcon::ciel_bleu` (`klima-core/src/weather.rs`) et son miroir
//! `WeatherCondition.Icon.cielBleu`, et les fonds s'en servent.
//!
//! Comme `bleu_pluie.rs`, le test lit les vues SwiftUI.

use std::fs;
use std::path::{Path, PathBuf};

fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn lire(chemin: &str) -> String {
    fs::read_to_string(racine().join(chemin)).unwrap_or_else(|e| panic!("{chemin} : {e}"))
}

#[test]
fn les_fonds_suivent_le_pictogramme() {
    for (vue, fonds) in [
        ("ios/Kliima/Views/SkyBackground.swift", 2),
        ("ios/KliimaWatch/WatchDashboardView.swift", 1),
    ] {
        let source = lire(vue);
        assert!(!source.contains("weatherCode >= 45"), "{vue} : seuil de code au lieu de `cielBleu`");
        assert_eq!(source.matches(".icon.cielBleu").count(), fonds, "{vue} : chaque fond passe par `cielBleu`");
    }

    let miroir = lire("ios/Kliima/Models/WeatherCondition.swift");
    assert!(
        miroir.contains("var cielBleu: Bool { self == .clear || self == .partly }"),
        "le miroir Swift de `ciel_bleu` a changé"
    );

    let web = lire("rust/klima-web/src/composants/ciel.rs");
    assert!(web.contains("icone.ciel_bleu()"), "le fond du web ne passe plus par `ciel_bleu`");
}
