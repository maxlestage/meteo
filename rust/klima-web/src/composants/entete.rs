//! L'en-tête : commune, température, temps et amplitude du jour.
//!

use klima_api::open_meteo::Forecast;
use klima_core::weather::weather_condition;
use yew::prelude::*;

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
            <p class="hero__temperature">{ f.temperature(forecast.current.temperature) }</p>
            <p class="hero__condition">{ i18n.t(condition.label_key) }</p>

            if let Some(today) = forecast.daily.first() {
                <p class="hero__range">
                    { format!("↑ {}", f.temperature(today.temperature_max)) }
                    { "\u{00a0} " }
                    { format!("↓ {}", f.temperature(today.temperature_min)) }
                </p>
            }
        </header>
    }
}
