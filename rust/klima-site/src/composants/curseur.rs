//! Le curseur qui suit, et ce qui lui répond.
//!
//! Un anneau suit la souris avec un temps de retard et s'ouvre sur ce qui se
//! clique ; les boutons marqués `data-aimant` se penchent vers elle quand
//! elle approche ; les téléphones marqués `data-incline` pivotent pour la
//! regarder ; et les cartes marquées `data-lueur` s'éclairent là où elle
//! passe. C'est un ornement : le pointeur du système reste là, et rien de
//! tout cela n'existe au doigt ni pour qui demande moins de mouvement.

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

/// Jusqu'où, autour d'un téléphone, la souris le fait pivoter (px), et de
/// combien au plus (degrés).
const PORTEE_INCLINE: f64 = 160.0;
const INCLINE_MAX: f64 = 9.0;

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

/// Le pivot d'un appareil vers la souris : `(rotateX, rotateY)` en degrés,
/// ou `None` si elle est trop loin. Le haut se penche vers elle quand elle
/// est en haut, le côté quand elle est sur le côté.
fn incline(x: f64, y: f64, gauche: f64, haut: f64, largeur: f64, hauteur: f64) -> Option<(f64, f64)> {
    let (dx, dy) = (x - (gauche + largeur / 2.0), y - (haut + hauteur / 2.0));
    let (mx, my) = (largeur / 2.0 + PORTEE_INCLINE, hauteur / 2.0 + PORTEE_INCLINE);
    if dx.abs() > mx || dy.abs() > my {
        return None;
    }
    Some((-dy / my * INCLINE_MAX, dx / mx * INCLINE_MAX))
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
                        // La lueur : la carte survolée sait où est la souris.
                        if let Some(carte) = e
                            .target()
                            .and_then(|t| t.dyn_into::<Element>().ok())
                            .and_then(|el| el.closest("[data-lueur]").ok().flatten())
                            .and_then(|el| el.dyn_into::<HtmlElement>().ok())
                        {
                            let r = carte.get_bounding_client_rect();
                            let style = carte.style();
                            let _ = style.set_property("--mx", &format!("{:.0}px", x - r.left()));
                            let _ = style.set_property("--my", &format!("{:.0}px", y - r.top()));
                        }
                        let Some(document) = &document else { return };
                        // Les aimants.
                        if let Ok(aimants) = document.query_selector_all("[data-aimant]") {
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
                        }
                        // Les téléphones qui se tournent vers elle.
                        if let Ok(appareils) = document.query_selector_all("[data-incline]") {
                            for i in 0..appareils.length() {
                                let Some(el) = appareils.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) else { continue };
                                let r = el.get_bounding_client_rect();
                                let style = el.style();
                                match incline(x, y, r.left(), r.top(), r.width(), r.height()) {
                                    Some((rx, ry)) => {
                                        let _ = style.set_property("--rx", &format!("{rx:.2}deg"));
                                        let _ = style.set_property("--ry", &format!("{ry:.2}deg"));
                                    }
                                    None => {
                                        let _ = style.remove_property("--rx");
                                        let _ = style.remove_property("--ry");
                                    }
                                }
                            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_appareil_se_tourne_vers_la_souris() {
        // Au centre, il ne bouge pas.
        assert_eq!(incline(150.0, 300.0, 100.0, 100.0, 100.0, 400.0), Some((0.0, 0.0)));
        // La souris à droite : il tourne sa face vers la droite.
        let (rx, ry) = incline(250.0, 300.0, 100.0, 100.0, 100.0, 400.0).unwrap();
        assert!(rx.abs() < 1e-9 && ry > 0.0);
        // En haut : le haut se penche vers elle.
        let (rx, _) = incline(150.0, 0.0, 100.0, 100.0, 100.0, 400.0).unwrap();
        assert!(rx > 0.0);
        // Jamais plus que le maximum, même au bord de la portée.
        let (rx, ry) = incline(150.0 + 50.0 + PORTEE_INCLINE, 300.0 - 200.0 - PORTEE_INCLINE, 100.0, 100.0, 100.0, 400.0).unwrap();
        assert!(rx <= INCLINE_MAX + 1e-9 && ry <= INCLINE_MAX + 1e-9);
    }

    #[test]
    fn loin_de_lui_il_revient_droit() {
        assert_eq!(incline(900.0, 300.0, 100.0, 100.0, 100.0, 400.0), None);
        assert_eq!(incline(150.0, 900.0, 100.0, 100.0, 100.0, 400.0), None);
    }
}
