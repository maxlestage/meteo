//! Ce qui défile de côté, et ce qui n'en a pas le droit.
//!
//! Ce n'est pas une préférence d'implémentation mais une décision de produit,
//! et elle n'est pas la même des deux côtés :
//!
//! - **Le web ne défile pas de côté.** Le bandeau horaire y est une grille qui
//!   se replie et montre douze heures ; les autres se déplient d'un bouton.
//! - **iOS, si — mais il le dit.** La grille essayée là-bas montrait tout d'un
//!   coup, prenait la moitié de l'écran et finissait sur une rangée ébréchée.
//!   Ce qui ne revient pas, c'est `showsIndicators: false` : la première
//!   version défilait sans rien dire, on voyait six heures et il fallait
//!   deviner que les autres existaient.
//!
//! Le test lit les sources plutôt que le rendu : vérifier le rendu demanderait
//! un navigateur et un simulateur dans l'intégration continue, alors que la
//! règle tient en une déclaration qu'on peut relire.
//!
//! Il vivait à la racine du dépôt, en TypeScript, parce qu'il porte sur les
//! deux plateformes à la fois. Il vit maintenant ici : c'est ce paquet qui
//! porte la feuille de style, et l'application iPhone est à deux dossiers de
//! là.

use std::fs;
use std::path::{Path, PathBuf};

/// La racine du dépôt, depuis `rust/klima-web`.
fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn lire(chemin: impl AsRef<Path>) -> String {
    let chemin = racine().join(chemin);
    fs::read_to_string(&chemin)
        .unwrap_or_else(|erreur| panic!("{} : {erreur}", chemin.display()))
}

/* ---- le web ne défile pas de côté ---- */

#[test]
fn aucune_regle_ne_rend_un_bloc_defilable_de_cote() {
    let styles = lire("rust/klima-web/styles.css");

    let coupables: Vec<&str> = styles
        .lines()
        .map(str::trim)
        .filter(|ligne| {
            let sans_espaces: String =
                ligne.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase();
            sans_espaces.starts_with("overflow:") || sans_espaces.starts_with("overflow-x:")
        })
        .filter(|ligne| ligne.contains("auto") || ligne.contains("scroll"))
        .collect();

    assert_eq!(coupables, Vec::<&str>::new());
}

/// La grille est ce qui remplace le défilement : repassée en rangée unique,
/// les heures ressortiraient de l'écran sans que la règle ci-dessus s'en
/// aperçoive.
#[test]
fn le_bandeau_horaire_se_replie_en_grille() {
    let styles = lire("rust/klima-web/styles.css");

    let debut = styles.find(".strip {").expect("un bloc .strip dans la feuille de style");
    let bloc = &styles[debut..];
    let bloc = &bloc[..bloc.find('}').expect("une accolade fermante")];

    assert!(bloc.contains("display: grid"), "{bloc}");
    assert!(
        bloc.contains("grid-template-columns: repeat(auto-fit")
            || bloc.contains("grid-template-columns: repeat(auto-fill"),
        "{bloc}"
    );
}

/* ---- iOS glisse à l'horizontale, mais le dit ---- */

fn vues_ios() -> Vec<String> {
    let dossier = racine().join("ios/Kliima/Views");
    let mut noms: Vec<String> = fs::read_dir(&dossier)
        .unwrap_or_else(|erreur| panic!("{} : {erreur}", dossier.display()))
        .filter_map(|entree| entree.ok())
        .map(|entree| entree.file_name().to_string_lossy().into_owned())
        .filter(|nom| nom.ends_with(".swift"))
        .collect();
    noms.sort();
    noms
}

#[test]
fn il_y_a_bien_des_vues_a_verifier() {
    assert!(!vues_ios().is_empty());
}

#[test]
fn seul_le_bandeau_horaire_defile_de_cote() {
    let coupables: Vec<String> = vues_ios()
        .into_iter()
        .filter(|nom| lire(format!("ios/Kliima/Views/{nom}")).contains("ScrollView(.horizontal"))
        .collect();

    assert_eq!(coupables, ["HourlyStripView.swift"]);
}

/// La règle qui reste, et c'est elle qui compte.
///
/// Un défilement horizontal est acceptable ; un défilement horizontal muet,
/// non.
#[test]
fn lindicateur_est_visible_et_personne_ne_le_cache() {
    let source = lire("ios/Kliima/Views/HourlyStripView.swift");
    assert!(source.contains(".scrollIndicators(.visible)"));

    let muettes: Vec<String> = vues_ios()
        .into_iter()
        .filter(|nom| lire(format!("ios/Kliima/Views/{nom}")).contains("showsIndicators: false"))
        .collect();

    assert_eq!(muettes, Vec::<String>::new());
}
