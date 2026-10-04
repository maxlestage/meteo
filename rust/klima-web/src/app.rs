//! L'application web de Klima.
//!

use klima_core::agro::summarize;
use klima_core::agro::thresholds::{GDD_BASE, SPRAY_WIND_MAX};
use klima_core::endpoints::Endpoints;
use klima_core::i18n::params;
use klima_core::position::Parcelle;
use yew::prelude::*;

use crate::composants::bandeau::Bandeau;
use crate::composants::entete::Entete;
use crate::composants::jours::Jours;
use klima_ui::composants::langue::SelecteurDeLangue;
use klima_ui::composants::marque::MarqueEtNom;
use crate::composants::pro::NotePro;
use crate::composants::recherche::Recherche;
use crate::composants::traitement::Traitement;
use crate::composants::tuile::{Jauge, Tuile};
use klima_ui::crochets::palier::use_palier;
use klima_ui::crochets::parcelle::use_parcelle;
use klima_ui::crochets::position::use_start_position;
use klima_ui::crochets::prevision::use_forecast;
use klima_ui::dates;
use klima_ui::i18n::use_i18n;

/// Parcelle par défaut : plaine céréalière de Beauce.
fn defaut() -> Parcelle {
    Parcelle {
        name: "Chartres".to_owned(),
        latitude: 48.4468,
        longitude: 1.4892,
        admin: Some("Centre-Val de Loire".to_owned()),
        country: Some("France".to_owned()),
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub endpoints: Endpoints,
    /// L'adresse du relais, quand il y en a un. Elle sert à lui demander le
    /// palier accordé — une question qui ne passe pas par les fournisseurs,
    /// et qui n'a donc pas sa place dans `Endpoints`.
    pub relais: Option<String>,
}

#[function_component]
pub fn App(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let lieu = use_parcelle(defaut());
    let palier = use_palier(props.relais.clone());

    // Première visite, rien de choisi ni de partagé : on part de là où est la
    // personne plutôt que de lui montrer la Beauce.
    use_start_position(lieu.origine, i18n.t("search.myField"), lieu.select.clone());

    let prevision = {
        let i18n = i18n.clone();
        use_forecast(lieu.parcelle.clone(), props.endpoints.clone(), 7, move |erreur| {
            i18n.with(erreur.message_key(), &erreur.params())
        })
    };
    let etat = &prevision.etat;

    // Les indicateurs se recalculent avec la prévision, pas à chaque rendu.
    let summary = use_memo(etat.forecast.clone(), |forecast| {
        forecast.as_ref().map(|f| summarize(&f.hourly, &f.daily))
    });

    let recharger = {
        let reload = prevision.reload.clone();
        Callback::from(move |_| reload.emit(()))
    };

    let ciel = match &etat.forecast {
        Some(forecast) => ciel_pour(forecast.current.is_day, forecast.current.weather_code),
        None => "night",
    };

    html! {
        <div class={format!("sky sky--{ciel}")}>
            <div class="shell">
                <header class="topbar">
                    // Le signe ramène à la vitrine. Sans lui l'application est
                    // un cul-de-sac : on y arrive par un lien partagé, un
                    // favori ou l'icône de l'écran d'accueil — sans historique
                    // à remonter.
                    <a class="topbar__home" href="../" aria-label={i18n.t("app.home")}>
                        <MarqueEtNom size={26} />
                    </a>
                    <SelecteurDeLangue class="lang" label={i18n.t("language.label")} />
                </header>

                <Recherche
                    current={lieu.parcelle.clone()}
                    endpoints={props.endpoints.clone()}
                    on_select={lieu.select.clone()}
                />

                if etat.loading && etat.forecast.is_none() {
                    <p class="state">{ i18n.t("app.loading") }</p>
                }

                if let Some(message) = &etat.error {
                    <div class="state state--error" role="alert">
                        <p>{ message }</p>
                        <button type="button" class="button" onclick={recharger.clone()}>
                            { i18n.t("app.retry") }
                        </button>
                    </div>
                }

                if let (Some(forecast), Some(summary)) = (&etat.forecast, &*summary) {
                    <>
                        <Entete forecast={forecast.clone()} />
                        <Bandeau
                            hours={forecast.hourly.clone()}
                            current={forecast.current.clone()}
                        />
                        <Jours
                            days={forecast.daily.clone()}
                            current_temperature={forecast.current.temperature}
                        />
                        <Traitement
                            hours={forecast.hourly.clone()}
                            next_spray={summary.next_spray.clone()}
                        />

                        <div class="tiles">
                            <Tuile
                                label={i18n.t("tile.soil")}
                                value={i18n.t(&format!("soil.{}", code_sol(summary.soil.state)))}
                                caption={i18n.with("tile.soil.caption", &params([
                                    ("moisture", f.percent(summary.soil.moisture * 100.0).as_str().into()),
                                    ("temperature", f.unit(summary.soil.temperature, "°C", 1).as_str().into()),
                                    ("state", i18n.t(if summary.soil.trafficable {
                                        "soil.trafficable"
                                    } else {
                                        "soil.compaction"
                                    }).as_str().into()),
                                ]))}
                                jauge={Jauge {
                                    position: summary.soil.moisture / 0.5,
                                    gradient: "linear-gradient(to right, #d8b36a 0%, #8fc46a 30%, #4aa3d8 70%, #2b5f9e 100%)",
                                }}
                            />

                            <Tuile
                                label={i18n.t("tile.water")}
                                value={f.signed_unit(summary.water.balance, "mm", 1)}
                                caption={
                                    if summary.water.irrigation_advice > 0.0 {
                                        i18n.with("tile.water.irrigation", &params([
                                            ("amount", f.unit(summary.water.irrigation_advice, "mm", 0).as_str().into()),
                                        ]))
                                    } else {
                                        i18n.with("tile.water.caption", &params([
                                            ("rain", f.unit(summary.water.precipitation, "mm", 1).as_str().into()),
                                            ("et0", f.unit(summary.water.evapotranspiration, "mm", 1).as_str().into()),
                                        ]))
                                    }
                                }
                            />

                            <Tuile
                                label={i18n.t("tile.wind")}
                                value={f.unit(forecast.current.wind_speed, "km/h", 0)}
                                caption={i18n.with("tile.wind.caption", &params([
                                    ("gusts", f.unit(forecast.current.wind_gusts, "km/h", 0).as_str().into()),
                                    ("limit", f.unit(SPRAY_WIND_MAX, "km/h", 0).as_str().into()),
                                ]))}
                            />

                            <Tuile
                                label={i18n.t("tile.frost")}
                                value={i18n.t(&format!("frost.{}", code_gel(summary.frost.severity)))}
                                caption={i18n.with("tile.frost.caption", &params([
                                    ("temperature", f.unit(summary.frost.min_temperature, "°C", 1).as_str().into()),
                                    ("hoarFrost", if summary.frost.hoar_frost {
                                        format!(", {}", i18n.t("frost.hoarFrost")).into()
                                    } else {
                                        "".into()
                                    }),
                                ]))}
                            />

                            <Tuile
                                label={i18n.t("tile.disease")}
                                value={i18n.t(&format!("disease.{}", code_maladie(summary.disease.level)))}
                                caption={i18n.with("tile.disease.caption", &params([
                                    ("hours", summary.disease.leaf_wetness_hours.into()),
                                ]))}
                            />

                            <Tuile
                                label={i18n.t("tile.gdd")}
                                value={f.unit(summary.gdd, "°C·j", 1)}
                                caption={i18n.with("tile.gdd.caption", &params([
                                    ("base", f.unit(GDD_BASE, "°C", 0).as_str().into()),
                                ]))}
                            />

                            <Tuile
                                label={i18n.t("tile.sunrise")}
                                value={heure(forecast.daily.first().and_then(|d| d.sunrise), i18n.locale())}
                                caption={i18n.with("tile.sunrise.caption", &params([
                                    ("time", heure(
                                        forecast.daily.first().and_then(|d| d.sunset),
                                        i18n.locale(),
                                    ).as_str().into()),
                                ]))}
                            />

                            if let Some(consensus) = &etat.consensus {
                                <Tuile
                                    label={i18n.t("consensus.title")}
                                    value={i18n.t(&format!("consensus.{}", code_accord(consensus.agreement)))}
                                    caption={format!(
                                        "{}{}. {}.",
                                        i18n.with("consensus.detail", &params([
                                            ("count", consensus.readings.len().into()),
                                            ("spread", f.unit(consensus.temperature.spread, "°C", 1).as_str().into()),
                                        ])),
                                        if consensus.agree_on_rain {
                                            String::new()
                                        } else {
                                            format!(" · {}", i18n.t("consensus.rainDisagreement"))
                                        },
                                        i18n.with("consensus.median", &params([
                                            ("value", f.unit(consensus.temperature.median, "°C", 1).as_str().into()),
                                        ])),
                                    )}
                                    jauge={Jauge {
                                        position: 1.0 - (consensus.temperature.spread / 5.0).min(1.0),
                                        gradient: "linear-gradient(to right, #ef8a5a, #f0c14b, #7ed07a)",
                                    }}
                                />
                            }

                            <Tuile
                                label={i18n.t("tile.sowing")}
                                value={i18n.t(if summary.soil.sowable {
                                    "tile.sowing.yes"
                                } else {
                                    "tile.sowing.no"
                                })}
                                caption={i18n.with("tile.sowing.caption", &params([
                                    ("temperature", f.unit(summary.soil.temperature, "°C", 1).as_str().into()),
                                ]))}
                            />
                        </div>

                        <NotePro palier={palier.clone()} />

                        <footer class="footer">
                            <p>{ format!("{}.", i18n.t("app.source")) }</p>
                            <button type="button" class="button" onclick={recharger}>
                                { i18n.t("app.refresh") }
                            </button>
                        </footer>
                    </>
                }
            </div>
        </div>
    }
}

/// Le fond suit le ciel : nuit, journée couverte ou journée dégagée.
fn ciel_pour(is_day: bool, code: u16) -> &'static str {
    if !is_day {
        return "night";
    }
    if code >= 45 { "grey" } else { "day" }
}

fn heure(ms: Option<i64>, locale: &str) -> String {
    match ms {
        Some(ms) => dates::heure_minute(ms, locale),
        None => "—".to_owned(),
    }
}

fn code_sol(etat: klima_core::agro::SoilState) -> &'static str {
    use klima_core::agro::SoilState::*;
    match etat {
        Sature => "sature",
        Ressuye => "ressuye",
        Sec => "sec",
    }
}

fn code_gel(severite: klima_core::agro::FrostSeverity) -> &'static str {
    use klima_core::agro::FrostSeverity::*;
    match severite {
        Aucun => "aucun",
        Faible => "faible",
        Modere => "modere",
        Severe => "severe",
    }
}

fn code_maladie(niveau: klima_core::agro::DiseaseLevel) -> &'static str {
    use klima_core::agro::DiseaseLevel::*;
    match niveau {
        Faible => "faible",
        Moyenne => "moyenne",
        Elevee => "elevee",
    }
}

fn code_accord(accord: klima_core::consensus::Agreement) -> &'static str {
    use klima_core::consensus::Agreement::*;
    match accord {
        Forte => "forte",
        Moyenne => "moyenne",
        Faible => "faible",
    }
}
