//! Faire apparaître un élément quand il entre dans la fenêtre.
//!
//! Le contenu part **visible** : il n'est masqué que dans l'instant qui
//! précède le premier affichage, et seulement si l'on sait pouvoir le ramener.
//! Sans `IntersectionObserver`, ou lorsque le système demande moins de
//! mouvement, la section reste simplement lisible — un texte caché par une
//! animation qui ne se déclenche pas est un texte perdu.

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, IntersectionObserver, IntersectionObserverInit};
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    /// Rien ne sera animé : le contenu reste tel quel.
    Simple,
    Cachee,
    Montree,
}

pub struct Apparition {
    pub node: NodeRef,
    pub class: &'static str,
}

#[hook]
pub fn use_apparition() -> Apparition {
    let node = use_node_ref();
    let phase = use_state(|| Phase::Simple);

    {
        let node = node.clone();
        let phase = phase.clone();
        use_effect_with((), move |()| {
            let Some(element) = node.cast::<Element>() else {
                return;
            };
            if moins_de_mouvement() {
                return;
            }

            phase.set(Phase::Cachee);

            let rappel = Closure::<dyn Fn(js_sys::Array)>::new(move |entrees: js_sys::Array| {
                let visible = entrees.iter().any(|entree| {
                    js_sys::Reflect::get(&entree, &JsValue::from_str("isIntersecting"))
                        .ok()
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                });
                if visible {
                    phase.set(Phase::Montree);
                }
            });

            let options = IntersectionObserverInit::new();
            // On déclenche un peu avant le bas de l'écran : l'animation a le
            // temps de se jouer pendant qu'on fait défiler.
            options.set_root_margin("0px 0px -12% 0px");

            if let Ok(observateur) = IntersectionObserver::new_with_options(
                rappel.as_ref().unchecked_ref(),
                &options,
            ) {
                observateur.observe(&element);
                rappel.forget();
            }
        });
    }

    Apparition {
        node,
        class: match *phase {
            Phase::Simple => "",
            Phase::Cachee => "reveal",
            Phase::Montree => "reveal is-visible",
        },
    }
}

/// Vrai si le système demande moins de mouvement.
fn moins_de_mouvement() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten())
        .map(|m| m.matches())
        .unwrap_or(false)
}
