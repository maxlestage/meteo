//! Mise en forme des nombres selon la langue.
//!
//! Le TypeScript que ce module remplace déléguait tout à `Intl`, qui n'existe
//! pas en WASM. Les règles sont donc écrites ici, et relevées sur ce que rend
//! `Intl` pour les trois langues de Klima — les valeurs attendues des tests
//! viennent de là, séparateurs invisibles compris.
//!
//! Ce qu'il y a à savoir, et qu'on ne devine pas :
//!
//! - Le français sépare les milliers par une **espace fine insécable**
//!   (U+202F), mais colle son pourcentage avec une **espace insécable
//!   ordinaire** (U+00A0). Deux caractères différents dans la même langue.
//! - L'espagnol groupe les milliers par un point, mais **seulement à partir de
//!   cinq chiffres** : 1234 s'écrit « 1234 », 1234567 s'écrit « 1.234.567 ».
//! - L'anglais ne met pas d'espace devant son `%`.
//!
//! Deux arrondis cohabitent, et ce n'est pas une négligence : les nombres
//! suivent celui d'`Intl` (le demi s'éloigne de zéro), la température celui de
//! `Math.round` (le demi monte), ce qui fait qu'un demi-degré sous zéro
//! s'affiche « 0° » et jamais « -0° ».

use crate::i18n::Language;

/// Espace insécable. Elle attache l'unité à son nombre : « 2,1 mm » ne se
/// coupe pas en fin de ligne.
pub const NBSP: char = '\u{00a0}';

/// Espace fine insécable, celle des milliers en français.
const NNBSP: char = '\u{202f}';

/// Les conventions d'écriture d'une langue.
struct Typographie {
    decimal: char,
    milliers: char,
    /// Nombre de chiffres à partir duquel on sépare les milliers.
    seuil_milliers: usize,
    /// Ce qui s'insère entre le nombre et son `%`.
    avant_pourcent: &'static str,
}

const FR: Typographie =
    Typographie { decimal: ',', milliers: NNBSP, seuil_milliers: 4, avant_pourcent: "\u{00a0}" };
const EN: Typographie =
    Typographie { decimal: '.', milliers: ',', seuil_milliers: 4, avant_pourcent: "" };
const ES: Typographie =
    Typographie { decimal: ',', milliers: '.', seuil_milliers: 5, avant_pourcent: "\u{00a0}" };

/// Mise en forme pour une langue donnée.
#[derive(Debug, Clone, Copy)]
pub struct Formats {
    language: Language,
}

impl Formats {
    pub fn new(language: Language) -> Self {
        Formats { language }
    }

    pub fn language(self) -> Language {
        self.language
    }

    fn typographie(self) -> Typographie {
        match self.language {
            Language::Fr => FR,
            Language::En => EN,
            Language::Es => ES,
        }
    }

    /// « 4,8 » en français, « 4.8 » en anglais.
    pub fn decimal(self, value: f64, digits: u32) -> String {
        let t = self.typographie();
        let texte = format!("{:.*}", digits as usize, arrondi_eloigne(value, digits));

        let (signe, reste) = match texte.strip_prefix('-') {
            Some(reste) => ("-", reste),
            None => ("", texte.as_str()),
        };
        let (entier, fraction) = match reste.split_once('.') {
            Some((entier, fraction)) => (entier, Some(fraction)),
            None => (reste, None),
        };

        let mut out = String::with_capacity(reste.len() + 4);
        out.push_str(signe);
        out.push_str(&grouper(entier, t.milliers, t.seuil_milliers));
        if let Some(fraction) = fraction {
            out.push(t.decimal);
            out.push_str(fraction);
        }
        out
    }

    /// « 4,8 mm », l'unité attachée par une espace insécable.
    pub fn unit(self, value: f64, unit: &str, digits: u32) -> String {
        format!("{}{NBSP}{unit}", self.decimal(value, digits))
    }

    /// « +2,7 mm » : le signe rend un bilan lisible d'un coup d'œil. Un bilan
    /// nul n'est pas « positif » et ne prend pas de signe.
    pub fn signed_unit(self, value: f64, unit: &str, digits: u32) -> String {
        let signe = if value > 0.0 { "+" } else { "" };
        format!("{signe}{}", self.unit(value, unit, digits))
    }

    /// « 27 % » — la ponctuation est propre à chaque langue.
    pub fn percent(self, value: f64) -> String {
        format!("{}{}%", self.decimal(value, 0), self.typographie().avant_pourcent)
    }

    /// « 16° », arrondi comme sur un bulletin météo.
    pub fn temperature(self, value: f64) -> String {
        // `floor(x + 0.5)` et non l'arrondi des nombres : un demi-degré sous
        // zéro doit donner « 0° », pas « -0° » ni « -1° ».
        format!("{}°", (value + 0.5).floor())
    }
}

/// L'arrondi d'`Intl` : à mi-chemin, on s'éloigne de zéro.
fn arrondi_eloigne(value: f64, digits: u32) -> f64 {
    let factor = 10f64.powi(digits as i32);
    (value * factor).round() / factor
}

/// Insère le séparateur de milliers dans une suite de chiffres.
fn grouper(entier: &str, separateur: char, seuil: usize) -> String {
    if entier.len() < seuil {
        return entier.to_owned();
    }

    let chiffres: Vec<char> = entier.chars().collect();
    let mut out = String::with_capacity(entier.len() + entier.len() / 3);
    for (index, chiffre) in chiffres.iter().enumerate() {
        if index > 0 && (chiffres.len() - index) % 3 == 0 {
            out.push(separateur);
        }
        out.push(*chiffre);
    }
    out
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    fn f(language: Language) -> Formats {
        Formats::new(language)
    }


    #[test]
    fn virgule_decimale_en_francais_et_en_espagnol_point_en_anglais() {
        assert_eq!(f(Language::Fr).decimal(4.8, 1), "4,8");
        assert_eq!(f(Language::Es).decimal(4.8, 1), "4,8");
        assert_eq!(f(Language::En).decimal(4.8, 1), "4.8");
    }

    #[test]
    fn unite_collee_par_une_espace_insecable() {
        assert_eq!(f(Language::Fr).unit(2.1, "mm", 1), "2,1\u{00a0}mm");
        assert_eq!(f(Language::En).unit(2.1, "mm", 1), "2.1\u{00a0}mm");
    }

    #[test]
    fn le_bilan_porte_son_signe() {
        assert_eq!(f(Language::Fr).signed_unit(2.7, "mm", 1), "+2,7\u{00a0}mm");
        assert_eq!(f(Language::Fr).signed_unit(-1.2, "mm", 1), "-1,2\u{00a0}mm");
        // Un bilan nul n'est pas « positif » : pas de signe.
        assert_eq!(f(Language::Fr).signed_unit(0.0, "mm", 1), "0,0\u{00a0}mm");
    }

    #[test]
    fn pourcentages_ponctues_selon_la_langue() {
        assert_eq!(f(Language::Fr).percent(26.6), "27\u{00a0}%");
        assert_eq!(f(Language::En).percent(26.6), "27%");
        assert_eq!(f(Language::Es).percent(26.6), "27\u{00a0}%");
    }

    #[test]
    fn temperatures_arrondies() {
        assert_eq!(f(Language::Fr).temperature(16.4), "16°");
        assert_eq!(f(Language::Fr).temperature(-1.4), "-1°");
        // Un demi-degré sous zéro s'affiche « 0° », jamais « -0° ».
        assert_eq!(f(Language::En).temperature(-0.5), "0°");
    }

    #[test]
    fn les_milliers_suivent_la_langue_separateurs_invisibles_compris() {
        // Relevé sur `Intl` : le français sépare par une espace fine
        // insécable (U+202F), l'espagnol par un point mais seulement à partir
        // de cinq chiffres, l'anglais par une virgule.
        assert_eq!(f(Language::Fr).decimal(1234.5, 1), "1\u{202f}234,5");
        assert_eq!(f(Language::En).decimal(1234.5, 1), "1,234.5");
        assert_eq!(f(Language::Es).decimal(1234.5, 1), "1234,5");

        assert_eq!(f(Language::Fr).decimal(1234567.5, 1), "1\u{202f}234\u{202f}567,5");
        assert_eq!(f(Language::En).decimal(1234567.5, 1), "1,234,567.5");
        assert_eq!(f(Language::Es).decimal(1234567.5, 1), "1.234.567,5");
    }

    #[test]
    fn lespace_du_pourcentage_nest_pas_celle_des_milliers() {
        // U+00A0 devant le `%`, U+202F entre les milliers. Dans la même langue.
        assert!(f(Language::Fr).percent(26.6).contains('\u{00a0}'));
        assert!(!f(Language::Fr).percent(26.6).contains('\u{202f}'));
        assert!(f(Language::Fr).decimal(1234.0, 0).contains('\u{202f}'));
    }

    #[test]
    fn le_signe_reste_devant_les_milliers() {
        assert_eq!(f(Language::Fr).decimal(-1234.5, 1), "-1\u{202f}234,5");
    }

    #[test]
    fn un_nombre_negatif_tout_juste_sous_zero_garde_son_signe() {
        // Comme `Intl`, qui écrit « -0,0 » : la valeur est négative, et
        // l'effacer ferait croire à un bilan nul.
        assert_eq!(f(Language::Fr).decimal(-0.04, 1), "-0,0");
    }

    #[test]
    fn les_decimales_demandees_sont_toutes_ecrites() {
        assert_eq!(f(Language::Fr).decimal(4.8, 2), "4,80");
        assert_eq!(f(Language::En).decimal(100.0, 0), "100");
    }

}
