//! Fournisseurs de prévision, et ce qu'ils livrent.
//!
//! Un fournisseur est un service qu'on interroge ; il peut livrer plusieurs
//! sources — Open-Meteo redistribue quatre modèles nationaux, MET Norway n'en
//! livre qu'un. Ce que le recoupement compare, ce sont les **sources**.
//!
//! Ce module ne contient que la table et les types : les appels réseau
//! viendront avec les routes du relais. Les mentions de licence, elles,
//! voyagent avec la source — c'est la condition pour l'afficher.

/// Là où un fournisseur peut être appelé sans enfreindre ses conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Web,
    Native,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherSource {
    pub id: &'static str,
    /// Nom du modèle ou du produit.
    pub name: &'static str,
    /// Service qui le produit.
    pub institution: &'static str,
    /// Code pays ou zone du service.
    pub country: &'static str,
    /// Fournisseur par lequel on l'obtient.
    pub provider: &'static str,
    /// Mention que la licence impose d'afficher.
    pub attribution: &'static str,
}

/// Relevé d'une source pour l'heure en cours.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceReading {
    pub source: WeatherSource,
    pub temperature: f64,
    /// Précipitations sur l'heure (mm).
    pub precipitation: f64,
    /// Vent moyen (km/h).
    pub wind_speed: f64,
}

/// Ce qu'un fournisseur a renvoyé — vide quand il n'a rien pu dire.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderOutcome {
    pub provider_id: String,
    pub readings: Vec<SourceReading>,
}

const OPEN_METEO_ATTRIBUTION: &str = "Open-Meteo — modèles Météo-France, ECMWF, DWD et NOAA";

/// Les quatre modèles nationaux qu'Open-Meteo redistribue.
/// Ce que Klima dit de lui-même aux fournisseurs.
///
/// MET Norway l'exige : une requête anonyme est refusée. Un navigateur n'a pas
/// le droit de poser cet en-tête, d'où la règle du dépôt — « seulement là où
/// l'on peut se nommer » : en direct le natif, sinon le relais, qui le pose
/// pour tout le monde.
pub const USER_AGENT: &str = "Klima/1.0 (météo agricole; https://maxlestage.github.io/meteo/)";

pub const OPEN_METEO_SOURCES: [WeatherSource; 4] = [
    WeatherSource {
        id: "meteofrance_seamless",
        name: "AROME / ARPEGE",
        institution: "Météo-France",
        country: "FR",
        provider: "open-meteo",
        attribution: OPEN_METEO_ATTRIBUTION,
    },
    WeatherSource {
        id: "ecmwf_ifs025",
        name: "IFS",
        institution: "ECMWF",
        country: "EU",
        provider: "open-meteo",
        attribution: OPEN_METEO_ATTRIBUTION,
    },
    WeatherSource {
        id: "icon_seamless",
        name: "ICON",
        institution: "Deutscher Wetterdienst",
        country: "DE",
        provider: "open-meteo",
        attribution: OPEN_METEO_ATTRIBUTION,
    },
    WeatherSource {
        id: "gfs_seamless",
        name: "GFS",
        institution: "NOAA",
        country: "US",
        provider: "open-meteo",
        attribution: OPEN_METEO_ATTRIBUTION,
    },
];

/// Une source par son identifiant.
pub fn weather_source(id: &str) -> Option<&'static WeatherSource> {
    OPEN_METEO_SOURCES.iter().find(|s| s.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn quatre_modeles_chez_open_meteo_tous_dinstituts_differents() {
        assert_eq!(OPEN_METEO_SOURCES.len(), 4);
        let instituts: HashSet<_> = OPEN_METEO_SOURCES.iter().map(|s| s.institution).collect();
        assert_eq!(instituts.len(), 4);
        let identifiants: HashSet<_> = OPEN_METEO_SOURCES.iter().map(|s| s.id).collect();
        assert_eq!(identifiants.len(), 4);
    }

    #[test]
    fn recherche_par_identifiant() {
        assert_eq!(weather_source("meteofrance_seamless").unwrap().institution, "Météo-France");
        assert!(weather_source("inconnu").is_none());
    }

    /// La licence impose d'afficher la mention : une source sans attribution
    /// ne doit pas pouvoir entrer dans la table.
    #[test]
    fn chaque_source_porte_sa_mention_de_licence() {
        for source in &OPEN_METEO_SOURCES {
            assert!(!source.attribution.is_empty(), "{} sans mention", source.id);
        }
    }
}
