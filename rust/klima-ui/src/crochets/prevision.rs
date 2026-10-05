//! Charger la prévision d'une ville.
//!
//! L'application et la vitrine faisaient le même appel pour en tirer deux
//! choses différentes — l'une la semaine, l'autre la journée. Ce qui est
//! commun est ici ; ce qu'on en tire reste chez chaque interface.
//!
//! Trois appels, et les deux derniers ne peuvent pas faire échouer le
//! premier : le recoupement des sources et l'air sont des plus, leur absence
//! ne prive de rien. C'est
//! aussi ce qui permet à un fournisseur d'être en panne sans que la météo
//! disparaisse.

use klima_api::open_meteo::{ApiError, Forecast};
use klima_core::air::AirSample;
use klima_core::consensus::{Consensus, consensus_from_outcomes};
use klima_core::endpoints::Endpoints;
use klima_core::position::Parcelle;
use yew::prelude::*;

use crate::horloge;
use crate::reseau;

#[derive(Clone, PartialEq)]
pub struct EtatPrevision {
    pub forecast: Option<Forecast>,
    /// Recoupement des modèles ; absent si la comparaison a échoué.
    pub consensus: Option<Consensus>,
    /// Qualité de l'air et pollens ; absente si le service n'a pas répondu.
    pub air: Option<AirSample>,
    pub loading: bool,
    /// Message déjà traduit : le domaine rend une clé, le crochet la traduit.
    pub error: Option<String>,
}

impl Default for EtatPrevision {
    fn default() -> Self {
        // On part en chargement : la première prévision est toujours en route.
        EtatPrevision { forecast: None, consensus: None, air: None, loading: true, error: None }
    }
}

pub struct Prevision {
    pub etat: EtatPrevision,
    pub reload: Callback<()>,
}

/// Charge la prévision sur `days` jours, puis le recoupement des modèles.
#[hook]
pub fn use_forecast(
    parcelle: Parcelle,
    endpoints: Endpoints,
    days: u32,
    traduire: impl Fn(&ApiError) -> String + 'static,
) -> Prevision {
    let etat = use_state(EtatPrevision::default);
    let nonce = use_state(|| 0_u32);

    {
        let etat = etat.clone();
        let endpoints = endpoints.clone();
        use_effect_with((parcelle.clone(), *nonce), move |(parcelle, _)| {
            let parcelle = parcelle.clone();
            etat.set(EtatPrevision { loading: true, error: None, ..(*etat).clone() });

            wasm_bindgen_futures::spawn_local(async move {
                let maintenant = horloge::maintenant_local();
                match reseau::forecast(&endpoints, &parcelle, days, maintenant).await {
                    Ok(forecast) => {
                        etat.set(EtatPrevision {
                            forecast: Some(forecast.clone()),
                            consensus: (*etat).consensus.clone(),
                            air: (*etat).air.clone(),
                            loading: false,
                            error: None,
                        });

                        // Le recoupement vient après, et à part : son échec ne
                        // doit pas effacer une prévision déjà affichée.
                        let a_la_parcelle =
                            horloge::maintenant_a_la_parcelle(forecast.utc_offset_seconds);
                        let (outcomes, air) = futures::join!(
                            reseau::readings(&endpoints, &parcelle, a_la_parcelle),
                            reseau::air(&endpoints, &parcelle),
                        );
                        etat.set(EtatPrevision {
                            forecast: Some(forecast),
                            consensus: consensus_from_outcomes(&outcomes),
                            air,
                            loading: false,
                            error: None,
                        });
                    }
                    Err(erreur) => etat.set(EtatPrevision {
                        forecast: None,
                        consensus: None,
                        air: None,
                        loading: false,
                        error: Some(traduire(&erreur)),
                    }),
                }
            });
        });
    }

    let reload = {
        let nonce = nonce.clone();
        Callback::from(move |()| nonce.set(*nonce + 1))
    };

    Prevision { etat: (*etat).clone(), reload }
}
