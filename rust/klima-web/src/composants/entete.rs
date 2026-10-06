//! L'en-tête : commune, température, temps et amplitude du jour.
//!

use klima_api::open_meteo::Forecast;
use klima_core::i18n::params;
use klima_core::weather::weather_condition;
use yew::prelude::*;

use klima_ui::composants::pictogramme::Pictogramme;
use klima_ui::composants::vu::CielVu;
use klima_ui::i18n::use_i18n;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub forecast: Forecast,
}

#[function_component]
pub fn Entete(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let forecast = &props.forecast;
    let condition = weather_condition(forecast.current.weather_code);

    html! {
        <header class="hero">
            <h1 class="hero__place">{ &forecast.parcelle.name }</h1>
            <Pictogramme
                icon={condition.icon}
                is_day={forecast.current.is_day}
                size={76}
                anime={true}
            />
            // La clé fait rejouer l'apparition quand la température change.
            <p class="hero__temperature" key={f.temperature(forecast.current.temperature)}>
                { f.temperature(forecast.current.temperature) }
            </p>
            <p class="hero__condition">{ i18n.t(condition.label_key) }</p>

            if let Some(today) = forecast.daily.first() {
                <p class="hero__range">
                    { format!("↑ {}", f.temperature(today.temperature_max)) }
                    { "\u{00a0} " }
                    { format!("↓ {}", f.temperature(today.temperature_min)) }
                </p>
            }

            // La prévision n'est pas celle d'un modèle : toutes les sources
            // l'ont faite. On le dit, et on renvoie à leur détail.
            if !forecast.sources.is_empty() {
                <a class="hero__recoupe" href="#sources">
                    { i18n.with("forecast.blended", &params([("count", forecast.sources.len().into())])) }
                </a>
            }

            // Ce qu'on voit tomber à l'aéroport le plus proche, quand il
            // tombe quelque chose : une observation, pas une prévision.
            <CielVu ciel={forecast.ciel.clone()} class={classes!("hero__vu")} />
        </header>
    }
}
