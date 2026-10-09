//! L'écran d'entrée s'ouvre quand la page est montée.
//!
//! Le signe se trace en CSS dès le premier affichage, avant même le
//! WebAssembly (`index.html`). Une fois la page montée — et pas avant que le
//! tracé ait eu le temps de finir —, la classe `ouverte` posée sur la racine
//! lève l'écran et fait monter le titre. Si le WebAssembly ne vient jamais,
//! l'écran se lève de lui-même au bout de quelques secondes.

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

/// Le temps du tracé et du compte (ms) : l'écran ne se lève pas avant.
const TRACE_MS: f64 = 1_350.0;

#[hook]
pub fn use_porte() {
    use_effect_with((), |()| {
        if let Some(fenetre) = web_sys::window() {
            let moins = fenetre
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
                .is_some_and(|m| m.matches());
            let ecoule = fenetre.performance().map(|p| p.now()).unwrap_or(TRACE_MS);
            let reste = if moins { 0.0 } else { (TRACE_MS - ecoule).max(0.0) };
            let ouvrir = Closure::once_into_js(|| {
                if let Some(racine) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()) {
                    let _ = racine.class_list().add_1("ouverte");
                }
            });
            let _ = fenetre.set_timeout_with_callback_and_timeout_and_arguments_0(ouvrir.unchecked_ref(), reste as i32);
        }
    });
}
