//! Ce que dit chaque source, l'une sous l'autre.
//!
//! La tuile « Accord des modèles » résume : combien de sources, quel écart,
//! quelle valeur retenue. Elle ne dit pas *qui* annonce quoi — et c'est
//! pourtant ce qu'on regarde quand Météo-France et le modèle américain ne
//! s'entendent pas sur la pluie. Miroir de `SourcesCardView.swift`.
//!
//! Deux lignes par source plutôt qu'une rangée de colonnes : l'institut et sa
//! température, puis le modèle, la pluie et le vent. Rien ne défile de côté.

use klima_api::apprenti::Apprentissage;
use klima_core::apprenti::meilleure;
use klima_core::consensus::Consensus;
use klima_core::consensus::thresholds::{RAIN_THRESHOLD, STRONG_TEMPERATURE_SPREAD};
use klima_core::i18n::params;
use yew::prelude::*;

use klima_ui::i18n::use_i18n;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub consensus: Consensus,
    /// Ce que l'apprenti a appris des modèles, quand le relais l'a dit.
    #[prop_or_default]
    pub apprentissage: Option<Apprentissage>,
}

/// Le nom d'une source d'après son identifiant : l'institut.
fn institut(id: &str) -> String {
    klima_core::providers::OPEN_METEO_SOURCES
        .iter()
        .find(|s| s.id == id)
        .map_or_else(|| id.to_owned(), |s| s.institution.to_owned())
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

    // L'apprenti : qui il a jugé, le plus juste, et ce qu'il a corrigé.
    let apprenti = props.apprentissage.as_ref().filter(|a| !a.notes.is_empty()).map(|appris| {
        let mut phrases = vec![i18n.with("apprenti.juge", &params([
            ("station", appris.nom.as_str().into()),
            ("days", (appris.jours as usize).into()),
        ]))];
        if let Some(juste) = meilleure(&appris.notes) {
            phrases.push(i18n.with("apprenti.meilleure", &params([
                ("source", institut(&juste.source_id).into()),
                ("ecart", f.unit(juste.ecart, "°C", 1).as_str().into()),
            ])));
        }
        let corrigees: Vec<String> = appris
            .notes
            .iter()
            .filter(|n| n.correction() != 0.0)
            .map(|n| {
                let cle = if n.correction() > 0.0 { "apprenti.tropChaud" } else { "apprenti.tropFroid" };
                i18n.with(cle, &params([
                    ("source", institut(&n.source_id).into()),
                    ("value", f.unit(n.correction().abs(), "°C", 1).as_str().into()),
                ]))
            })
            .collect();
        if corrigees.is_empty() {
            phrases.push(i18n.t("apprenti.aucun"));
        } else {
            phrases.extend(corrigees);
        }
        if let Some(pluie) = appris
            .notes
            .iter()
            .filter(|n| n.pluie_n >= klima_core::apprenti::seuils::RELEVES_MIN)
            .max_by(|a, b| a.justesse_pluie().unwrap_or(0.0).total_cmp(&b.justesse_pluie().unwrap_or(0.0)))
        {
            phrases.push(i18n.with("apprenti.pluie", &params([
                ("source", institut(&pluie.source_id).into()),
                ("percent", pluie.justesse_pluie().unwrap_or(0.0).into()),
            ])));
        }
        html! {
            <div class="sources__apprenti">
                <h3 class="sources__apprenti-titre">{ i18n.t("apprenti.title") }</h3>
                <ul>{ for phrases.into_iter().map(|p| html! { <li>{ p }</li> }) }</ul>
            </div>
        }
    });

    html! {
        <section class="card" id="sources" aria-label={i18n.t("sources.title")}>
            <h2 class="card__label">{ i18n.t("sources.title") }</h2>
            <ul class="sources">{ lignes }</ul>
            <p class="sources__resume">{ resume }</p>
            { apprenti.unwrap_or_default() }
        </section>
    }
}
