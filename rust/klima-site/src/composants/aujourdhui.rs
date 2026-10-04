//! La météo du jour, et rien d'autre : ce que Klima dit de la journée en cours
//! sur la parcelle choisie.
//!

use klima_api::today::DayDigest;
use klima_core::agro::{CurrentSample, FrostSeverity, SoilState};
use klima_core::endpoints::Endpoints;
use klima_core::i18n::params;
use klima_core::position::Parcelle;
use klima_core::weather::weather_condition;
use klima_ui::composants::pictogramme::Pictogramme;
use klima_ui::dates;
use klima_ui::i18n::use_i18n;
use yew::prelude::*;

use crate::composants::commune::ChoixDeCommune;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub parcelle: Parcelle,
    pub digest: Option<DayDigest>,
    /// Relevé du moment : c'est lui qui donne la grande température.
    pub current: Option<CurrentSample>,
    pub loading: bool,
    pub error: Option<String>,
    pub endpoints: Endpoints,
    pub on_select: Callback<Parcelle>,
    pub on_retry: Callback<()>,
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
            <div class="section-head">
                <h2>{ i18n.t("today.title") }</h2>
                <p>{ i18n.t("today.lead") }</p>
                <a class="today__app" href="./app/">{ i18n.t("app.open") }</a>
            </div>

            <ChoixDeCommune
                current={props.parcelle.clone()}
                endpoints={props.endpoints.clone()}
                on_select={props.on_select.clone()}
            />

            <div class="today__card">
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
                    />
                }
            </div>
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct ContenuProps {
    parcelle: Parcelle,
    digest: DayDigest,
    current: CurrentSample,
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
                </div>

                <div class="today__now">
                    <Pictogramme
                        icon={condition.icon}
                        size={54}
                        title={i18n.t(condition.label_key)}
                    />
                    <p class="today__temperature">
                        { f.temperature(props.current.temperature) }
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
                    value={f.unit(digest.precipitation_sum, "mm", 1)}
                    detail={i18n.with("today.rain.detail", &params([
                        ("probability", f.percent(digest.precipitation_probability_max).as_str().into()),
                    ]))}
                />
                <Fait
                    label={i18n.t("today.gusts")}
                    value={f.unit(digest.wind_gusts_max, "km/h", 0)}
                    detail={i18n.t("today.gusts.detail")}
                />
                <Fait
                    label={i18n.t("today.sunrise")}
                    value={heure(digest.sunrise)}
                    detail={i18n.with("today.sunrise.detail", &params([
                        ("time", heure(digest.sunset).as_str().into()),
                    ]))}
                />
                <Fait
                    label={i18n.t("today.et0")}
                    value={f.unit(digest.et0_sum, "mm", 1)}
                    detail={i18n.t("today.et0.detail")}
                />
            </dl>

            <div class="today__agro">
                <Conseil
                    label={i18n.t("today.spray")}
                    value={match &digest.spray {
                        Some(spray) => format!(
                            "{} → {}",
                            dates::heure(spray.start, i18n.locale()),
                            dates::heure(spray.end, i18n.locale())
                        ),
                        None => i18n.t("today.spray.none"),
                    }}
                    detail={match &digest.spray {
                        Some(spray) => i18n.with("today.spray.score", &params([
                            ("score", spray.score.into()),
                        ])),
                        None => i18n.t("today.spray.blocked"),
                    }}
                    ton={match &digest.spray {
                        Some(spray) if spray.score >= 80 => Ton::Bon,
                        Some(_) => Ton::Attention,
                        None => Ton::Mauvais,
                    }}
                />
                <Conseil
                    label={i18n.t("today.balance")}
                    value={f.signed_unit(digest.balance, "mm", 1)}
                    detail={i18n.t(if digest.balance < 0.0 {
                        "today.balance.deficit"
                    } else {
                        "today.balance.ok"
                    })}
                    ton={if digest.balance < 0.0 { Ton::Attention } else { Ton::Bon }}
                />
                <Conseil
                    label={i18n.t("today.soil")}
                    value={i18n.t(match digest.soil.state {
                        SoilState::Sature => "soil.sature",
                        SoilState::Ressuye => "soil.ressuye",
                        SoilState::Sec => "soil.sec",
                    })}
                    detail={i18n.with("today.soil.detail", &params([
                        ("moisture", f.percent(digest.soil.moisture * 100.0).as_str().into()),
                        ("state", i18n.t(if digest.soil.trafficable {
                            "soil.trafficable"
                        } else {
                            "soil.compaction"
                        }).to_lowercase().as_str().into()),
                    ]))}
                    ton={match digest.soil.state {
                        SoilState::Sature => Ton::Mauvais,
                        SoilState::Sec => Ton::Attention,
                        SoilState::Ressuye => Ton::Bon,
                    }}
                />
                <Conseil
                    label={i18n.t("today.frost")}
                    value={i18n.t(match digest.frost.severity {
                        FrostSeverity::Aucun => "frost.aucun",
                        FrostSeverity::Faible => "frost.faible",
                        FrostSeverity::Modere => "frost.modere",
                        FrostSeverity::Severe => "frost.severe",
                    })}
                    detail={i18n.with("today.frost.detail", &params([
                        ("temperature", f.unit(digest.frost.min_temperature, "°C", 1).as_str().into()),
                        ("hoarFrost", if digest.frost.hoar_frost {
                            format!(" · {}", i18n.t("frost.hoarFrost")).into()
                        } else {
                            "".into()
                        }),
                    ]))}
                    ton={match digest.frost.severity {
                        FrostSeverity::Aucun => Ton::Bon,
                        FrostSeverity::Faible => Ton::Attention,
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
    value: AttrValue,
    detail: AttrValue,
}

#[function_component]
fn Fait(props: &FaitProps) -> Html {
    html! {
        <div class="fact">
            <dt>{ &props.label }</dt>
            <dd>
                <span class="fact__value">{ &props.value }</span>
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
