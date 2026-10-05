//! Le guetteur relit le ciel tous les quarts d'heure.
//!
//! Une lecture à l'ouverture, puis une à chaque quart, une minute après son
//! début (`klima_core::veille::prochaine_lecture`). Une horloge bat toutes les
//! trente secondes : elle fait avancer « maintenant » dans la série déjà lue —
//! le quart en cours change même sans nouvelle lecture — et déclenche la
//! relecture quand l'heure est passée. Un onglet laissé en arrière-plan,
//! dont le navigateur endort les minuteurs, relit donc dès qu'on y revient.

use klima_api::veille::Quarts;
use klima_core::endpoints::Endpoints;
use klima_core::position::Parcelle;
use klima_core::veille::prochaine_lecture;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

use crate::horloge;
use crate::reseau;

/// Battement de l'horloge du guetteur.
const BATTEMENT_MS: i32 = 30_000;

#[derive(Clone, PartialEq)]
pub struct EtatVeille {
    /// La dernière série lue ; absente tant que rien n'est arrivé, ou si le
    /// service n'a pas de série au quart d'heure pour cette ville.
    pub quarts: Option<Quarts>,
    /// Une lecture est en cours.
    pub loading: bool,
    /// Quand la dernière lecture a abouti (ou échoué), en temps universel.
    pub lu_a: Option<i64>,
    /// Maintenant, en temps universel, au dernier battement.
    pub maintenant: i64,
}

impl EtatVeille {
    /// Maintenant, à l'heure de la ville — l'horloge des séries.
    pub fn maintenant_a_la_ville(&self) -> Option<i64> {
        self.quarts.as_ref().map(|q| self.maintenant + q.utc_offset_seconds * 1000)
    }
}

#[hook]
pub fn use_veille(parcelle: Parcelle, endpoints: Endpoints) -> EtatVeille {
    let quarts = use_state(|| None::<(Parcelle, Quarts)>);
    let loading = use_state(|| true);
    let lu_a = use_state(|| None::<i64>);
    let maintenant = use_state(horloge::maintenant_utc);
    let nonce = use_state(|| 0_u32);

    // La lecture : à l'ouverture, au changement de ville, et à chaque quart.
    {
        let quarts = quarts.clone();
        let loading = loading.clone();
        let lu_a = lu_a.clone();
        use_effect_with((parcelle.clone(), *nonce), move |(parcelle, _)| {
            let parcelle = parcelle.clone();
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let lu = reseau::quarts(&endpoints, &parcelle).await;
                // Un échec de relecture garde la série d'avant : elle reste
                // vraie pour les quarts qu'elle couvre encore. Celle d'une
                // autre ville, non — d'où la ville gardée avec la série.
                match lu {
                    Some(lu) => quarts.set(Some((parcelle, lu))),
                    None if (*quarts).as_ref().is_some_and(|(p, _)| *p != parcelle) => {
                        quarts.set(None)
                    }
                    None => {}
                }
                lu_a.set(Some(horloge::maintenant_utc()));
                loading.set(false);
            });
        });
    }

    // L'horloge.
    {
        let maintenant = maintenant.clone();
        use_effect_with((), move |()| {
            let rappel = Closure::<dyn Fn()>::new(move || maintenant.set(horloge::maintenant_utc()));
            let fenetre = web_sys::window();
            let minuteur = fenetre.as_ref().and_then(|f| {
                f.set_interval_with_callback_and_timeout_and_arguments_0(
                    rappel.as_ref().unchecked_ref(),
                    BATTEMENT_MS,
                )
                .ok()
            });
            move || {
                if let (Some(f), Some(id)) = (fenetre, minuteur) {
                    f.clear_interval_with_handle(id);
                }
                drop(rappel);
            }
        });
    }

    // La relecture, quand l'heure est passée.
    {
        let nonce = nonce.clone();
        let en_cours = *loading;
        use_effect_with((*maintenant, *lu_a), move |(maintenant, lu_a)| {
            if let Some(lu_a) = lu_a {
                if !en_cours && *maintenant >= prochaine_lecture(*lu_a) {
                    nonce.set(*nonce + 1);
                }
            }
        });
    }

    EtatVeille {
        quarts: (*quarts)
            .as_ref()
            .filter(|(p, _)| *p == parcelle)
            .map(|(_, q)| q.clone()),
        loading: *loading,
        lu_a: *lu_a,
        maintenant: *maintenant,
    }
}
