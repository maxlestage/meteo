//! La barre de recherche de commune, avec repli sur la géolocalisation.
//!
//! Miroir de `web/src/components/ParcelleSearch.tsx`.
//!
//! Deux attentions qui ne se voient pas : la recherche est différée d'un quart
//! de seconde — on laisse finir de taper, et une requête par lettre serait
//! quatre requêtes pour « Reims » —, et la liste se referme quand on clique
//! ailleurs.

use gloo_timers::callback::Timeout;
use klima_core::endpoints::Endpoints;
use klima_core::i18n::params;
use klima_core::position::parcelle_from_position;
use klima_core::position::Parcelle;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::i18n::use_i18n;
use crate::reseau;

/// Le temps qu'on laisse à la frappe avant d'interroger.
const ATTENTE_MS: u32 = 250;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub current: Parcelle,
    pub endpoints: Endpoints,
    pub on_select: Callback<Parcelle>,
}

#[function_component]
pub fn Recherche(props: &Props) -> Html {
    let i18n = use_i18n();
    let saisie = use_state(String::new);
    let resultats = use_state(Vec::<Parcelle>::new);
    let localisation = use_state(|| false);
    let erreur = use_state(|| Option::<String>::None);

    // Recherche différée : le minuteur est annulé par son `Drop` dès que la
    // saisie change, donc une frappe rapide ne lance qu'un appel.
    {
        let resultats = resultats.clone();
        let endpoints = props.endpoints.clone();
        use_effect_with((*saisie).clone(), move |texte: &String| {
            let texte = texte.trim().to_owned();

            let minuteur = if texte.chars().count() < 2 {
                resultats.set(Vec::new());
                None
            } else {
                Some(Timeout::new(ATTENTE_MS, move || {
                    wasm_bindgen_futures::spawn_local(async move {
                        let trouvees =
                            reseau::search(&endpoints, &texte).await.unwrap_or_default();
                        resultats.set(trouvees);
                    });
                }))
            };

            // Le minuteur est annulé par son `Drop` : une frappe rapide ne
            // lance donc qu'un seul appel, celui de la dernière lettre.
            move || drop(minuteur)
        });
    }

    // Un clic ailleurs referme la liste.
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

    let choisir = {
        let on_select = props.on_select.clone();
        let saisie = saisie.clone();
        let resultats = resultats.clone();
        let erreur = erreur.clone();
        Callback::from(move |parcelle: Parcelle| {
            on_select.emit(parcelle);
            saisie.set(String::new());
            resultats.set(Vec::new());
            erreur.set(None);
        })
    };

    let localiser = {
        let choisir = choisir.clone();
        let localisation = localisation.clone();
        let erreur = erreur.clone();
        let nom = i18n.t("search.myField");
        let indisponible = i18n.t("search.unsupported");
        let refusee = i18n.t("search.denied");

        Callback::from(move |_| {
            let Some(geo) = web_sys::window().and_then(|w| w.navigator().geolocation().ok()) else {
                erreur.set(Some(indisponible.clone()));
                return;
            };
            localisation.set(true);

            let succes = {
                let choisir = choisir.clone();
                let localisation = localisation.clone();
                let nom = nom.clone();
                Closure::<dyn Fn(web_sys::Position)>::new(move |position: web_sys::Position| {
                    localisation.set(false);
                    let coords = position.coords();
                    choisir.emit(parcelle_from_position(
                        &nom,
                        coords.latitude(),
                        coords.longitude(),
                    ));
                })
            };
            let echec = {
                let localisation = localisation.clone();
                let erreur = erreur.clone();
                let refusee = refusee.clone();
                Closure::<dyn Fn(JsValue)>::new(move |_| {
                    localisation.set(false);
                    erreur.set(Some(refusee.clone()));
                })
            };

            let options = web_sys::PositionOptions::new();
            options.set_timeout(10_000);
            let _ = geo.get_current_position_with_error_callback_and_options(
                succes.as_ref().unchecked_ref(),
                Some(echec.as_ref().unchecked_ref()),
                &options,
            );
            // Le navigateur rappellera ces fonctions après ce tour de boucle :
            // les oublier ici libérerait la mémoire qu'il va lire.
            succes.forget();
            echec.forget();
        })
    };

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
                let choisir = choisir.clone();
                let resultat = resultat.clone();
                Callback::from(move |_| choisir.emit(resultat.clone()))
            };
            let region: Vec<&str> = [resultat.admin.as_deref(), resultat.country.as_deref()]
                .into_iter()
                .flatten()
                .collect();

            html! {
                <li key={format!("{},{}", resultat.latitude, resultat.longitude)}>
                    <button type="button" {onclick}>
                        <span>{ &resultat.name }</span>
                        <span class="search__admin">{ region.join(", ") }</span>
                    </button>
                </li>
            }
        })
        .collect();

    html! {
        <div class="search">
            <div class="search__row">
                <input
                    type="search"
                    class="search__input"
                    value={(*saisie).clone()}
                    oninput={sur_saisie}
                    placeholder={i18n.with(
                        "search.placeholder",
                        &params([("parcelle", props.current.name.as_str().into())]),
                    )}
                    aria-label={i18n.t("search.label")}
                />
                <button
                    type="button"
                    class="button"
                    onclick={localiser}
                    disabled={*localisation}
                >
                    { i18n.t(if *localisation { "search.locating" } else { "search.locate" }) }
                </button>
            </div>

            if let Some(message) = &*erreur {
                <p class="search__error">{ message }</p>
            }

            if !resultats.is_empty() {
                <ul class="search__results">{ liste }</ul>
            }
        </div>
    }
}
