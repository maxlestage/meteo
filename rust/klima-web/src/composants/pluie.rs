//! La pluie qui vient, et ce qu'il faut emporter.
//!
//! La première question qu'on pose à une météo de ville : faut-il un
//! parapluie, et jusqu'à quand. La carte y répond en une phrase, la montre
//! sur douze barres — une par heure, aussi hautes que le risque —, puis dit
//! ce qu'il faut prendre avant de sortir. Miroir de `PluieCardView.swift`.
//!
//! Les douze barres tiennent dans la largeur de la carte : rien ne défile de
//! côté.

use klima_core::i18n::params;
use klima_core::meteo::HourlySample;
use klima_core::ville::{Pluie, conseils, pluvieuse, prochaine_pluie, seuils::HORIZON_PLUIE};
use yew::prelude::*;

use klima_ui::dates;
use klima_ui::i18n::use_i18n;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub hours: Vec<HourlySample>,
}

#[function_component]
pub fn CartePluie(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let locale = i18n.locale();

    let pluie = prochaine_pluie(&props.hours);
    let titre = match &pluie {
        Pluie::Aucune { heures } => i18n.with("rain.none", &params([("hours", (*heures).into())])),
        Pluie::EnCours { fin: Some(fin) } => {
            i18n.with("rain.now", &params([("time", dates::heure(*fin, locale).as_str().into())]))
        }
        Pluie::EnCours { fin: None } => i18n.t("rain.nowLasting"),
        Pluie::Prevue { debut, .. } => {
            i18n.with("rain.soon", &params([("time", dates::heure(*debut, locale).as_str().into())]))
        }
    };
    let detail = match &pluie {
        Pluie::Prevue { probabilite, cumul, .. } => Some(i18n.with(
            "rain.detail",
            &params([
                ("probability", f.percent(*probabilite).as_str().into()),
                ("amount", f.unit(*cumul, "mm", 1).as_str().into()),
            ]),
        )),
        _ => None,
    };

    let barres: Html = props
        .hours
        .iter()
        .take(HORIZON_PLUIE)
        .map(|heure| {
            let hauteur = heure.precipitation_probability.clamp(4.0, 100.0);
            html! {
                <span
                    key={heure.time}
                    class={classes!("rain__bar", pluvieuse(heure).then_some("rain__bar--wet"))}
                    style={format!("height: {hauteur}%")}
                    title={format!("{} · {}", dates::heure(heure.time, locale), f.percent(heure.precipitation_probability))}
                />
            }
        })
        .collect();

    let liste = conseils(&props.hours);
    let a_emporter: Html = if liste.is_empty() {
        html! { <p class="advice__none">{ i18n.t("advice.none") }</p> }
    } else {
        html! {
            <ul class="advice">
                { for liste.iter().map(|c| html! { <li key={c.code()}>{ i18n.t(&c.key()) }</li> }) }
            </ul>
        }
    };

    html! {
        <section class="card" aria-label={i18n.t("rain.title")}>
            <h2 class="card__label">{ i18n.t("rain.title") }</h2>
            <p class="rain__headline">{ titre }</p>
            if let Some(detail) = detail {
                <p class="rain__detail">{ detail }</p>
            }

            <div class="rain__bars" aria-hidden="true">{ barres }</div>
            <div class="rain__scale" aria-hidden="true">
                <span>{ i18n.t("rain.axisNow") }</span>
                <span>{ "+6 h" }</span>
                <span>{ "+12 h" }</span>
            </div>

            <h3 class="card__label advice__title">{ i18n.t("advice.title") }</h3>
            { a_emporter }
        </section>
    }
}
