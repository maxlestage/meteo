//! Ce que Klima vous dit.
//!
//! Les seuils cités viennent du code et sont mis en forme dans la langue
//! courante : la page ne peut pas annoncer autre chose que ce que
//! l'application applique.

use klima_core::alerts::seuils::{CHALEUR, RAFALES};
use klima_core::i18n::{Params, params};
use klima_core::veille::seuils::{FORTE_MM_H, PLUIE_QUART_MM};
use klima_core::ville::seuils::{
    HORIZON_PLUIE, MANTEAU_RESSENTI, PLUIE_MM, PLUIE_PROBABILITE, UV_CREME, UV_LUNETTES,
};
use klima_ui::i18n::use_i18n;

use crate::composants::film::mots;
use crate::composants::illustrations::IconeFonction;
use yew::prelude::*;

#[function_component]
pub fn Fonctions() -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let celsius = |valeur: f64| f.unit(valeur, "°C", 0);

    let fonctions: Vec<(&str, Params, Params)> = vec![
        // Le guetteur ouvre la liste, sur toute la largeur : c'est lui qui
        // répond à la question la plus pressée.
        (
            "veille",
            params([
                ("amount", f.unit(PLUIE_QUART_MM, "mm", 1).as_str().into()),
                ("heavy", f.unit(FORTE_MM_H, "mm", 1).as_str().into()),
            ]),
            Params::new(),
        ),
        (
            "rain",
            params([
                ("amount", f.unit(PLUIE_MM, "mm", 1).as_str().into()),
                ("probability", f.percent(PLUIE_PROBABILITE).as_str().into()),
            ]),
            params([("hours", HORIZON_PLUIE.into())]),
        ),
        (
            "advice",
            params([
                ("coat", celsius(MANTEAU_RESSENTI).as_str().into()),
                ("glasses", f.decimal(UV_LUNETTES, 0).as_str().into()),
                ("sunscreen", f.decimal(UV_CREME, 0).as_str().into()),
            ]),
            Params::new(),
        ),
        ("uv", Params::new(), Params::new()),
        ("air", Params::new(), Params::new()),
        ("pollen", Params::new(), Params::new()),
        (
            "alerts",
            params([
                ("heat", celsius(CHALEUR).as_str().into()),
                ("gusts", f.unit(RAFALES, "km/h", 0).as_str().into()),
            ]),
            Params::new(),
        ),
    ];

    let cartes: Html = fonctions
        .into_iter()
        .enumerate()
        .map(|(i, (cle, regle, detail))| {
            html! {
                <article
                    class={classes!("feature", (cle == "veille").then_some("feature--large"))}
                    key={cle}
                    style={format!("--i: {i}")}
                    data-lueur=""
                >
                    <IconeFonction cle={cle} />
                    <h3>{ i18n.t(&format!("feature.{cle}.title")) }</h3>
                    <p class="feature__rule">
                        { i18n.with(&format!("feature.{cle}.rule"), &regle) }
                    </p>
                    <p class="feature__detail">
                        { i18n.with(&format!("feature.{cle}.detail"), &detail) }
                    </p>
                </article>
            }
        })
        .collect();

    html! {
        <section class="features" id="indicateurs">
            <div class="section-head" data-apparait="titre">
                <h2>{ mots(&i18n.t("features.title")) }</h2>
                <p>{ i18n.t("features.lead") }</p>
            </div>
            <div class="features__grid" data-apparait="cascade">{ cartes }</div>
        </section>
    }
}
