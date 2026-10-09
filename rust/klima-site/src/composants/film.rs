//! Le film de grains : sous l'ouverture, la page se fige sur un ciel de nuit
//! et des milliers de grains y dessinent, au fil du défilement, ce que Klima
//! regarde — la pluie qui vient, les heures, le radar, les neuf sources qui
//! votent, ce qu'il faut emporter, le soleil, les appareils. Une phrase par
//! station, en bas de l'écran, là où rien ne se dessine.
//!
//! C'est le défilement qui fait avancer le film : rien ne part tout seul, et
//! l'on peut revenir en arrière. Sans WebGL 2, les phrases restent, en liste.
//! Pour qui demande moins de mouvement, les formes changent toujours avec le
//! défilement — c'est le lecteur qui les fait changer —, mais rien ne tombe,
//! ne tourne ni ne frémit de soi-même.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use klima_ui::i18n::use_i18n;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, HtmlCanvasElement};
use yew::prelude::*;

use crate::film::formes::STATIONS;
use crate::film::moteur::Moteur;
use crate::film::{ligne, moment};

/// Une boucle d'images : le rappel se redemande lui-même tant qu'il vit.
type Boucle = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;

fn moins_de_mouvement() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten())
        .is_some_and(|m| m.matches())
}

/// Une phrase coupée en mots, pour qu'ils montent l'un après l'autre.
pub fn mots(texte: &str) -> Html {
    let n = texte.split_whitespace().count();
    texte
        .split_whitespace()
        .enumerate()
        .map(|(i, mot)| {
            html! {
                <>
                    <span class="mot" style={format!("--i: {i}")}>{ mot.to_owned() }</span>
                    { if i + 1 < n { " " } else { "" } }
                </>
            }
        })
        .collect()
}

#[function_component]
pub fn Film() -> Html {
    let i18n = use_i18n();
    let section = use_node_ref();
    let toile = use_node_ref();
    let active = use_state(|| 0usize);
    let simple = use_state(|| false);

    {
        let section = section.clone();
        let toile = toile.clone();
        let active = active.clone();
        let simple = simple.clone();
        use_effect_with((), move |()| {
            let mut nettoyage: Option<Box<dyn FnOnce()>> = None;
            let fenetre = web_sys::window();
            let elements = (section.cast::<Element>(), toile.cast::<HtmlCanvasElement>());
            if let (Some(fenetre), (Some(section), Some(toile))) = (fenetre, elements) {
                let etroit = fenetre.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1200.0) < 700.0;
                match Moteur::nouveau(&toile, if etroit { 7_000 } else { 16_000 }) {
                    None => simple.set(true),
                    Some(moteur) => {
                        let mouvement = !moins_de_mouvement();
                        let vivant = Rc::new(Cell::new(true));
                        let derniere = Rc::new(Cell::new(usize::MAX));
                        let boucle: Boucle = Rc::new(RefCell::new(None));
                        let suite = boucle.clone();
                        let rappel_vivant = vivant.clone();
                        let rappel_fenetre = fenetre.clone();
                        *boucle.borrow_mut() = Some(Closure::new(move |ms: f64| {
                            if !rappel_vivant.get() {
                                return;
                            }
                            let haut = rappel_fenetre.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
                            let cadre = section.get_bounding_client_rect();
                            // On ne dessine que ce qui se voit.
                            if cadre.bottom() > 0.0 && cadre.top() < haut {
                                let course = (cadre.height() - haut).max(1.0);
                                let avancee = (-cadre.top() / course).clamp(0.0, 1.0) as f32;
                                let dpr = rappel_fenetre.device_pixel_ratio().min(2.0) as f32;
                                let (l, h) = (toile.client_width() as f32, toile.client_height() as f32);
                                let (lp, hp) = ((l * dpr) as u32, (h * dpr) as u32);
                                if toile.width() != lp || toile.height() != hp {
                                    toile.set_width(lp);
                                    toile.set_height(hp);
                                }
                                let (de, vers, t) = moment(avancee, STATIONS.len());
                                moteur.dessiner(de, vers, t, (ms / 1000.0) as f32, l, h, dpr, mouvement);
                                let i = ligne(avancee, STATIONS.len());
                                if derniere.replace(i) != i {
                                    active.set(i);
                                }
                            }
                            if let Some(c) = suite.borrow().as_ref() {
                                let _ = rappel_fenetre.request_animation_frame(c.as_ref().unchecked_ref());
                            }
                        }));
                        if let Some(c) = boucle.borrow().as_ref() {
                            let _ = fenetre.request_animation_frame(c.as_ref().unchecked_ref());
                        }
                        nettoyage = Some(Box::new(move || {
                            vivant.set(false);
                            boucle.borrow_mut().take();
                        }));
                    }
                }
            }
            move || {
                if let Some(nettoyer) = nettoyage {
                    nettoyer();
                }
            }
        });
    }

    let lignes: Html = STATIONS
        .iter()
        .enumerate()
        .map(|(i, nom)| {
            let classe = classes!("film__ligne", (i == *active).then_some("film__ligne--active"));
            let texte = mots(&i18n.t(&format!("film.{nom}")));
            if i + 1 == STATIONS.len() {
                html! {
                    <div class={classe}>
                        <p>{ texte }</p>
                        <a class="button film__bouton" href="./app/" data-aimant="">{ i18n.t("app.open") }</a>
                    </div>
                }
            } else {
                html! { <p class={classe}>{ texte }</p> }
            }
        })
        .collect();

    html! {
        <section
            class={classes!("film", simple.then_some("film--simple"))}
            ref={section}
            style={format!("--stations: {}", STATIONS.len())}
            aria-label={i18n.t("film.label")}
        >
            <div class="film__scene">
                <canvas class="film__grains" ref={toile} aria-hidden="true"></canvas>
                <div class="film__lignes">{ lignes }</div>
                <div class="film__defiler" aria-hidden="true"><i></i></div>
            </div>
        </section>
    }
}
