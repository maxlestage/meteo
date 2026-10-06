//! L'application web de Klima.
//!

use klima_core::air::{ATTRIBUTION as AIR_ATTRIBUTION, pollen_dominant, qualite_air};
use klima_core::ville::{heure_de, niveau_uv};
use klima_core::endpoints::Endpoints;
use klima_core::i18n::params;
use klima_core::position::Parcelle;
use yew::prelude::*;

use crate::composants::bandeau::Bandeau;
use crate::composants::entete::Entete;
use crate::composants::jours::Jours;
use crate::composants::ciel::CielVivant;
use crate::composants::pluie::CartePluie;
use klima_ui::composants::langue::SelecteurDeLangue;
use klima_ui::composants::marque::MarqueEtNom;
use crate::composants::pro::NotePro;
use crate::composants::recherche::Recherche;
use crate::composants::sources::Sources;
use crate::composants::tuile::{Jauge, Tuile};
use klima_ui::crochets::palier::use_palier;
use klima_ui::crochets::parcelle::use_parcelle;
use klima_ui::crochets::position::use_start_position;
use klima_ui::crochets::prevision::use_forecast;
use klima_ui::crochets::direct::use_direct;
use klima_ui::crochets::veille::use_veille;
use klima_ui::composants::guetteur::Guetteur;
use klima_ui::dates;
use klima_ui::i18n::use_i18n;

/// Ville par défaut, quand on ne sait pas où est la personne.
fn defaut() -> Parcelle {
    Parcelle {
        name: "Paris".to_owned(),
        latitude: 48.8566,
        longitude: 2.3522,
        admin: Some("Île-de-France".to_owned()),
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
    let lieu = use_parcelle(defaut(), i18n.t("search.myField"));
    let palier = use_palier(props.relais.clone());

    // Première visite, rien de choisi ni de partagé : on part de là où est la
    // personne plutôt que de lui montrer Paris.
    use_start_position(lieu.origine, i18n.t("search.myField"), lieu.select.clone());

    // Le guetteur lit à part, au quart d'heure, et relit tout seul.
    // Le direct : avec un relais, la météo est poussée par WebSocket dès
    // qu'elle change — prévision, sources, quart d'heure, air.
    let direct = use_direct(lieu.parcelle.clone(), props.endpoints.clone(), 7);

    let veille = use_veille(lieu.parcelle.clone(), props.endpoints.clone(), direct.quarts.clone());

    let prevision = {
        let i18n = i18n.clone();
        use_forecast(lieu.parcelle.clone(), props.endpoints.clone(), 7, direct.clone(), move |erreur| {
            i18n.with(erreur.message_key(), &erreur.params())
        })
    };
    let etat = &prevision.etat;

    let recharger = {
        let reload = prevision.reload.clone();
        Callback::from(move |_| reload.emit(()))
    };

    let conditions =
        etat.forecast.as_ref().map(|f| (f.current.is_day, f.current.weather_code));

    html! {
        <div class="sky">
            // Le temps qu'il fait, derrière tout le reste.
            <CielVivant {conditions} />
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
                    // Des cartes fantômes qui miroitent : la page a déjà sa
                    // forme pendant que la prévision arrive.
                    <div class="attente" role="status">
                        <p class="attente__texte">{ i18n.t("app.loading") }</p>
                        <div class="attente__carte attente__carte--hero" />
                        <div class="attente__carte" />
                        <div class="attente__carte" />
                    </div>
                }

                if let Some(message) = &etat.error {
                    <div class="state state--error" role="alert">
                        <p>{ message }</p>
                        <button type="button" class="button" onclick={recharger.clone()}>
                            { i18n.t("app.retry") }
                        </button>
                    </div>
                }

                if let Some(forecast) = &etat.forecast {
                    // La clé est la ville : quand elle change, les cartes
                    // reviennent en cascade.
                    <div
                        class="contenu"
                        key={format!("{}:{}", forecast.parcelle.latitude, forecast.parcelle.longitude)}
                    >
                        <Entete forecast={forecast.clone()} />
                        // La demi-heure en cours et les deux heures qui
                        // viennent, au quart d'heure : la question qu'on pose
                        // la main sur la poignée.
                        <Guetteur
                            etat={veille.clone()}
                            ciel={forecast.ciel.clone()}
                            radar={etat.radar.clone()}
                            class={classes!("card")}
                        />
                        // La première question : faut-il un parapluie, et
                        // jusqu'à quand — puis ce qu'il faut emporter.
                        <CartePluie hours={forecast.hourly.clone()} />
                        <Bandeau
                            hours={forecast.hourly.clone()}
                            current={forecast.current.clone()}
                        />
                        <Jours
                            days={forecast.daily.clone()}
                            current_temperature={forecast.current.temperature}
                        />
                        // Qui annonce quoi pour la ville : la tuile d'accord
                        // le résume, cette carte le détaille.
                        if let Some(consensus) = &etat.consensus {
                            <Sources consensus={consensus.clone()} />
                        }
                        <div class="tiles">
                            <Tuile
                                label={i18n.t("tile.feelsLike")}
                                value={f.temperature(forecast.current.apparent_temperature)}
                                caption={i18n.with("tile.feelsLike.caption", &params([
                                    ("temperature", f.temperature(forecast.current.temperature).as_str().into()),
                                ]))}
                            />

                            <Tuile
                                label={i18n.t("tile.humidity")}
                                value={f.percent(forecast.current.relative_humidity)}
                                caption={heure_de(forecast.current.time, &forecast.hourly).map(|h| {
                                    AttrValue::from(i18n.with("tile.humidity.caption", &params([
                                        ("dewPoint", f.temperature(h.dew_point).as_str().into()),
                                    ])))
                                })}
                            />

                            <Tuile
                                label={i18n.t("tile.wind")}
                                value={f.unit(forecast.current.wind_speed, "km/h", 0)}
                                caption={i18n.with("tile.wind.caption", &params([
                                    ("gusts", f.unit(forecast.current.wind_gusts, "km/h", 0).as_str().into()),
                                ]))}
                            />

                            {{
                                let maintenant = heure_de(forecast.current.time, &forecast.hourly)
                                    .map_or(0.0, |h| h.uv_index);
                                let maximum = forecast.daily.first().map_or(maintenant, |d| d.uv_index_max);
                                let niveau = |indice: f64| i18n.t(&niveau_uv(indice).key());
                                html! {
                                    <Tuile
                                        label={i18n.t("tile.uv")}
                                        value={format!("{} · {}", f.decimal(maintenant, 0), niveau(maintenant))}
                                        caption={i18n.with("tile.uv.caption", &params([
                                            ("max", f.decimal(maximum, 0).as_str().into()),
                                            ("level", niveau(maximum).to_lowercase().as_str().into()),
                                        ]))}
                                        jauge={Jauge {
                                            position: maintenant / 11.0,
                                            gradient: "linear-gradient(to right, #7ed07a, #f0c14b, #ef8a5a, #d9534f, #9b59b6)",
                                        }}
                                    />
                                }
                            }}

                            if let Some(today) = forecast.daily.first() {
                                <Tuile
                                    label={i18n.t("tile.rainToday")}
                                    value={f.unit(today.precipitation_sum, "mm", 1)}
                                    caption={i18n.with("tile.rainToday.caption", &params([
                                        ("probability", f.percent(today.precipitation_probability_max).as_str().into()),
                                    ]))}
                                />
                            }

                            <Tuile
                                label={i18n.t("tile.pressure")}
                                value={f.unit(forecast.current.pressure, "hPa", 0)}
                                caption={i18n.t("tile.pressure.caption")}
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

                            if let Some(aqi) = etat.air.as_ref().and_then(|a| a.european_aqi) {
                                <Tuile
                                    label={i18n.t("tile.air")}
                                    value={i18n.t(&qualite_air(aqi).key())}
                                    caption={i18n.with("tile.air.caption", &params([
                                        ("aqi", f.decimal(aqi, 0).as_str().into()),
                                        ("pm25", etat.air.as_ref().and_then(|a| a.pm2_5)
                                            .map_or("—".to_owned(), |v| f.unit(v, "µg/m³", 0)).as_str().into()),
                                    ]))}
                                    jauge={Jauge {
                                        position: aqi / 100.0,
                                        gradient: "linear-gradient(to right, #50f0e6, #50ccaa, #f0e641, #ff5050, #960032, #7d2181)",
                                    }}
                                />
                            }

                            if let Some(air) = etat.air.as_ref().filter(|a| !a.pollens.is_empty()) {
                                if let Some((pollen, grains, niveau)) = pollen_dominant(air) {
                                    <Tuile
                                        label={i18n.t("tile.pollen")}
                                        value={i18n.t(&pollen.key())}
                                        caption={i18n.with("tile.pollen.caption", &params([
                                            ("grains", f.decimal(grains, 0).as_str().into()),
                                            ("level", i18n.t(&niveau.key()).to_lowercase().as_str().into()),
                                        ]))}
                                    />
                                } else {
                                    <Tuile
                                        label={i18n.t("tile.pollen")}
                                        value={i18n.t("tile.pollen.none")}
                                        caption={i18n.t("tile.pollen.noneCaption")}
                                    />
                                }
                            }

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
                        </div>

                        <NotePro palier={palier.clone()} />

                        <footer class="footer">
                            <p>{ format!("{}.", i18n.t("app.source")) }</p>
                            if etat.air.is_some() {
                                <p>{ format!("{AIR_ATTRIBUTION}.") }</p>
                            }
                            <button type="button" class="button" onclick={recharger}>
                                { i18n.t("app.refresh") }
                            </button>
                        </footer>
                    </div>
                }
            </div>
        </div>
    }
}

fn heure(ms: Option<i64>, locale: &str) -> String {
    match ms {
        Some(ms) => dates::heure_minute(ms, locale),
        None => "—".to_owned(),
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
