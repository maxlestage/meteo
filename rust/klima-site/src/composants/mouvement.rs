//! Ce qui entre quand on y arrive.
//!
//! Les blocs marqués `data-apparait` entrent à l'écran au moment où on les
//! atteint : un seul observateur les guette tous et pose `data-vu` sur ceux
//! qu'il a vus. La feuille de style fait le reste — ce qui entre, comment, et
//! dans quel ordre. Tant qu'un bloc n'a pas été vu, ses animations attendent,
//! arrêtées sur leur première image.
//!
//! Le contenu part **visible** : la classe `anime`, sans laquelle rien n'est
//! caché, n'est posée sur la racine que si l'on sait pouvoir le ramener —
//! avec `IntersectionObserver`, et quand le système ne demande pas moins de
//! mouvement. Un texte caché par une animation qui ne se déclenche pas est un
//! texte perdu. Un bloc monté plus tard (une réponse qui arrive, une ville
//! qu'on change) est guetté dès qu'il paraît.
//!
//! On marque par un attribut et non par une classe : Yew réécrit l'attribut
//! `class` quand il change, et emporterait une classe posée à côté de lui ;
//! un attribut qu'il ne connaît pas, il le laisse.

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{
    Document, Element, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit,
    MutationObserver, MutationObserverInit,
};
use yew::prelude::*;

/// Le bas de l'écran est rogné d'un dixième : un bloc entre quand il est
/// vraiment à l'écran, pas quand il en touche le bord.
const MARGE: &str = "0px 0px -10% 0px";

/// Vrai si le système demande moins de mouvement.
pub fn moins_de_mouvement() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten())
        .is_some_and(|m| m.matches())
}

type RappelDeVue = Closure<dyn FnMut(js_sys::Array, IntersectionObserver)>;

/// Un observateur et son rappel, défaits ensemble quand on les lâche.
pub struct Guet {
    observateur: IntersectionObserver,
    _rappel: RappelDeVue,
}

impl Drop for Guet {
    fn drop(&mut self) {
        self.observateur.disconnect();
    }
}

/// Un observateur qui appelle `vu` pour chaque élément arrivé à l'écran, et
/// cesse aussitôt de le guetter.
fn observateur(mut vu: impl FnMut(&Element) + 'static) -> Option<Guet> {
    let rappel: RappelDeVue = Closure::new(move |entrees: js_sys::Array, obs: IntersectionObserver| {
        for entree in entrees.iter() {
            let entree: IntersectionObserverEntry = entree.unchecked_into();
            if entree.is_intersecting() {
                let cible = entree.target();
                obs.unobserve(&cible);
                vu(&cible);
            }
        }
    });
    let options = IntersectionObserverInit::new();
    options.set_root_margin(MARGE);
    let observateur =
        IntersectionObserver::new_with_options(rappel.as_ref().unchecked_ref(), &options).ok()?;
    Some(Guet { observateur, _rappel: rappel })
}

/// Appelle `action` une fois, quand `element` arrive à l'écran — tout de
/// suite s'il y est déjà. `None` si le navigateur ne sait pas guetter : à
/// l'appelant de faire sans.
pub fn quand_visible(element: &Element, action: impl FnOnce() + 'static) -> Option<Guet> {
    let mut action = Some(action);
    let guet = observateur(move |_| {
        if let Some(action) = action.take() {
            action();
        }
    })?;
    guet.observateur.observe(element);
    Some(guet)
}

/// Pose `data-vu` sur les blocs qui arrivent à l'écran.
struct Installation {
    _guet: Guet,
    mutations: MutationObserver,
    _rappel: Closure<dyn FnMut()>,
}

impl Drop for Installation {
    fn drop(&mut self) {
        self.mutations.disconnect();
        if let Some(racine) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let _ = racine.class_list().remove_1("anime");
        }
    }
}

/// Fait guetter tous les blocs qui n'ont pas encore été vus. Guetter deux
/// fois le même élément ne coûte rien.
fn guetter(document: &Document, observateur: &IntersectionObserver) {
    let Ok(blocs) = document.query_selector_all("[data-apparait]:not([data-vu])") else {
        return;
    };
    for i in 0..blocs.length() {
        if let Some(bloc) = blocs.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
            observateur.observe(&bloc);
        }
    }
}

fn installer() -> Option<Installation> {
    if moins_de_mouvement() {
        return None;
    }
    let document = web_sys::window()?.document()?;
    let racine = document.document_element()?;

    let guet = observateur(|bloc| {
        let _ = bloc.set_attribute("data-vu", "");
    })?;
    guetter(&document, &guet.observateur);

    // Ce qui se monte plus tard est guetté à son tour.
    let rappel: Closure<dyn FnMut()> = {
        let (document, observateur) = (document.clone(), guet.observateur.clone());
        Closure::new(move || guetter(&document, &observateur))
    };
    let mutations = MutationObserver::new(rappel.as_ref().unchecked_ref()).ok()?;
    let options = MutationObserverInit::new();
    options.set_child_list(true);
    options.set_subtree(true);
    mutations.observe_with_options(&racine, &options).ok()?;

    // En dernier : on ne cache rien tant qu'on n'est pas sûr de le ramener.
    racine.class_list().add_1("anime").ok()?;
    Some(Installation { _guet: guet, mutations, _rappel: rappel })
}

/// Met la page en mouvement. À monter une fois, après le reste.
#[function_component]
pub fn Mouvement() -> Html {
    use_effect_with((), |()| {
        let installation = installer();
        move || drop(installation)
    });
    html! {}
}
