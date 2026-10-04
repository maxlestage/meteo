//! Ce que Klima calcule.
//!
//! Les seuils cités viennent du code et sont mis en forme dans la langue
//! courante : la page ne peut pas annoncer autre chose que ce que
//! l'application applique.

use klima_core::agro::thresholds::*;
use klima_core::i18n::{Params, params};
use klima_ui::i18n::use_i18n;
use yew::prelude::*;

#[function_component]
pub fn Fonctions() -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let kmh = |valeur: f64| f.unit(valeur, "km/h", 0);
    let celsius = |valeur: f64| f.unit(valeur, "°C", 0);

    let fonctions: Vec<(&str, Params, Params)> = vec![
        (
            "water",
            Params::new(),
            params([("deficit", f.unit(IRRIGATION_DEFICIT.abs(), "mm", 0).as_str().into())]),
        ),
        (
            "soil",
            Params::new(),
            params([
                ("wet", f.percent(SOIL_TOO_WET * 100.0).as_str().into()),
                ("dry", f.percent(SOIL_TOO_DRY * 100.0).as_str().into()),
            ]),
        ),
        (
            "spray",
            params([
                ("min", kmh(SPRAY_WIND_MIN).as_str().into()),
                ("max", kmh(SPRAY_WIND_MAX).as_str().into()),
                ("gusts", kmh(SPRAY_GUST_MAX).as_str().into()),
            ]),
            params([
                ("tempMin", celsius(SPRAY_TEMP_MIN).as_str().into()),
                ("tempMax", celsius(SPRAY_TEMP_MAX).as_str().into()),
                ("vpd", f.unit(SPRAY_VPD_MAX, "kPa", 1).as_str().into()),
            ]),
        ),
        (
            "disease",
            params([("humidity", f.percent(LEAF_WETNESS_HUMIDITY).as_str().into())]),
            params([
                ("min", celsius(DISEASE_TEMP_MIN).as_str().into()),
                ("max", celsius(DISEASE_TEMP_MAX).as_str().into()),
            ]),
        ),
        ("frost", Params::new(), Params::new()),
        (
            "gdd",
            params([("base", celsius(GDD_BASE).as_str().into())]),
            params([("ceiling", celsius(GDD_CEILING).as_str().into())]),
        ),
    ];

    let cartes: Html = fonctions
        .into_iter()
        .map(|(cle, regle, detail)| {
            html! {
                <article class="feature" key={cle}>
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
            <div class="section-head">
                <h2>{ i18n.t("features.title") }</h2>
                <p>{ i18n.t("features.lead") }</p>
            </div>
            <div class="features__grid">{ cartes }</div>
        </section>
    }
}
