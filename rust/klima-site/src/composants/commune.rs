//! Le petit sélecteur de ville, pour essayer la section sur la sienne.
//!

use klima_core::endpoints::Endpoints;
use klima_core::i18n::params;
use klima_core::position::Parcelle;
use klima_ui::i18n::use_i18n;
use klima_ui::reseau;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub current: Parcelle,
    pub endpoints: Endpoints,
    pub on_select: Callback<Parcelle>,
}

#[function_component]
pub fn ChoixDeCommune(props: &Props) -> Html {
    let i18n = use_i18n();
    let saisie = use_state(String::new);
    let resultats = use_state(Vec::<Parcelle>::new);

    // La recherche n'est pas différée ici comme dans l'application : la liste
    // est courte, et le site n'en montre qu'une à la fois.
    {
        let resultats = resultats.clone();
        let endpoints = props.endpoints.clone();
        use_effect_with((*saisie).clone(), move |texte: &String| {
            let texte = texte.trim().to_owned();
            if texte.chars().count() < 2 {
                resultats.set(Vec::new());
                return;
            }
            wasm_bindgen_futures::spawn_local(async move {
                let trouvees = reseau::search(&endpoints, &texte).await.unwrap_or_default();
                resultats.set(trouvees);
            });
        });
    }

    {
        let resultats = resultats.clone();
        use_effect_with((), move |()| {
            let fermer = Closure::<dyn Fn(JsValue)>::new(move |_| resultats.set(Vec::new()));
            let document = web_sys::window().and_then(|w| w.document());
            if let Some(document) = &document {
                let _ = document
                    .add_event_listener_with_callback("mousedown", fermer.as_ref().unchecked_ref());
            }
            move || {
                if let Some(document) = &document {
                    let _ = document.remove_event_listener_with_callback(
                        "mousedown",
                        fermer.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    let sur_saisie = {
        let saisie = saisie.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(champ) = event.target_dyn_into::<HtmlInputElement>() {
                saisie.set(champ.value());
            }
        })
    };

    let liste: Html = resultats
        .iter()
        .map(|resultat| {
            let onclick = {
                let on_select = props.on_select.clone();
                let saisie = saisie.clone();
                let resultats = resultats.clone();
                let resultat = resultat.clone();
                Callback::from(move |_| {
                    on_select.emit(resultat.clone());
                    saisie.set(String::new());
                    resultats.set(Vec::new());
                })
            };
            let region: Vec<&str> = [resultat.admin.as_deref(), resultat.country.as_deref()]
                .into_iter()
                .flatten()
                .collect();

            html! {
                <li key={format!("{},{}", resultat.latitude, resultat.longitude)}>
                    <button type="button" {onclick}>
                        <span>{ &resultat.name }</span>
                        <span class="commune__admin">{ region.join(", ") }</span>
                    </button>
                </li>
            }
        })
        .collect();

    html! {
        <div class="commune">
            <input
                type="search"
                class="commune__input"
                value={(*saisie).clone()}
                oninput={sur_saisie}
                placeholder={i18n.with(
                    "search.placeholder",
                    &params([("commune", props.current.name.as_str().into())]),
                )}
                aria-label={i18n.t("search.label")}
            />
            if !resultats.is_empty() {
                <ul class="commune__results">{ liste }</ul>
            }
        </div>
    }
}
