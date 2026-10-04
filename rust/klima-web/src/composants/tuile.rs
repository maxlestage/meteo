//! La tuile de détail, dans l'esprit des cartes « Vent » ou « Indice UV ».
//!

use yew::prelude::*;

/// Une jauge : une position de 0 à 1 sur une échelle colorée.
#[derive(Clone, PartialEq)]
pub struct Jauge {
    pub position: f64,
    pub gradient: &'static str,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub label: AttrValue,
    pub value: AttrValue,
    #[prop_or_default]
    pub caption: Option<AttrValue>,
    #[prop_or_default]
    pub jauge: Option<Jauge>,
}

#[function_component]
pub fn Tuile(props: &Props) -> Html {
    html! {
        <article class="tile">
            <h3 class="card__label">{ &props.label }</h3>
            <p class="tile__value">{ &props.value }</p>

            if let Some(jauge) = &props.jauge {
                <div class="gauge" style={format!("background: {}", jauge.gradient)}>
                    <span
                        class="gauge__cursor"
                        style={format!("left: {}%", jauge.position.clamp(0.0, 1.0) * 100.0)}
                    />
                </div>
            }

            if let Some(caption) = &props.caption {
                <p class="tile__caption">{ caption }</p>
            }
        </article>
    }
}
