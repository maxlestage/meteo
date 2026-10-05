//! Fournisseurs de prévision, et ce qu'ils livrent.
//!
//! Un fournisseur est un service qu'on interroge ; il peut livrer plusieurs
//! sources — Open-Meteo redistribue quatre modèles nationaux, MET Norway n'en
//! livre qu'un. Ce que le recoupement compare, ce sont les **sources**.
//!
//! Ce module tient la table, les types et la règle qui dit qui peut appeler
//! qui. Le décodage des réponses vit dans `klima-api` ; l'appel lui-même chez
//! qui a un client HTTP. Les mentions de licence, elles, voyagent avec la
//! source — c'est la condition pour l'afficher.

use crate::endpoints::Transport;

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
pub const USER_AGENT: &str = "Klima/1.0 (météo de ville; https://maxlestage.github.io/meteo/)";

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

/// Institut météorologique norvégien — `Locationforecast 2.0`.
///
/// Ses conditions imposent un en-tête `User-Agent` identifiant l'application
/// et un moyen de contact. Un navigateur interdit de le fixer : en appel
/// direct, ce fournisseur est donc réservé au natif, plutôt que d'envoyer des
/// requêtes anonymes contre leur volonté.
pub const MET_NORWAY_SOURCE: WeatherSource = WeatherSource {
    id: "met-no-locationforecast",
    name: "Locationforecast",
    institution: "MET Norway",
    country: "NO",
    provider: "met-norway",
    attribution:
        "Données de MET Norway (Norwegian Meteorological Institute), licence NLOD / CC BY 4.0",
};

/// Bright Sky — relais ouvert des données du Deutscher Wetterdienst.
///
/// Contrairement aux autres, cette source n'est pas une sortie de modèle mais
/// une **observation de station** : elle dit ce qu'il fait, pas ce qui est
/// prévu. La couverture suit le réseau du DWD, dense en Allemagne et
/// clairsemée ailleurs — hors de portée d'une station, elle ne renvoie rien et
/// sort du recoupement.
pub const BRIGHT_SKY_SOURCE: WeatherSource = WeatherSource {
    id: "brightsky-dwd",
    name: "Observation DWD",
    institution: "Deutscher Wetterdienst",
    country: "DE",
    provider: "bright-sky",
    attribution: "Bright Sky — données du Deutscher Wetterdienst, licence CC BY 4.0",
};

/// Un service qu'on interroge, et ce qu'il a le droit de recevoir comme appel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub id: &'static str,
    pub institution: &'static str,
    /// Plateformes d'où l'appel **direct** est légitime et possible.
    pub platforms: &'static [Platform],
    /// Vrai quand le relais peut l'interroger pour le compte de n'importe quel
    /// client. C'est ce qui ouvre MET Norway au navigateur : l'obstacle n'était
    /// pas la plateforme mais l'en-tête d'identification, que le relais pose.
    pub via_relay: bool,
    /// Mention à afficher dès qu'une de ses sources est utilisée.
    pub attribution: &'static str,
}

pub const PROVIDERS: [Provider; 3] = [
    Provider {
        id: "open-meteo",
        institution: "Open-Meteo",
        platforms: &[Platform::Web, Platform::Native],
        via_relay: true,
        attribution: OPEN_METEO_ATTRIBUTION,
    },
    Provider {
        id: "met-norway",
        institution: "MET Norway",
        platforms: &[Platform::Native],
        via_relay: true,
        attribution: MET_NORWAY_SOURCE.attribution,
    },
    Provider {
        id: "bright-sky",
        institution: "Bright Sky / DWD",
        platforms: &[Platform::Web, Platform::Native],
        via_relay: true,
        attribution: BRIGHT_SKY_SOURCE.attribution,
    },
];

/// Fournisseurs interrogeables depuis la plateforme donnée.
///
/// En appel direct, c'est la plateforme qui décide : un navigateur ne peut pas
/// se nommer auprès de MET Norway, donc il ne l'appelle pas. Par le relais,
/// c'est le serveur qui appelle et qui se nomme — la plateforme du client
/// n'entre plus en ligne de compte, et le web gagne les mêmes sources que le
/// natif.
pub fn providers_for(platform: Platform, transport: Transport) -> Vec<&'static Provider> {
    PROVIDERS
        .iter()
        .filter(|provider| match transport {
            Transport::Relais => provider.via_relay,
            Transport::Direct => provider.platforms.contains(&platform),
        })
        .collect()
}

/// Toutes les sources connues, tous fournisseurs confondus.
pub fn all_sources() -> Vec<&'static WeatherSource> {
    OPEN_METEO_SOURCES
        .iter()
        .chain(std::iter::once(&MET_NORWAY_SOURCE))
        .chain(std::iter::once(&BRIGHT_SKY_SOURCE))
        .collect()
}

/// Une source par son identifiant.
pub fn weather_source(id: &str) -> Option<&'static WeatherSource> {
    all_sources().into_iter().find(|s| s.id == id)
}

/// Mentions à afficher pour les sources effectivement utilisées, une fois
/// chacune.
pub fn attributions_for(readings: &[SourceReading]) -> Vec<&'static str> {
    let mut mentions = Vec::new();
    for reading in readings {
        if !mentions.contains(&reading.source.attribution) {
            mentions.push(reading.source.attribution);
        }
    }
    mentions
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
        assert_eq!(weather_source("brightsky-dwd").unwrap().name, "Observation DWD");
        assert!(weather_source("inconnu").is_none());
    }

    #[test]
    fn six_sources_pour_trois_fournisseurs() {
        assert_eq!(PROVIDERS.len(), 3);
        assert_eq!(all_sources().len(), 6);
    }

    #[test]
    fn met_norway_nest_pas_appele_directement_depuis_le_web() {
        // Ses conditions imposent un User-Agent qu'un navigateur refuse de
        // poser.
        assert_eq!(
            PROVIDERS.iter().find(|p| p.id == "met-norway").unwrap().platforms,
            &[Platform::Native]
        );
        let web = providers_for(Platform::Web, Transport::Direct);
        assert!(!web.iter().any(|p| p.id == "met-norway"));
        let natif = providers_for(Platform::Native, Transport::Direct);
        assert!(natif.iter().any(|p| p.id == "met-norway"));
    }

    #[test]
    fn le_relais_le_rend_accessible_au_web_cest_lui_qui_se_nomme() {
        let web = providers_for(Platform::Web, Transport::Relais);
        assert!(web.iter().any(|p| p.id == "met-norway"));
        assert_eq!(web, providers_for(Platform::Native, Transport::Relais));
    }

    #[test]
    fn le_natif_interroge_un_fournisseur_de_plus_que_le_web() {
        assert_eq!(
            providers_for(Platform::Native, Transport::Direct).len(),
            providers_for(Platform::Web, Transport::Direct).len() + 1
        );
    }

    #[test]
    fn les_mentions_ne_sont_listees_quune_fois_par_licence() {
        let releve = |source: WeatherSource| SourceReading {
            source,
            temperature: 18.0,
            precipitation: 0.0,
            wind_speed: 12.0,
        };
        let readings = [
            releve(OPEN_METEO_SOURCES[0].clone()),
            releve(OPEN_METEO_SOURCES[1].clone()),
            releve(BRIGHT_SKY_SOURCE),
        ];

        let mentions = attributions_for(&readings);
        assert_eq!(mentions.len(), 2);
        assert!(mentions.iter().any(|m| m.contains("Open-Meteo")));
        assert!(mentions.iter().any(|m| m.contains("Bright Sky")));
    }

    /// La licence impose d'afficher la mention : une source sans attribution
    /// ne doit pas pouvoir entrer dans la table.
    #[test]
    fn chaque_source_porte_sa_mention_de_licence() {
        for source in all_sources() {
            assert!(!source.attribution.is_empty(), "{} sans mention", source.id);
        }
    }
}
