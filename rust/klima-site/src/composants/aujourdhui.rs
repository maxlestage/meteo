//! La météo du jour, et rien d'autre : ce que Klima dit de la journée en cours
//! dans la ville choisie.
//!

use klima_api::today::DayDigest;
use klima_core::meteo::CurrentSample;
use klima_core::ville::{NiveauUv, Pluie, niveau_uv};
use klima_core::endpoints::Endpoints;
use klima_core::i18n::params;
use klima_core::position::Parcelle;
use klima_core::weather::weather_condition;
use klima_ui::composants::pictogramme::Pictogramme;
use klima_ui::dates;
use klima_ui::i18n::use_i18n;
use yew::prelude::*;

use crate::composants::commune::ChoixDeCommune;
use crate::composants::compte::{Compte, Forme};
use crate::composants::film::mots;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub parcelle: Parcelle,
    pub digest: Option<DayDigest>,
    /// Relevé du moment : c'est lui qui donne la grande température.
    pub current: Option<CurrentSample>,
    /// Combien de sources ont fait la prévision ; 0 tant qu'elle n'est pas
    /// recoupée.
    #[prop_or_default]
    pub sources: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub endpoints: Endpoints,
    pub on_select: Callback<Parcelle>,
    pub on_retry: Callback<()>,
    /// Ce qui suit la carte du jour : le guetteur, sur la même ville.
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn SectionDuJour(props: &Props) -> Html {
    let i18n = use_i18n();

    let reessayer = {
        let on_retry = props.on_retry.clone();
        Callback::from(move |_| on_retry.emit(()))
    };

    html! {
        <section class="today" id="aujourdhui">
            <div class="section-head" data-apparait="titre">
                <h2>{ mots(&i18n.t("today.title")) }</h2>
                <p>{ i18n.t("today.lead") }</p>
                <a class="today__app" href="./app/">{ i18n.t("app.open") }</a>
            </div>

            <ChoixDeCommune
                current={props.parcelle.clone()}
                endpoints={props.endpoints.clone()}
                on_select={props.on_select.clone()}
            />

            <div class="today__card" data-apparait="monte">
                if props.loading {
                    <p class="today__state">{ i18n.t("today.loading") }</p>
                }

                if let Some(message) = &props.error {
                    <div class="today__state" role="alert">
                        <p>{ message }</p>
                        <button type="button" class="button button--ghost" onclick={reessayer}>
                            { i18n.t("today.retry") }
                        </button>
                    </div>
                }

                if let (Some(digest), Some(current), false, None) =
                    (&props.digest, &props.current, props.loading, &props.error)
                {
                    <Contenu
                        parcelle={props.parcelle.clone()}
                        digest={digest.clone()}
                        current={current.clone()}
                        sources={props.sources}
                    />
                }
            </div>

            <div class="today__suite" data-apparait="monte">
                { props.children.clone() }
            </div>
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct ContenuProps {
    parcelle: Parcelle,
    digest: DayDigest,
    current: CurrentSample,
    sources: usize,
}

#[function_component]
fn Contenu(props: &ContenuProps) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let digest = &props.digest;
    let condition = weather_condition(props.current.weather_code);

    let heure = |ms: Option<i64>| match ms {
        Some(ms) => dates::heure_minute(ms, i18n.locale()),
        None => "—".to_owned(),
    };

    html! {
        <>
            <div class="today__hero">
                <div>
                    <p class="today__place">{ &props.parcelle.name }</p>
                    <p class="today__date">
                        { dates::jour_complet(digest.date, i18n.locale()) }
                    </p>
                    if props.sources > 0 {
                        <p class="today__recoupe">
                            { i18n.with("forecast.blended", &params([("count", props.sources.into())])) }
                        </p>
                    }
                </div>

                <div class="today__now">
                    <Pictogramme
                        icon={condition.icon}
                        size={54}
                        title={i18n.t(condition.label_key)}
                        anime={true}
                    />
                    <p class="today__temperature">
                        <Compte valeur={props.current.temperature} forme={Forme::Temperature} />
                    </p>
                    <div>
                        <p class="today__condition">{ i18n.t(condition.label_key) }</p>
                        <p class="today__range">
                            { format!("↑ {}", f.temperature(digest.temperature_max)) }
                            { "\u{00a0} " }
                            { format!("↓ {}", f.temperature(digest.temperature_min)) }
                        </p>
                    </div>
                </div>
            </div>

            <dl class="today__facts">
                <Fait
                    label={i18n.t("today.rain")}
                    value={html! { <Compte valeur={digest.precipitation_sum} forme={Forme::Unite("mm", 1)} /> }}
                    detail={i18n.with("today.rain.detail", &params([
                        ("probability", f.percent(digest.precipitation_probability_max).as_str().into()),
                    ]))}
                />
                <Fait
                    label={i18n.t("today.gusts")}
                    value={html! { <Compte valeur={digest.wind_gusts_max} forme={Forme::Unite("km/h", 0)} /> }}
                    detail={i18n.t("today.gusts.detail")}
                />
                <Fait
                    label={i18n.t("today.sunrise")}
                    value={html! { heure(digest.sunrise) }}
                    detail={i18n.with("today.sunrise.detail", &params([
                        ("time", heure(digest.sunset).as_str().into()),
                    ]))}
                />
                <Fait
                    label={i18n.t("today.uv")}
                    value={html! { <Compte valeur={digest.uv_index_max} forme={Forme::Decimale(0)} /> }}
                    detail={i18n.t(&digest.uv.key())}
                />
            </dl>

            <div class="today__agro">
                <Conseil
                    label={i18n.t("today.next")}
                    value={match &digest.pluie {
                        Pluie::Aucune { heures } => i18n.with("rain.none", &params([("hours", (*heures).into())])),
                        Pluie::EnCours { fin: Some(fin) } => i18n.with("rain.now", &params([
                            ("time", dates::heure(*fin, i18n.locale()).as_str().into()),
                        ])),
                        Pluie::EnCours { fin: None } => i18n.t("rain.nowLasting"),
                        Pluie::Prevue { debut, .. } => i18n.with("rain.soon", &params([
                            ("time", dates::heure(*debut, i18n.locale()).as_str().into()),
                        ])),
                    }}
                    detail={match &digest.pluie {
                        Pluie::Prevue { probabilite, cumul, .. } => i18n.with("rain.detail", &params([
                            ("probability", f.percent(*probabilite).as_str().into()),
                            ("amount", f.unit(*cumul, "mm", 1).as_str().into()),
                        ])),
                        _ => i18n.t("today.next.detail"),
                    }}
                    ton={match &digest.pluie {
                        Pluie::Aucune { .. } => Ton::Bon,
                        Pluie::Prevue { .. } => Ton::Attention,
                        Pluie::EnCours { .. } => Ton::Mauvais,
                    }}
                />
                <Conseil
                    label={i18n.t("advice.title")}
                    value={match digest.conseils.first() {
                        Some(premier) => i18n.t(&premier.key()),
                        None => i18n.t("today.advice.none"),
                    }}
                    detail={if digest.conseils.len() > 1 {
                        digest.conseils[1..].iter().map(|c| i18n.t(&c.key())).collect::<Vec<_>>().join(" · ")
                    } else {
                        i18n.t("today.advice.detail")
                    }}
                    ton={if digest.conseils.is_empty() { Ton::Bon } else { Ton::Attention }}
                />
                <Conseil
                    label={i18n.t("today.feels")}
                    value={f.temperature(props.current.apparent_temperature)}
                    detail={i18n.with("today.feels.detail", &params([
                        ("temperature", f.temperature(props.current.temperature).as_str().into()),
                    ]))}
                    ton={if (0.0..30.0).contains(&props.current.apparent_temperature) {
                        Ton::Bon
                    } else {
                        Ton::Attention
                    }}
                />
                <Conseil
                    label={i18n.t("today.sun")}
                    value={i18n.t(&niveau_uv(digest.uv_index_max).key())}
                    detail={i18n.with("today.sun.detail", &params([
                        ("index", f.decimal(digest.uv_index_max, 0).as_str().into()),
                    ]))}
                    ton={match digest.uv {
                        NiveauUv::Faible | NiveauUv::Modere => Ton::Bon,
                        NiveauUv::Eleve => Ton::Attention,
                        _ => Ton::Mauvais,
                    }}
                />
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct FaitProps {
    label: AttrValue,
    /// Un chiffre, le plus souvent qui roule.
    value: Html,
    detail: AttrValue,
}

#[function_component]
fn Fait(props: &FaitProps) -> Html {
    html! {
        <div class="fact">
            <dt>{ &props.label }</dt>
            <dd>
                <span class="fact__value">{ props.value.clone() }</span>
                <span class="fact__detail">{ &props.detail }</span>
            </dd>
        </div>
    }
}

/// Ce que le conseil vaut, d'un coup d'œil.
#[derive(Clone, Copy, PartialEq)]
enum Ton {
    Bon,
    Attention,
    Mauvais,
}

impl Ton {
    fn code(self) -> &'static str {
        match self {
            Ton::Bon => "good",
            Ton::Attention => "warn",
            Ton::Mauvais => "bad",
        }
    }
}

#[derive(Properties, PartialEq)]
struct ConseilProps {
    label: AttrValue,
    value: AttrValue,
    detail: AttrValue,
    ton: Ton,
}

#[function_component]
fn Conseil(props: &ConseilProps) -> Html {
    html! {
        <article class={format!("advice advice--{}", props.ton.code())}>
            <h3>{ &props.label }</h3>
            <p class="advice__value">{ &props.value }</p>
            <p class="advice__detail">{ &props.detail }</p>
        </article>
    }
}
