//! Le bandeau horaire.
//!
//! Miroir de `web/src/components/HourlyStrip.tsx`.
//!
//! Ce qu'on montre sans rien demander : douze heures, une demi-journée. Les
//! vingt-quatre repliées en grille tenaient dans la carte, mais la carte
//! tenait tout l'écran. Le reste se déplie d'un bouton — et **rien ne défile
//! de côté** : c'est la règle du web, et seule l'application iOS a un bandeau
//! qui glisse.

use klima_core::agro::{CurrentSample, HourlySample};
use klima_core::weather::weather_condition;
use yew::prelude::*;

use klima_ui::composants::pictogramme::Pictogramme;
use klima_ui::dates;
use klima_ui::i18n::use_i18n;

/// Une demi-journée répond à la question qu'on se pose en ouvrant
/// l'application.
const APERCU: usize = 12;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub hours: Vec<HourlySample>,
    /// Conditions observées, affichées sur la colonne « Maintenant ».
    pub current: CurrentSample,
}

#[function_component]
pub fn Bandeau(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let deplie = use_state(|| false);

    let limite = if *deplie { 24 } else { APERCU };
    let colonnes: Html = props
        .hours
        .iter()
        .take(limite)
        .enumerate()
        .map(|(index, hour)| {
            // La première colonne montre le relevé courant, pas la prévision
            // de l'heure déjà entamée.
            let observe = index == 0;
            let code = if observe { props.current.weather_code } else { hour.weather_code };
            let condition = weather_condition(code);

            html! {
                <div class="strip__item" key={hour.time}>
                    <span class="strip__hour">
                        if observe {
                            { i18n.t("hourly.now") }
                        } else {
                            { dates::heure(hour.time, i18n.locale()) }
                        }
                    </span>

                    <Pictogramme
                        icon={condition.icon}
                        is_day={if observe { props.current.is_day } else { hour.is_day }}
                        title={i18n.t(condition.label_key)}
                    />

                    <span class="strip__rain">
                        if hour.precipitation_probability >= 10.0 {
                            { f.percent(hour.precipitation_probability) }
                        } else {
                            { "\u{00a0}" }
                        }
                    </span>

                    <span class="strip__temp">
                        { f.temperature(if observe {
                            props.current.temperature
                        } else {
                            hour.temperature
                        }) }
                    </span>
                </div>
            }
        })
        .collect();

    let basculer = {
        let deplie = deplie.clone();
        Callback::from(move |_| deplie.set(!*deplie))
    };

    html! {
        <section class="card" aria-label={i18n.t("hourly.title")}>
            <h2 class="card__label">{ i18n.t("hourly.title") }</h2>
            <div class="strip">{ colonnes }</div>

            if props.hours.len() > APERCU {
                <button
                    type="button"
                    class="strip__toggle"
                    aria-expanded={(*deplie).to_string()}
                    onclick={basculer}
                >
                    { i18n.t(if *deplie { "hourly.fold" } else { "hourly.unfold" }) }
                </button>
            }
        </section>
    }
}
