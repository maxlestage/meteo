//! Le curseur qui suit, et les boutons qui l'attirent.
//!
//! Un anneau suit la souris avec un temps de retard et s'ouvre sur ce qui se
//! clique ; les boutons marqués `data-aimant` se penchent vers elle quand
//! elle approche. C'est un ornement : le pointeur du système reste là, et
//! rien de tout cela n'existe au doigt ni pour qui demande moins de
//! mouvement.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, HtmlElement, PointerEvent};
use yew::prelude::*;

/// Jusqu'où, autour d'un bouton, la souris l'attire (px), et de combien il
/// la suit (part de l'écart).
const PORTEE: f64 = 36.0;
const ATTRACTION: f64 = 0.3;

/// Une boucle d'images : le rappel se redemande lui-même tant qu'il vit.
type Boucle = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

fn souris_fine() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(hover: hover) and (pointer: fine) and (prefers-reduced-motion: no-preference)")
                .ok()
                .flatten()
        })
        .is_some_and(|m| m.matches())
}

#[function_component]
pub fn Curseur() -> Html {
    let anneau = use_node_ref();
    let actif = use_state(|| false);

    {
        let anneau = anneau.clone();
        let actif = actif.clone();
        use_effect_with((), move |()| {
            let mut nettoyage: Option<Box<dyn FnOnce()>> = None;
            if let (true, Some(fenetre)) = (souris_fine(), web_sys::window()) {
                actif.set(true);
                let cible = Rc::new(Cell::new((-100.0f64, -100.0f64)));
                let ici = Rc::new(Cell::new((-100.0f64, -100.0f64)));
                let vivant = Rc::new(Cell::new(true));

                let au_mouvement = {
                    let cible = cible.clone();
                    let anneau = anneau.clone();
                    let document = fenetre.document();
                    Closure::<dyn Fn(PointerEvent)>::new(move |e: PointerEvent| {
                        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
                        cible.set((x, y));
                        let sur_lien = e
                            .target()
                            .and_then(|t| t.dyn_into::<Element>().ok())
                            .and_then(|el| el.closest("a, button, input, [role=button]").ok().flatten())
                            .is_some();
                        if let Some(a) = anneau.cast::<Element>() {
                            let _ = a.class_list().toggle_with_force("curseur--lien", sur_lien);
                        }
                        // Les aimants.
                        let Some(document) = &document else { return };
                        let Ok(aimants) = document.query_selector_all("[data-aimant]") else { return };
                        for i in 0..aimants.length() {
                            let Some(el) = aimants.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) else { continue };
                            let r = el.get_bounding_client_rect();
                            let (cx, cy) = (r.left() + r.width() / 2.0, r.top() + r.height() / 2.0);
                            let proche = x > r.left() - PORTEE && x < r.right() + PORTEE && y > r.top() - PORTEE && y < r.bottom() + PORTEE;
                            let decalage = if proche {
                                format!("translate({:.1}px, {:.1}px)", (x - cx) * ATTRACTION, (y - cy) * ATTRACTION)
                            } else {
                                String::new()
                            };
                            let _ = el.style().set_property("transform", &decalage);
                        }
                    })
                };

                // L'anneau rattrape la souris, image après image.
                let boucle: Boucle = Rc::new(RefCell::new(None));
                let suite = boucle.clone();
                let (rappel_vivant, rappel_fenetre) = (vivant.clone(), fenetre.clone());
                *boucle.borrow_mut() = Some(Closure::new(move || {
                    if !rappel_vivant.get() {
                        return;
                    }
                    let ((cx, cy), (x, y)) = (cible.get(), ici.get());
                    let (nx, ny) = (x + (cx - x) * 0.2, y + (cy - y) * 0.2);
                    ici.set((nx, ny));
                    if let Some(a) = anneau.cast::<HtmlElement>() {
                        let _ = a.style().set_property("transform", &format!("translate3d({nx:.1}px, {ny:.1}px, 0)"));
                    }
                    if let Some(c) = suite.borrow().as_ref() {
                        let _ = rappel_fenetre.request_animation_frame(c.as_ref().unchecked_ref());
                    }
                }));
                if let Some(c) = boucle.borrow().as_ref() {
                    let _ = fenetre.request_animation_frame(c.as_ref().unchecked_ref());
                }

                let rappel = au_mouvement.as_ref().unchecked_ref::<js_sys::Function>().clone();
                let _ = fenetre.add_event_listener_with_callback("pointermove", &rappel);
                nettoyage = Some(Box::new(move || {
                    vivant.set(false);
                    let _ = fenetre.remove_event_listener_with_callback("pointermove", &rappel);
                    drop(au_mouvement);
                    boucle.borrow_mut().take();
                }));
            }
            move || {
                if let Some(nettoyer) = nettoyage {
                    nettoyer();
                }
            }
        });
    }

    html! {
        <div class={classes!("curseur", (*actif).then_some("curseur--actif"))} ref={anneau} aria-hidden="true">
            <span></span>
        </div>
    }
}
