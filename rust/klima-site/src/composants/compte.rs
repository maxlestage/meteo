//! Les chiffres qui roulent.
//!
//! Un chiffre arrive en roulant depuis zéro la première fois qu'on le voit,
//! puis d'une valeur à l'autre quand elle change — une autre ville, une
//! prévision qui se met à jour. Il ne roule qu'à l'écran : hors de vue, il
//! attend, et affiche en attendant sa vraie valeur, jamais un zéro de départ.
//! Pour qui demande moins de mouvement, ou sans moyen de savoir ce qui est à
//! l'écran, il ne roule pas du tout.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use klima_core::format::Formats;
use klima_ui::i18n::use_i18n;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::Element;
use yew::prelude::*;

use crate::composants::mouvement::{moins_de_mouvement, quand_visible};

/// Le temps d'un roulement (ms).
const DUREE: f64 = 1_100.0;

/// Comment le chiffre s'écrit, dans la langue de la page.
#[derive(Clone, Copy, PartialEq)]
pub enum Forme {
    /// « 16° ».
    Temperature,
    /// « 4,8 mm » : l'unité et le nombre de décimales.
    Unite(&'static str, u32),
    /// « 6 » : le nombre de décimales.
    Decimale(u32),
}

impl Forme {
    fn ecrire(self, f: Formats, valeur: f64) -> String {
        match self {
            Forme::Temperature => f.temperature(valeur),
            Forme::Unite(unite, decimales) => f.unit(valeur, unite, decimales),
            Forme::Decimale(decimales) => f.decimal(valeur, decimales),
        }
    }
}

/// Un départ lent, une arrivée douce.
fn adoucir(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub valeur: f64,
    pub forme: Forme,
}

type Boucle = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;

#[function_component]
pub fn Compte(props: &Props) -> Html {
    let i18n = use_i18n();
    let noeud = use_node_ref();
    // Ce qui s'affiche pendant que le chiffre roule ; `None` : la valeur.
    let roule = use_state_eq(|| None::<f64>);
    // D'où part le prochain roulement : ce qui est affiché. Zéro au début.
    let depart = use_mut_ref(|| 0.0f64);

    {
        let (noeud, roule, depart) = (noeud.clone(), roule.clone(), depart.clone());
        use_effect_with(props.valeur, move |&cible| {
            let vivant = Rc::new(Cell::new(true));
            let boucle: Boucle = Rc::new(RefCell::new(None));
            let mut guet = None;

            let de = *depart.borrow();
            if let (false, false, Some(element)) =
                (moins_de_mouvement(), de == cible, noeud.cast::<Element>())
            {
                let (vivant, boucle) = (vivant.clone(), boucle.clone());
                let (roule, depart) = (roule.clone(), depart.clone());
                guet = quand_visible(&element, move || {
                    let Some(fenetre) = web_sys::window() else { return };
                    let debut = Rc::new(Cell::new(None::<f64>));
                    let suite = boucle.clone();
                    let rappel_fenetre = fenetre.clone();
                    // La boucle se défait elle-même, à sa dernière image ou à
                    // la première qui suit son arrêt : la lâcher d'ailleurs,
                    // une image encore demandée, l'appellerait après coup.
                    *boucle.borrow_mut() = Some(Closure::new(move |ms: f64| {
                        if !vivant.get() {
                            suite.borrow_mut().take();
                            return;
                        }
                        let t0 = debut.get().unwrap_or(ms);
                        debut.set(Some(t0));
                        let t = ((ms - t0) / DUREE).clamp(0.0, 1.0);
                        let ici = de + (cible - de) * adoucir(t);
                        *depart.borrow_mut() = ici;
                        if t < 1.0 {
                            roule.set(Some(ici));
                            if let Some(c) = suite.borrow().as_ref() {
                                let _ = rappel_fenetre.request_animation_frame(c.as_ref().unchecked_ref());
                            }
                        } else {
                            roule.set(None);
                            suite.borrow_mut().take();
                        }
                    }));
                    if let Some(c) = boucle.borrow().as_ref() {
                        let _ = fenetre.request_animation_frame(c.as_ref().unchecked_ref());
                    }
                });
            }
            // Sans roulement possible, la valeur est affichée telle quelle,
            // et c'est d'elle que partira le prochain.
            if guet.is_none() {
                *depart.borrow_mut() = cible;
                roule.set(None);
            }

            move || {
                vivant.set(false);
                drop(guet);
            }
        });
    }

    let valeur = (*roule).unwrap_or(props.valeur);
    html! {
        <span class="compte" ref={noeud}>{ props.forme.ecrire(i18n.f(), valeur) }</span>
    }
}
