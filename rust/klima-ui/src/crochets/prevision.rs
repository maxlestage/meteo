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

use klima_api::open_meteo::{ApiError, Forecast, decode_forecast};
use klima_core::air::AirSample;
use klima_core::consensus::{Consensus, consensus_from_outcomes};
use klima_core::endpoints::Endpoints;
use klima_core::position::Parcelle;
use yew::prelude::*;

use crate::crochets::direct::Corps;
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

/// Charge la prévision sur `days` jours, puis la refait avec toutes les sources.
#[hook]
pub fn use_forecast(
    parcelle: Parcelle,
    endpoints: Endpoints,
    days: u32,
    direct: Corps,
    traduire: impl Fn(&ApiError) -> String + 'static,
) -> Prevision {
    let etat = use_state(EtatPrevision::default);
    let nonce = use_state(|| 0_u32);
    // Vrai dès que le direct a fait la prévision de cette ville : une requête
    // qui reviendrait après ne doit pas la remplacer par plus ancien qu'elle.
    let par_le_direct = use_mut_ref(|| false);

    {
        let etat = etat.clone();
        let endpoints = endpoints.clone();
        let par_le_direct = par_le_direct.clone();
        use_effect_with((parcelle.clone(), *nonce), move |(parcelle, _)| {
            let parcelle = parcelle.clone();
            *par_le_direct.borrow_mut() = false;
            etat.set(EtatPrevision { loading: true, error: None, ..(*etat).clone() });

            wasm_bindgen_futures::spawn_local(async move {
                let maintenant = horloge::maintenant_local();
                match reseau::forecast(&endpoints, &parcelle, days, maintenant).await {
                    Ok(_) if *par_le_direct.borrow() => {}
                    Ok(forecast) => {
                        etat.set(EtatPrevision {
                            forecast: Some(forecast.clone()),
                            consensus: (*etat).consensus.clone(),
                            air: (*etat).air.clone(),
                            loading: false,
                            error: None,
                        });

                        // Le recoupement vient après, et à part : son échec ne
                        // doit pas effacer une prévision déjà affichée. Quand
                        // il arrive, toutes les sources refont la prévision —
                        // la base d'un seul modèle n'était qu'un premier jet.
                        let a_la_parcelle =
                            horloge::maintenant_a_la_parcelle(forecast.utc_offset_seconds);
                        let (recoupement, air) = futures::join!(
                            reseau::recoupement(
                                &endpoints,
                                &parcelle,
                                days,
                                forecast.utc_offset_seconds,
                                a_la_parcelle,
                            ),
                            reseau::air(&endpoints, &parcelle),
                        );
                        let forecast = forecast.recoupee(
                            &recoupement.series,
                            recoupement.observation,
                            recoupement.ciel,
                        );
                        if *par_le_direct.borrow() {
                            return;
                        }
                        etat.set(EtatPrevision {
                            forecast: Some(forecast),
                            consensus: consensus_from_outcomes(&recoupement.outcomes),
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

    // Le direct : à chaque corps poussé par le relais, la prévision est refaite
    // avec ce qu'il a envoyé — la base et les sept modèles d'abord, puis MET
    // Norway, la station, les aéroports et l'air quand ils sont là.
    {
        let etat = etat.clone();
        let par_le_direct = par_le_direct.clone();
        use_effect_with((parcelle.clone(), direct), move |(parcelle, direct)| {
            let point = Some((parcelle.latitude, parcelle.longitude));
            let (Some(base), Some(modeles)) = (&direct.base, &direct.ensemble) else { return };
            if direct.point != point {
                return;
            }
            let maintenant = horloge::maintenant_local();
            let Ok(forecast) = decode_forecast(parcelle.clone(), base, maintenant) else { return };
            let a_la_parcelle = horloge::maintenant_a_la_parcelle(forecast.utc_offset_seconds);
            let texte = |corps: &Option<std::rc::Rc<String>>| {
                corps.as_ref().map(|c| c.as_str().to_owned()).unwrap_or_default()
            };
            let (met, station) = (texte(&direct.met), texte(&direct.station));
            let recoupement = reseau::lire_recoupement(
                reseau::Corps {
                    open_meteo: Some(modeles.as_str()),
                    met: Some(&met),
                    bright_sky: Some(&station),
                    aviation: direct.ciel.as_deref().map(String::as_str),
                },
                (parcelle.latitude, parcelle.longitude),
                forecast.utc_offset_seconds,
                a_la_parcelle,
            );
            let air = direct
                .air
                .as_ref()
                .and_then(|a| klima_api::air::decode_air(a))
                .or_else(|| (*etat).air.clone());
            *par_le_direct.borrow_mut() = true;
            etat.set(EtatPrevision {
                forecast: Some(forecast.recoupee(
                    &recoupement.series,
                    recoupement.observation,
                    recoupement.ciel,
                )),
                consensus: consensus_from_outcomes(&recoupement.outcomes),
                air,
                loading: false,
                error: None,
            });
        });
    }

    let reload = {
        let nonce = nonce.clone();
        Callback::from(move |()| nonce.set(*nonce + 1))
    };

    Prevision { etat: (*etat).clone(), reload }
}
