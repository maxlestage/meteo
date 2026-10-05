//! Les langues de Klima, et la traduction des libellés partagés.
//!
//! Le domaine ne fabrique jamais de phrase : il renvoie des états et des
//! motifs structurés, que l'interface traduit. C'est pour cela que ce module
//! est ici et pas dans l'interface : un motif à trous — « il gèlera à
//! {temperature} » — est produit par une règle du domaine, et la règle vit
//! dans le cœur. Seul le remplissage du trou dépend de la langue.
//!
//! Les catalogues partagés suivront dans `messages.rs` ; ce module ne connaît
//! que la mécanique.

use std::collections::BTreeMap;
use std::fmt;

/// Les trois langues de Klima.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Language {
    Fr,
    En,
    Es,
}

/// Dans l'ordre d'affichage du sélecteur de langue.
pub const LANGUAGES: [Language; 3] = [Language::Fr, Language::En, Language::Es];

/// Langue de référence : celle dans laquelle les textes sont écrits d'abord.
pub const REFERENCE_LANGUAGE: Language = Language::Fr;

impl Language {
    /// Code ISO à deux lettres, tel qu'il apparaît dans une adresse.
    pub fn code(self) -> &'static str {
        match self {
            Language::Fr => "fr",
            Language::En => "en",
            Language::Es => "es",
        }
    }

    /// Nom de la langue dans cette langue — jamais traduit.
    pub fn name(self) -> &'static str {
        match self {
            Language::Fr => "Français",
            Language::En => "English",
            Language::Es => "Español",
        }
    }

    /// Étiquette de locale, pour le formatage des nombres et des dates.
    pub fn locale(self) -> &'static str {
        match self {
            Language::Fr => "fr-FR",
            Language::En => "en-GB",
            Language::Es => "es-ES",
        }
    }

    /// Reconnaît un code de langue, variante régionale comprise
    /// (« fr-BE » compte comme « fr »).
    pub fn parse(value: &str) -> Option<Language> {
        let base = value.split('-').next().unwrap_or("").to_ascii_lowercase();
        LANGUAGES.into_iter().find(|language| language.code() == base)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Première langue reconnue parmi celles que propose le navigateur. Faute de
/// correspondance, la référence : une langue inconnue vaut mieux que rien.
pub fn detect_language<S: AsRef<str>>(candidates: &[S]) -> Language {
    candidates
        .iter()
        .find_map(|candidate| Language::parse(candidate.as_ref()))
        .unwrap_or(REFERENCE_LANGUAGE)
}

/* ---------------------------------------------------------------- */
/* Les valeurs qui bouchent les trous                                */
/* ---------------------------------------------------------------- */

/// Une valeur à insérer dans un motif.
///
/// Le cœur ne met pas en forme : il passe un nombre brut, et l'interface le
/// formate selon la langue du lecteur. D'où ces deux cas et pas davantage.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    Text(String),
    Number(f64),
}

impl fmt::Display for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamValue::Text(text) => f.write_str(text),
            ParamValue::Number(value) => write!(f, "{value}"),
        }
    }
}

impl From<&str> for ParamValue {
    fn from(value: &str) -> Self {
        ParamValue::Text(value.to_owned())
    }
}

impl From<String> for ParamValue {
    fn from(value: String) -> Self {
        ParamValue::Text(value)
    }
}

impl From<f64> for ParamValue {
    fn from(value: f64) -> Self {
        ParamValue::Number(value)
    }
}

impl From<i32> for ParamValue {
    fn from(value: i32) -> Self {
        ParamValue::Number(f64::from(value))
    }
}

impl From<usize> for ParamValue {
    fn from(value: usize) -> Self {
        ParamValue::Number(value as f64)
    }
}

/// Les valeurs d'un motif, par nom de trou.
pub type Params = BTreeMap<String, ParamValue>;

/// Rassemble des valeurs nommées. `params([("temperature", (-2.4).into())])`.
pub fn params<const N: usize>(entries: [(&str, ParamValue); N]) -> Params {
    entries
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

/* ---------------------------------------------------------------- */
/* Traduction                                                        */
/* ---------------------------------------------------------------- */

/// Les textes d'une langue, par clé.
pub type Catalog = BTreeMap<&'static str, &'static str>;

/// Un catalogue décliné dans les trois langues.
#[derive(Debug, Clone, Default)]
pub struct MessageSet {
    pub fr: Catalog,
    pub en: Catalog,
    pub es: Catalog,
}

impl MessageSet {
    pub fn catalog(&self, language: Language) -> &Catalog {
        match language {
            Language::Fr => &self.fr,
            Language::En => &self.en,
            Language::Es => &self.es,
        }
    }

    /// Le texte d'une clé dans cette langue, sans repli.
    pub fn get(&self, language: Language, key: &str) -> Option<&'static str> {
        self.catalog(language).get(key).copied()
    }
}

/// Remplace les `{jetons}` d'un motif par leurs valeurs.
///
/// Un jeton sans valeur reste en place : un trou visible se corrige, un trou
/// effacé produit une phrase fausse que personne ne remarque.
pub fn interpolate(template: &str, params: &Params) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let name_len = after
            .char_indices()
            .find(|(_, c)| !(c.is_ascii_alphanumeric() || *c == '_'))
            .map_or(after.len(), |(index, _)| index);

        match (name_len > 0, after[name_len..].starts_with('}')) {
            (true, true) => {
                let name = &after[..name_len];
                match params.get(name) {
                    Some(value) => out.push_str(&value.to_string()),
                    None => {
                        out.push('{');
                        out.push_str(name);
                        out.push('}');
                    }
                }
                rest = &after[name_len + 1..];
            }
            // Ni un jeton ni une accolade fermante : l'accolade est du texte.
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }

    out.push_str(rest);
    out
}

/// Traduit en lisant plusieurs catalogues dans l'ordre donné.
///
/// Une clé absente de la langue demandée retombe sur la référence, puis sur la
/// clé elle-même — un texte manquant se voit, il ne disparaît pas.
pub struct Translator<'a> {
    language: Language,
    sets: Vec<&'a MessageSet>,
}

impl<'a> Translator<'a> {
    pub fn new(language: Language, sets: &[&'a MessageSet]) -> Self {
        Translator { language, sets: sets.to_vec() }
    }

    pub fn language(&self) -> Language {
        self.language
    }

    /// Le texte d'une clé, sans valeur à insérer.
    pub fn t(&self, key: &str) -> String {
        self.with(key, &Params::new())
    }

    /// Le texte d'une clé, trous bouchés.
    pub fn with(&self, key: &str, params: &Params) -> String {
        for set in &self.sets {
            if let Some(template) =
                set.get(self.language, key).or_else(|| set.get(REFERENCE_LANGUAGE, key))
            {
                return interpolate(template, params);
            }
        }
        key.to_owned()
    }
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    fn set(entries: &[(Language, &'static str, &'static str)]) -> MessageSet {
        let mut messages = MessageSet::default();
        for (language, key, value) in entries {
            let catalog = match language {
                Language::Fr => &mut messages.fr,
                Language::En => &mut messages.en,
                Language::Es => &mut messages.es,
            };
            catalog.insert(key, value);
        }
        messages
    }

    #[test]
    fn reconnait_une_variante_regionale() {
        assert_eq!(detect_language(&["fr-BE", "nl"]), Language::Fr);
        assert_eq!(detect_language(&["es-MX"]), Language::Es);
        assert_eq!(detect_language(&["en-US"]), Language::En);
    }

    #[test]
    fn retombe_sur_la_langue_de_reference() {
        assert_eq!(detect_language(&["de-DE", "it"]), Language::Fr);
        assert_eq!(detect_language::<&str>(&[]), Language::Fr);
    }

    #[test]
    fn prend_la_premiere_langue_reconnue_pas_la_premiere_tout_court() {
        assert_eq!(detect_language(&["de", "es-ES", "en"]), Language::Es);
    }

    #[test]
    fn un_code_inconnu_nest_pas_une_langue() {
        assert_eq!(Language::parse("de"), None);
        assert_eq!(Language::parse(""), None);
        assert_eq!(Language::parse("FR-be"), Some(Language::Fr));
    }

    #[test]
    fn interpole_les_valeurs() {
        let values = params([("wind", "24 km/h".into()), ("limit", "19 km/h".into())]);
        assert_eq!(interpolate("Vent {wind} (max {limit})", &values), "Vent 24 km/h (max 19 km/h)");
    }

    #[test]
    fn laisse_un_jeton_inconnu_en_place_plutot_que_deffacer() {
        assert_eq!(interpolate("Vent {wind}", &Params::new()), "Vent {wind}");
    }

    #[test]
    fn une_accolade_qui_nouvre_aucun_jeton_reste_du_texte() {
        assert_eq!(interpolate("{ {wind} }", &params([("wind", 9.into())])), "{ 9 }");
        assert_eq!(interpolate("{}", &Params::new()), "{}");
    }

    #[test]
    fn un_nombre_entier_sinscrit_sans_decimale() {
        assert_eq!(interpolate("{score} %", &params([("score", 90.into())])), "90 %");
        assert_eq!(
            interpolate("{temperature} °C", &params([("temperature", (-2.4).into())])),
            "-2.4 °C"
        );
    }

    #[test]
    fn une_cle_absente_se_voit() {
        let messages = set(&[(Language::Fr, "greeting", "Bonjour")]);
        assert_eq!(Translator::new(Language::En, &[&messages]).t("nexiste.pas"), "nexiste.pas");
    }

    #[test]
    fn une_langue_incomplete_retombe_sur_la_reference() {
        let messages = set(&[(Language::Fr, "greeting", "Bonjour")]);
        assert_eq!(Translator::new(Language::En, &[&messages]).t("greeting"), "Bonjour");
    }

    #[test]
    fn le_premier_catalogue_qui_connait_la_cle_gagne() {
        let propre = set(&[(Language::Fr, "titre", "Klima")]);
        let partage = set(&[(Language::Fr, "titre", "Partagé")]);
        assert_eq!(Translator::new(Language::Fr, &[&propre, &partage]).t("titre"), "Klima");
    }

    #[test]
    fn les_trois_langues_ont_un_nom_et_une_locale() {
        for language in LANGUAGES {
            assert!(!language.name().is_empty());
            assert!(language.locale().starts_with(language.code()));
        }
    }
}
