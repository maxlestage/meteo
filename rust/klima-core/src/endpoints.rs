//! Où l'on va chercher la météo.
//!
//! Deux acheminements, et le choix a des conséquences juridiques autant que
//! techniques :
//!
//! - **En direct.** Chaque appareil interroge les fournisseurs lui-même.
//!   Simple, sans infrastructure, mais la facture Open-Meteo suit le nombre
//!   d'utilisateurs, MET Norway reste hors de portée du navigateur, et le plan
//!   gratuit d'Open-Meteo interdit l'usage commercial.
//! - **Par le relais.** Un service interroge une fois pour tout le monde. Il
//!   détient la clé du plan commercial, pose l'en-tête que MET Norway exige, et
//!   mutualise le cache.
//!
//! Un écart avec le TypeScript, et il est volontaire : là-bas, l'acheminement
//! est un réglage de processus qu'on change par `useRelay()`. Ici, `Endpoints`
//! est une valeur que l'appelant garde. Le relais est un serveur qui répond à
//! tout le monde en même temps ; un réglage global y serait au mieux inutile,
//! au pire une source de surprises entre deux requêtes.

/// Comment les appels sortent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Direct,
    Relais,
}

impl Transport {
    pub fn code(self) -> &'static str {
        match self {
            Transport::Direct => "direct",
            Transport::Relais => "relais",
        }
    }
}

/// Les six adresses des fournisseurs, et par où elles passent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoints {
    pub transport: Transport,
    pub open_meteo_forecast: String,
    pub open_meteo_search: String,
    /// La qualité de l'air et les pollens (Copernicus, par Open-Meteo).
    pub open_meteo_air: String,
    pub met_norway: String,
    pub bright_sky: String,
    /// Les bulletins d'aéroport (METAR) de l'Aviation Weather Center : ce
    /// qu'on voit tomber (`ciel`).
    pub aviation: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Endpoints::direct()
    }
}

impl Endpoints {
    /// Chaque appareil pour soi : les adresses publiques des fournisseurs.
    pub fn direct() -> Self {
        Endpoints {
            transport: Transport::Direct,
            open_meteo_forecast: "https://api.open-meteo.com/v1/forecast".to_owned(),
            open_meteo_search: "https://geocoding-api.open-meteo.com/v1/search".to_owned(),
            open_meteo_air: "https://air-quality-api.open-meteo.com/v1/air-quality".to_owned(),
            met_norway: "https://api.met.no/weatherapi/locationforecast/2.0/compact".to_owned(),
            bright_sky: "https://api.brightsky.dev/current_weather".to_owned(),
            aviation: "https://aviationweather.gov/api/data/metar".to_owned(),
        }
    }

    /// Les mêmes fournisseurs, vus à travers le relais.
    pub fn relais(origin: &str) -> Self {
        let base = origin.trim_end_matches('/');
        Endpoints {
            transport: Transport::Relais,
            open_meteo_forecast: format!("{base}/v1/open-meteo/forecast"),
            open_meteo_search: format!("{base}/v1/open-meteo/search"),
            open_meteo_air: format!("{base}/v1/open-meteo/air-quality"),
            met_norway: format!("{base}/v1/met-norway/compact"),
            bright_sky: format!("{base}/v1/bright-sky/current"),
            aviation: format!("{base}/v1/aviation/metar"),
        }
    }

    /// L'adresse du direct (`/v1/direct`, en WebSocket), quand on passe par un
    /// relais ; `None` en direct chez les fournisseurs, qui ne poussent rien.
    pub fn direct_url(&self) -> Option<String> {
        if self.transport != Transport::Relais {
            return None;
        }
        let base = self.open_meteo_forecast.strip_suffix("/v1/open-meteo/forecast")?;
        let ws = if let Some(reste) = base.strip_prefix("https://") {
            format!("wss://{reste}")
        } else if let Some(reste) = base.strip_prefix("http://") {
            format!("ws://{reste}")
        } else {
            return None;
        };
        Some(format!("{ws}/v1/direct"))
    }

    /// L'adresse du radar (`/v1/radar`), quand on passe par un relais :
    /// c'est lui qui lit la mosaïque européenne, trop lourde pour un
    /// téléphone ou un navigateur. `None` en direct.
    pub fn radar_url(&self) -> Option<String> {
        if self.transport != Transport::Relais {
            return None;
        }
        let base = self.open_meteo_forecast.strip_suffix("/v1/open-meteo/forecast")?;
        Some(format!("{base}/v1/radar"))
    }

    /// Les six adresses, pour les vérifications d'ensemble.
    pub fn urls(&self) -> [&str; 6] {
        [
            &self.open_meteo_forecast,
            &self.open_meteo_search,
            &self.open_meteo_air,
            &self.met_norway,
            &self.bright_sky,
            &self.aviation,
        ]
    }
}

/// La valeur qui dit « le relais, c'est l'hôte qui sert cette page ».
///
/// Quand le relais sert lui-même l'application, son adresse n'est connue qu'au
/// moment de l'affichage : elle dépend du nom de domaine, qui change d'un
/// hébergement à l'autre et n'existe pas à la construction.
pub const MEME_ORIGINE: &str = "meme-origine";

/// L'adresse du relais, d'après le réglage et l'origine de la page.
///
/// Rien de configuré : pas de relais, chaque navigateur pour soi. C'est le
/// défaut, et il reste sur le plan gratuit d'Open-Meteo.
pub fn relay_from(configure: Option<&str>, origine: &str) -> Option<String> {
    let valeur = configure.unwrap_or("").trim();
    match valeur {
        "" => None,
        MEME_ORIGINE => Some(origine.to_owned()),
        adresse => Some(adresse.to_owned()),
    }
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn en_direct_par_defaut() {
        assert_eq!(Endpoints::default(), Endpoints::direct());
        assert_eq!(Endpoints::default().transport, Transport::Direct);
    }

    #[test]
    fn le_relais_remplace_les_six_adresses_dun_coup() {
        let via = Endpoints::relais("https://relais.klima");

        assert_eq!(via.transport, Transport::Relais);
        for url in via.urls() {
            assert!(url.starts_with("https://relais.klima/"), "{url}");
            // Aucune adresse de fournisseur ne subsiste : tout passe par le relais.
            assert!(!url.contains("open-meteo.com"), "{url}");
            assert!(!url.contains("met.no"), "{url}");
            assert!(!url.contains("brightsky"), "{url}");
            assert!(!url.contains("aviationweather"), "{url}");
        }
    }

    #[test]
    fn une_barre_oblique_en_trop_ne_double_pas_dans_lurl() {
        assert_eq!(
            Endpoints::relais("https://relais.klima//").open_meteo_forecast,
            "https://relais.klima/v1/open-meteo/forecast"
        );
    }

    #[test]
    fn les_six_adresses_directes_sont_distinctes_et_en_https() {
        let direct = Endpoints::direct();
        for url in direct.urls() {
            assert!(url.starts_with("https://"), "{url}");
        }
        let mut uniques = direct.urls().to_vec();
        uniques.sort_unstable();
        uniques.dedup();
        assert_eq!(uniques.len(), 6);
    }

    #[test]
    fn le_radar_ne_passe_que_par_le_relais() {
        assert_eq!(
            Endpoints::relais("https://klima.example/").radar_url().as_deref(),
            Some("https://klima.example/v1/radar")
        );
        assert_eq!(Endpoints::direct().radar_url(), None);
    }

    #[test]
    fn le_direct_passe_par_le_relais_en_websocket() {
        assert_eq!(
            Endpoints::relais("https://klima.example/").direct_url().as_deref(),
            Some("wss://klima.example/v1/direct")
        );
        assert_eq!(
            Endpoints::relais("http://localhost:8787").direct_url().as_deref(),
            Some("ws://localhost:8787/v1/direct")
        );
        // Sans relais, personne pour pousser.
        assert_eq!(Endpoints::direct().direct_url(), None);
    }

    #[test]
    fn rien_de_configure_chaque_navigateur_pour_soi() {
        assert_eq!(relay_from(None, "https://klima.example"), None);
        assert_eq!(relay_from(Some(""), "https://klima.example"), None);
        assert_eq!(relay_from(Some("   "), "https://klima.example"), None);
    }

    #[test]
    fn une_adresse_configuree_est_prise_telle_quelle() {
        assert_eq!(
            relay_from(Some("https://relais.example"), "https://klima.example").as_deref(),
            Some("https://relais.example")
        );
    }

    #[test]
    fn meme_origine_designe_lhote_qui_sert_la_page() {
        // Le cas de l'hébergement unique : le relais sert la page, et son
        // adresse n'est connue qu'au moment de l'affichage.
        assert_eq!(
            relay_from(Some(MEME_ORIGINE), "https://klima-abc.herokuapp.com").as_deref(),
            Some("https://klima-abc.herokuapp.com")
        );
    }

    #[test]
    fn les_espaces_autour_du_reglage_ne_comptent_pas() {
        assert_eq!(
            relay_from(Some("  meme-origine  "), "https://x.example").as_deref(),
            Some("https://x.example")
        );
    }
}
