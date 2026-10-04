//! Le sélecteur de langue : trois boutons, celui de la langue courante marqué.
//!

use klima_core::i18n::LANGUAGES;
use yew::prelude::*;

use crate::i18n::use_i18n;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Le style appartient à l'application ; seul le comportement est partagé.
    #[prop_or(AttrValue::from("lang"))]
    pub class: AttrValue,
    pub label: AttrValue,
}

#[function_component]
pub fn SelecteurDeLangue(props: &Props) -> Html {
    let i18n = use_i18n();

    let boutons: Html = LANGUAGES
        .into_iter()
        .map(|candidate| {
            let courante = candidate == i18n.language;
            let onclick = {
                let set = i18n.set_language.clone();
                Callback::from(move |_| set.emit(candidate))
            };

            html! {
                <button
                    type="button"
                    class={classes!(courante.then_some("is-current"))}
                    aria-current={courante.then(|| AttrValue::from("true"))}
                    title={candidate.name()}
                    {onclick}
                >
                    { candidate.code().to_uppercase() }
                </button>
            }
        })
        .collect();

    html! {
        <div class={props.class.clone()} role="group" aria-label={props.label.clone()}>
            { boutons }
        </div>
    }
}
