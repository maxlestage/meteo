//! La parcelle consultée, tenue dans l'adresse.
//!
//! Miroir de `core/src/ui/useParcelleInUrl.ts`.
//!
//! Sans cela, choisir une commune ne laisse aucune trace : le bouton retour du
//! navigateur ne défait rien, recharger la page perd le choix, et envoyer
//! l'adresse à quelqu'un lui montre une autre parcelle que la sienne. Trois
//! défauts pour une seule cause — l'état n'était pas dans l'URL.
//!
//! L'ordre de préférence au démarrage : ce que dit l'adresse — elle est
//! explicite, et c'est elle qu'on a partagée —, puis la dernière parcelle
//! consultée, puis celle par défaut.

use klima_api::parcelle_url::{parcelle_from_query, same_parcelle, url_for_parcelle};
use klima_core::position::{Parcelle, ParcelleOrigin};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;

use crate::storage;

/// Clé de mémorisation de la dernière parcelle consultée.
const STORAGE_KEY: &str = "klima.parcelle";

pub struct ParcelleEnUrl {
    pub parcelle: Parcelle,
    pub select: Callback<Parcelle>,
    /// D'où vient celle qu'on affiche. C'est elle qui dit s'il y a un choix
    /// derrière, et donc s'il est permis d'aller chercher la position.
    pub origine: ParcelleOrigin,
}

#[hook]
pub fn use_parcelle(defaut: Parcelle) -> ParcelleEnUrl {
    // L'origine est calculée une seule fois, avec la parcelle de départ.
    let depart = use_state(|| {
        if let Some(adresse) = depuis_adresse() {
            return (adresse, ParcelleOrigin::Adresse);
        }
        if let Some(memoire) = depuis_memoire() {
            return (memoire, ParcelleOrigin::Memoire);
        }
        (defaut.clone(), ParcelleOrigin::Defaut)
    });

    let parcelle = use_state(|| depart.0.clone());

    {
        let parcelle_courante = (*parcelle).clone();
        use_effect_with(parcelle_courante, |parcelle| {
            storage::set(STORAGE_KEY, &klima_api::parcelle_url::parcelle_to_query(parcelle));
        });
    }

    // Le bouton retour du navigateur remonte ici : l'adresse a changé sans que
    // l'application le sache, on se remet sur ce qu'elle dit.
    {
        let parcelle = parcelle.clone();
        let defaut = defaut.clone();
        use_effect_with((), move |()| {
            let retour = Closure::<dyn Fn(JsValue)>::new(move |_| {
                parcelle.set(depuis_adresse().unwrap_or_else(|| defaut.clone()));
            });
            let window = web_sys::window();
            if let Some(window) = &window {
                let _ = window.add_event_listener_with_callback(
                    "popstate",
                    retour.as_ref().unchecked_ref(),
                );
            }
            move || {
                if let Some(window) = &window {
                    let _ = window.remove_event_listener_with_callback(
                        "popstate",
                        retour.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    let select = {
        let parcelle = parcelle.clone();
        Callback::from(move |suivante: Parcelle| {
            // Rechoisir la même commune n'a pas à créer une étape
            // d'historique : sinon il faudrait appuyer trois fois sur retour
            // pour rien.
            if same_parcelle(Some(&parcelle), Some(&suivante)) {
                return;
            }
            pousser_dans_lhistorique(&suivante);
            parcelle.set(suivante);
        })
    };

    ParcelleEnUrl { parcelle: (*parcelle).clone(), select, origine: depart.1 }
}

/// La parcelle que désigne l'adresse, si elle en désigne une de valide.
fn depuis_adresse() -> Option<Parcelle> {
    let search = web_sys::window()?.location().search().ok()?;
    parcelle_from_query(&search)
}

fn depuis_memoire() -> Option<Parcelle> {
    parcelle_from_query(&storage::get(STORAGE_KEY)?)
}

fn pousser_dans_lhistorique(parcelle: &Parcelle) {
    let Some(window) = web_sys::window() else { return };
    let Ok(href) = window.location().href() else { return };
    let Ok(historique) = window.history() else { return };

    let _ = historique.push_state_with_url(
        &JsValue::NULL,
        "",
        Some(&url_for_parcelle(&href, parcelle)),
    );
}
