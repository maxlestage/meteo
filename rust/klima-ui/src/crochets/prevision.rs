//! Charger la prévision d'une parcelle.
//!
//! Miroirs : `web/src/hooks/useAgroForecast.ts` et
//! `site/src/hooks/useDayDigest.ts`. Les deux faisaient le même appel pour en
//! tirer deux choses différentes — l'un la semaine, l'autre la journée. Ce qui
//! est commun est ici ; ce qu'on en tire reste chez chaque interface.
//!
//! Deux appels, et le second ne peut pas faire échouer le premier : le
//! recoupement des modèles est un plus, son absence ne prive de rien. C'est
//! aussi ce qui permet à un fournisseur d'être en panne sans que la météo
//! disparaisse.

use klima_api::open_meteo::{AgroApiError, AgroForecast};
use klima_core::consensus::{Consensus, consensus_from_outcomes};
use klima_core::endpoints::Endpoints;
use klima_core::position::Parcelle;
use yew::prelude::*;

use crate::horloge;
use crate::reseau;

#[derive(Clone, PartialEq)]
pub struct EtatPrevision {
    pub forecast: Option<AgroForecast>,
    /// Recoupement des modèles ; absent si la comparaison a échoué.
    pub consensus: Option<Consensus>,
    pub loading: bool,
    /// Message déjà traduit : le domaine rend une clé, le crochet la traduit.
    pub error: Option<String>,
}

impl Default for EtatPrevision {
    fn default() -> Self {
        // On part en chargement : la première prévision est toujours en route.
        EtatPrevision { forecast: None, consensus: None, loading: true, error: None }
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
    traduire: impl Fn(&AgroApiError) -> String + 'static,
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
                            loading: false,
                            error: None,
                        });

                        // Le recoupement vient après, et à part : son échec ne
                        // doit pas effacer une prévision déjà affichée.
                        let a_la_parcelle =
                            horloge::maintenant_a_la_parcelle(forecast.utc_offset_seconds);
                        let outcomes =
                            reseau::readings(&endpoints, &parcelle, a_la_parcelle).await;
                        etat.set(EtatPrevision {
                            consensus: consensus_from_outcomes(&outcomes),
                            ..(*etat).clone()
                        });
                    }
                    Err(erreur) => etat.set(EtatPrevision {
                        forecast: None,
                        consensus: None,
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
