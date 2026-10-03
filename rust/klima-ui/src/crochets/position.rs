//! Caler la page sur la position de la personne, à la première visite.
//!
//! Miroir de `core/src/ui/useStartPosition.ts`.
//!
//! Ne bloque rien : la parcelle par défaut se charge pendant que le navigateur
//! demande l'autorisation, et bascule quand la position arrive. Attendre la
//! réponse laisserait une page vide derrière la boîte de dialogue, pour un
//! geste que personne n'a demandé.
//!
//! Silencieux en cas d'échec, pour la même raison : un refus n'est pas une
//! erreur à afficher. On garde la parcelle par défaut, et la recherche de
//! commune reste là.

use klima_core::position::{Parcelle, ParcelleOrigin, locates_on_start, parcelle_from_position};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;

#[hook]
pub fn use_start_position(origine: ParcelleOrigin, nom: String, select: Callback<Parcelle>) {
    use_effect_with(origine, move |origine| {
        if !locates_on_start(*origine) {
            return;
        }
        let Some(geo) = web_sys::window().and_then(|w| w.navigator().geolocation().ok()) else {
            return;
        };

        let succes = Closure::<dyn Fn(web_sys::Position)>::new(move |position: web_sys::Position| {
            let coords = position.coords();
            select.emit(parcelle_from_position(&nom, coords.latitude(), coords.longitude()));
        });
        // Refus, position indisponible, délai dépassé : rien à dire.
        let echec = Closure::<dyn Fn(JsValue)>::new(|_| {});

        let options = web_sys::PositionOptions::new();
        options.set_timeout(10_000);
        options.set_maximum_age(5 * 60_000);

        let _ = geo.get_current_position_with_error_callback_and_options(
            succes.as_ref().unchecked_ref(),
            Some(echec.as_ref().unchecked_ref()),
            &options,
        );
        succes.forget();
        echec.forget();
    });
}
