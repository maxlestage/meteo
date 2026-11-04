//! La position ne bloque pas l'affichage, sur la montre non plus.
//!
//! `WatchViewModel.load()` attendait la position du poignet, puis le nom de
//! la commune, avant de demander la moindre prévision : à la première
//! ouverture, la demande d'autorisation laissait un sablier derrière elle.
//! L'iPhone charge la ville connue d'abord et bascule quand la position
//! arrive (`DashboardViewModel.locateIfUnchosen`) ; la montre fait pareil.
//!
//! Comme `maintenant.rs`, le test lit le Swift.

use std::fs;
use std::path::Path;

#[test]
fn la_montre_charge_avant_de_savoir_ou_elle_est() {
    let chemin = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ios/KliimaWatch/WatchViewModel.swift");
    let source = fs::read_to_string(&chemin).expect("WatchViewModel.swift");

    let debut = source.find("func load() async {").expect("`load()` a disparu");
    let fin = debut + source[debut..].find("\n    }\n").expect("fin de `load()`");
    let load = &source[debut..fin];

    assert!(
        !load.contains("currentCoordinate") && !load.contains("reverseGeocodeLocation"),
        "`load()` attend de nouveau la position avant la prévision"
    );
    let releve = load.find("async let poignet").expect("la position n'est plus relevée en parallèle");
    let prevision = load.find("await chargerPrevision()").expect("`load()` ne charge plus la prévision");
    let attente = load.find("await poignet").expect("la position n'est plus attendue");
    assert!(
        releve < prevision && prevision < attente,
        "la prévision doit partir avant que la position soit attendue"
    );
}
