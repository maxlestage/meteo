//! La vitrine de Klima.
//!

use klima_api::today::day_digest;
use klima_core::endpoints::Endpoints;
use klima_core::i18n::params;
use klima_core::ville::seuils::{PLUIE_MM, PLUIE_PROBABILITE, UV_CREME};
use klima_core::position::Parcelle;
use klima_core::weather::{ConditionIcon, weather_condition};
use klima_ui::composants::langue::SelecteurDeLangue;
use klima_ui::composants::marque::MarqueEtNom;
use klima_ui::crochets::parcelle::use_parcelle;
use klima_ui::crochets::position::use_start_position;
use klima_ui::crochets::prevision::use_forecast;
use klima_ui::crochets::direct::use_direct;
use klima_ui::crochets::veille::use_veille;
use klima_ui::composants::guetteur::Guetteur;
use klima_ui::composants::vu::CielVu;
use klima_ui::i18n::use_i18n;
use yew::prelude::*;

use crate::composants::aujourdhui::SectionDuJour;
use crate::composants::ciel::{Ciel, Horizon};
use crate::composants::fonctions::Fonctions;
use crate::composants::galerie::Galerie;
use crate::composants::illustrations::{EchelleUv, IconeFonction, SceneDeCiel, ScenePluie};
use crate::composants::pied::Pied;
use crate::composants::sources::Sources;
use crate::composants::telephone::Telephone;
use crate::crochets::apparition::use_apparition;

/// Paris, au premier chargement, quand on ne sait pas où est le visiteur.
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
}

#[function_component]
pub fn App(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();

    // La ville consultée vit dans l'adresse : le bouton retour la défait, et
    // l'adresse envoyée à quelqu'un lui montre bien la ville qu'on a
    // regardée.
    let lieu = use_parcelle(defaut(), i18n.t("search.myField"));

    // La vitrine montre une vraie prévision : autant que ce soit celle du
    // visiteur. Un refus laisse la ville par défaut.
    use_start_position(lieu.origine, i18n.t("search.myField"), lieu.select.clone());

    // Deux jours suffisent : aujourd'hui, et les douze heures qui débordent
    // sur demain pour la pluie et les conseils.
    // Le direct : avec un relais, la météo est poussée par WebSocket dès
    // qu'elle change — prévision, sources, quart d'heure, air.
    let direct = use_direct(lieu.parcelle.clone(), props.endpoints.clone(), 2);

    let prevision = {
        let i18n = i18n.clone();
        use_forecast(lieu.parcelle.clone(), props.endpoints.clone(), 2, direct.clone(), move |erreur| {
            i18n.with(erreur.message_key(), &erreur.params())
        })
    };
    // Le guetteur : la demi-heure en cours et les deux heures, au quart
    // d'heure, relus tout seuls.
    let veille = use_veille(lieu.parcelle.clone(), props.endpoints.clone(), direct.quarts.clone());
    let etat = &prevision.etat;

    let digest = use_memo(etat.forecast.clone(), |forecast| {
        forecast.as_ref().and_then(day_digest)
    });

    let courant = etat.forecast.as_ref().map(|f| f.current.clone());
    let pluie = courant.as_ref().is_some_and(|c| {
        matches!(
            weather_condition(c.weather_code).icon,
            ConditionIcon::Drizzle | ConditionIcon::Rain | ConditionIcon::Showers
                | ConditionIcon::Thunder
        )
    });

    let fonctions = use_apparition();
    let donnees = use_apparition();

    html! {
        <>
            <Ciel />
            <header class="nav">
                <a class="nav__brand" href="#top" aria-label="Klima">
                    <MarqueEtNom />
                </a>
                <div class="nav__end">
                    <nav>
                        <a href="#images">{ i18n.t("nav.images") }</a>
                        <a href="#aujourdhui">{ i18n.t("nav.today") }</a>
                        <a href="#indicateurs">{ i18n.t("nav.indicators") }</a>
                        <a href="#sources">{ i18n.t("nav.sources") }</a>
                        <a href="#donnees">{ i18n.t("nav.data") }</a>
                    </nav>
                    // Sur téléphone, le bouton dit « L'app » : la marque, le bouton
                    // et les trois langues tiennent alors sur une ligne.
                    <a class="button button--compact" href="./app/" aria-label={i18n.t("app.open")}>
                        <span class="nav__app-long">{ i18n.t("app.open") }</span>
                        <span class="nav__app-court" aria-hidden="true">{ i18n.t("app.openShort") }</span>
                    </a>
                    <SelecteurDeLangue class="lang" label={i18n.t("language.label")} />
                </div>
            </header>

            <main id="top">
                <section class="hero">
                    <div class="hero__text">
                        <p class="hero__eyebrow">{ i18n.t("hero.eyebrow") }</p>
                        <h1>{ i18n.t("hero.title") }</h1>
                        <p class="hero__lead">{ i18n.t("hero.lead") }</p>
                        <div class="hero__actions">
                            <a class="button" href="#aujourdhui">{ i18n.t("hero.cta.today") }</a>
                            <a class="button button--ghost" href="#indicateurs">
                                { i18n.t("hero.cta.indicators") }
                            </a>
                        </div>
                        <p class="hero__note">{ i18n.t("hero.note") }</p>
                    </div>

                    <Telephone
                        parcelle={lieu.parcelle.clone()}
                        digest={(*digest).clone()}
                        current={courant.clone()}
                    />
                </section>

                <div class="banner">
                    <SceneDeCiel
                        is_day={courant.as_ref().map(|c| c.is_day).unwrap_or(true)}
                        raining={pluie}
                    />
                </div>

                <Galerie />

                <SectionDuJour
                    parcelle={lieu.parcelle.clone()}
                    digest={(*digest).clone()}
                    current={courant}
                    sources={etat.forecast.as_ref().map_or(0, |f| f.sources.len())}
                    loading={etat.loading}
                    error={etat.error.clone()}
                    endpoints={props.endpoints.clone()}
                    on_select={lieu.select.clone()}
                    on_retry={prevision.reload.clone()}
                >
                    <CielVu
                        ciel={etat.forecast.as_ref().and_then(|f| f.ciel.clone())}
                        class={classes!("today__vu")}
                    />
                    <Guetteur
                        etat={veille}
                        tombe={etat.forecast.as_ref().and_then(|f| f.ciel.as_ref()).and_then(|c| c.tombe)}
                        class={classes!("today__guetteur")}
                    />
                </SectionDuJour>

                <Sources consensus={etat.consensus.clone()} loading={etat.loading} />

                <div ref={fonctions.node} class={fonctions.class}>
                    <Fonctions />
                    <div class="figures">
                        <figure>
                            <ScenePluie />
                            <figcaption>
                                { i18n.with("feature.rain.rule", &params([
                                    ("amount", f.unit(PLUIE_MM, "mm", 1).as_str().into()),
                                    ("probability", f.percent(PLUIE_PROBABILITE).as_str().into()),
                                ])) }
                            </figcaption>
                        </figure>
                        <figure>
                            <EchelleUv />
                            <figcaption>
                                { i18n.with("figure.uv", &params([
                                    ("sunscreen", f.decimal(UV_CREME, 0).as_str().into()),
                                ])) }
                            </figcaption>
                        </figure>
                    </div>
                </div>

                <section class="data" id="donnees" ref={donnees.node}>
                    <div class="section-head">
                        <h2>{ i18n.t("data.title") }</h2>
                    </div>
                    <div class={classes!("data__grid", donnees.class)}>
                        <article>
                            <IconeFonction cle="model" />
                            <h3>{ i18n.t("data.model.title") }</h3>
                            <p>{ i18n.t("data.model.body") }</p>
                        </article>
                        <article>
                            <IconeFonction cle="rules" />
                            <h3>{ i18n.t("data.rules.title") }</h3>
                            <p>{ i18n.t("data.rules.body") }</p>
                        </article>
                        <article>
                            <IconeFonction cle="hours" />
                            <h3>{ i18n.t("data.hours.title") }</h3>
                            <p>{ i18n.t("data.hours.body") }</p>
                        </article>
                    </div>
                </section>
            </main>

            <Pied />
            <Horizon />
        </>
    }
}
