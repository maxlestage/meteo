//! Le recoupement des modèles, montré plutôt qu'affirmé.
//!
//! Chaque service porte son point sur l'axe des températures, la médiane est
//! marquée, et l'écart se lit d'un coup d'œil. Dire « nous recoupons cinq
//! instituts » n'engage à rien ; les montrer en désaccord de 1,9 °C, si.

use klima_core::consensus::{Agreement, Consensus};
use klima_core::i18n::params;
use klima_core::providers::attributions_for;
use klima_ui::i18n::use_i18n;
use yew::prelude::*;

use crate::crochets::apparition::use_apparition;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub consensus: Option<Consensus>,
    pub loading: bool,
}

fn code_accord(accord: Agreement) -> &'static str {
    match accord {
        Agreement::Forte => "forte",
        Agreement::Moyenne => "moyenne",
        Agreement::Faible => "faible",
    }
}

#[function_component]
pub fn Sources(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let apparition = use_apparition();

    let modeles: Html = props
        .consensus
        .as_ref()
        .map(|consensus| {
            consensus
                .readings
                .iter()
                .map(|reading| {
                    // Une étendue nulle mettrait tous les points au même
                    // endroit : on garde un dixième de degré de marge.
                    let etendue = consensus.temperature.spread.max(0.1);
                    let position =
                        (reading.temperature - consensus.temperature.min) / etendue * 100.0;

                    html! {
                        <li class="models__row" key={reading.source.id}>
                            <span class="models__flag">{ reading.source.country }</span>
                            <span class="models__name">
                                <strong>{ reading.source.institution }</strong>
                                <span>{ reading.source.name }</span>
                            </span>
                            <span class="models__axis">
                                <span
                                    class="models__dot"
                                    style={format!("left: {position}%")}
                                />
                            </span>
                            <span class="models__value">
                                { f.unit(reading.temperature, "°C", 1) }
                            </span>
                        </li>
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let mentions: Html = props
        .consensus
        .as_ref()
        .map(|consensus| {
            attributions_for(&consensus.readings)
                .into_iter()
                .map(|mention| html! { <li key={mention}>{ mention }</li> })
                .collect()
        })
        .unwrap_or_default();

    html! {
        <section class="sources" id="sources">
            <div class="section-head">
                <h2>{ i18n.t("sources.title") }</h2>
                <p>{ i18n.t("sources.lead") }</p>
            </div>

            <div
                ref={apparition.node}
                class={classes!("sources__panel", apparition.class)}
            >
                if props.loading && props.consensus.is_none() {
                    <p class="sources__state">{ i18n.t("sources.loading") }</p>
                }
                if !props.loading && props.consensus.is_none() {
                    <p class="sources__state">{ i18n.t("sources.unavailable") }</p>
                }

                if let Some(consensus) = &props.consensus {
                    <>
                        <header class="sources__header">
                            <div>
                                <p class="sources__label">{ i18n.t("consensus.title") }</p>
                                <p class={format!(
                                    "sources__verdict sources__verdict--{}",
                                    code_accord(consensus.agreement)
                                )}>
                                    { i18n.t(&format!(
                                        "consensus.{}",
                                        code_accord(consensus.agreement)
                                    )) }
                                </p>
                            </div>
                            <div class="sources__median">
                                <p class="sources__label">{ i18n.t("sources.now") }</p>
                                <p class="sources__value">
                                    { f.unit(consensus.temperature.median, "°C", 1) }
                                </p>
                                <p class="sources__providers">
                                    { i18n.with("sources.answered", &params([
                                        ("answered", consensus.providers_answered.into()),
                                        ("queried", consensus.providers_queried.into()),
                                    ])) }
                                </p>
                                <p class="sources__spread">
                                    { i18n.with("consensus.detail", &params([
                                        ("count", consensus.readings.len().into()),
                                        ("spread", f.unit(consensus.temperature.spread, "°C", 1).as_str().into()),
                                    ])) }
                                    if !consensus.agree_on_rain {
                                        { format!(" · {}", i18n.t("consensus.rainDisagreement")) }
                                    }
                                </p>
                            </div>
                        </header>

                        <ul class="models">{ modeles }</ul>
                    </>
                }
            </div>

            <div class="sources__method">
                <h3>{ i18n.t("sources.method") }</h3>
                <p>{ i18n.t("sources.methodBody") }</p>
                <p class="sources__note">{ i18n.t("sources.note") }</p>
                if props.consensus.is_some() {
                    <ul class="sources__attributions">{ mentions }</ul>
                }
            </div>
        </section>
    }
}
