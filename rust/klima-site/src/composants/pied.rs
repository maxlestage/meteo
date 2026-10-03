//! Le pied de page : ce que le produit couvre, d'où viennent ses chiffres, qui
//! l'a fait, et ce qu'il ne prétend pas remplacer.
//!
//! Miroir de `site/src/components/Footer.tsx`.
//!
//! Pas de lien vers le code source : c'est une règle du dépôt, et un test du
//! catalogue la tient.

use klima_core::i18n::{LANGUAGES, params};
use klima_ui::composants::marque::Marque;
use klima_ui::i18n::use_i18n;
use yew::prelude::*;

#[function_component]
pub fn Pied() -> Html {
    let i18n = use_i18n();
    let annee = js_sys::Date::new_0().get_full_year();

    let langues: Html = LANGUAGES
        .into_iter()
        .map(|candidate| {
            let courante = candidate == i18n.language;
            let onclick = {
                let set = i18n.set_language.clone();
                Callback::from(move |_| set.emit(candidate))
            };
            html! {
                <li key={candidate.code()}>
                    <button
                        type="button"
                        class={classes!(courante.then_some("is-current"))}
                        aria-current={courante.then(|| AttrValue::from("true"))}
                        {onclick}
                    >
                        { candidate.name() }
                    </button>
                </li>
            }
        })
        .collect();

    html! {
        <footer class="footer">
            <div class="footer__grid">
                <div class="footer__brand">
                    <span class="brand">
                        <Marque size={30} />
                        <span class="footer__wordmark">{ "Klima" }</span>
                    </span>
                    <p class="footer__tagline">{ i18n.t("footer.tagline") }</p>
                    <ul class="footer__languages">{ langues }</ul>
                </div>

                <nav class="footer__column" aria-label={i18n.t("footer.product")}>
                    <h2>{ i18n.t("footer.product") }</h2>
                    <ul>
                        <li><a href="#aujourdhui">{ i18n.t("nav.today") }</a></li>
                        <li><a href="#indicateurs">{ i18n.t("nav.indicators") }</a></li>
                        <li><a href="#sources">{ i18n.t("nav.sources") }</a></li>
                        <li><a href="#donnees">{ i18n.t("nav.data") }</a></li>
                        <li><a href="./app/">{ i18n.t("footer.app") }</a></li>
                    </ul>
                </nav>

                <div class="footer__column">
                    <h2>{ i18n.t("footer.dataTitle") }</h2>
                    <ul>
                        <li>
                            <a href="https://open-meteo.com/" rel="noreferrer noopener" target="_blank">
                                { "Open-Meteo" }
                            </a>
                        </li>
                        <li>
                            <a href="https://api.met.no/" rel="noreferrer noopener" target="_blank">
                                { "MET Norway" }
                            </a>
                        </li>
                        <li>
                            <a href="https://brightsky.dev/" rel="noreferrer noopener" target="_blank">
                                { "Bright Sky / DWD" }
                            </a>
                        </li>
                        <li>{ i18n.t("footer.models") }</li>
                        <li>{ i18n.t("footer.method") }</li>
                    </ul>
                </div>

                <div class="footer__column footer__column--credits">
                    <h2>{ i18n.t("footer.credits") }</h2>
                    <p class="footer__author">{ i18n.t("footer.author") }</p>
                    <p class="footer__role">{ i18n.t("footer.role") }</p>
                </div>
            </div>

            <div class="footer__bottom">
                <p>{ i18n.with("footer.rights", &params([("year", (annee as i32).into())])) }</p>
                <p class="footer__legal">{ i18n.t("footer.legal") }</p>
            </div>
        </footer>
    }
}
