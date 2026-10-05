//! Ce que dit chaque source, l'une sous l'autre.
//!
//! La tuile « Accord des modèles » résume : combien de sources, quel écart,
//! quelle valeur retenue. Elle ne dit pas *qui* annonce quoi — et c'est
//! pourtant ce qu'on regarde quand Météo-France et le modèle américain ne
//! s'entendent pas sur la pluie. Miroir de `SourcesCardView.swift`.
//!
//! Deux lignes par source plutôt qu'une rangée de colonnes : l'institut et sa
//! température, puis le modèle, la pluie et le vent. Rien ne défile de côté.

use klima_core::consensus::Consensus;
use klima_core::consensus::thresholds::{RAIN_THRESHOLD, STRONG_TEMPERATURE_SPREAD};
use klima_core::i18n::params;
use yew::prelude::*;

use klima_ui::i18n::use_i18n;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub consensus: Consensus,
}

#[function_component]
pub fn Sources(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let consensus = &props.consensus;

    let lignes: Html = consensus
        .readings
        .iter()
        .map(|lecture| {
            let pluie = if lecture.precipitation >= RAIN_THRESHOLD {
                f.unit(lecture.precipitation, "mm", 1)
            } else {
                i18n.t("sources.dry")
            };
            let detail =
                [lecture.source.name.to_owned(), pluie, f.unit(lecture.wind_speed, "km/h", 0)].join(" · ");
            // Une source qui s'écarte de la valeur retenue de plus que l'accord
            // fort ne le tolère se signale : c'est elle qu'il faut regarder.
            let ecartee =
                (lecture.temperature - consensus.temperature.median).abs() > STRONG_TEMPERATURE_SPREAD;

            html! {
                <li class="sources__row" key={lecture.source.id}>
                    <span class="sources__institution">{ lecture.source.institution }</span>
                    <span class={classes!("sources__temperature", ecartee.then_some("sources__temperature--ecart"))}>
                        { f.temperature(lecture.temperature) }
                    </span>
                    <span class="sources__detail">{ detail }</span>
                </li>
            }
        })
        .collect();

    let resume = format!(
        "{}. {}.",
        i18n.with("consensus.detail", &params([
            ("count", consensus.readings.len().into()),
            ("spread", f.unit(consensus.temperature.spread, "°C", 1).as_str().into()),
        ])),
        i18n.with("consensus.median", &params([
            ("value", f.unit(consensus.temperature.median, "°C", 1).as_str().into()),
        ])),
    );

    html! {
        <section class="card" aria-label={i18n.t("sources.title")}>
            <h2 class="card__label">{ i18n.t("sources.title") }</h2>
            <ul class="sources">{ lignes }</ul>
            <p class="sources__resume">{ resume }</p>
        </section>
    }
}
