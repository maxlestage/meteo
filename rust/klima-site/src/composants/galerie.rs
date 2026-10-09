//! Dans votre poche : trois écrans de l'application, tels qu'un téléphone les
//! montre.
//!
//! Des captures de l'application elle-même, prises sur un après-midi qui
//! raconte quelque chose — une averse vers 17 h, du soleil, des graminées.
//! Rien à licencier : ce sont nos propres écrans. Elles se refont avec
//! `rust/scripts/captures.mjs` quand l'interface change.

use klima_ui::i18n::use_i18n;
use yew::prelude::*;

use crate::composants::film::mots;

/// (fichier, clé) : la clé donne le titre, le texte et la description.
const ECRANS: [(&str, &str); 3] =
    [("accueil.jpg", "home"), ("tuiles.jpg", "tiles"), ("sources.jpg", "sources")];

#[function_component]
pub fn Galerie() -> Html {
    let i18n = use_i18n();

    let ecrans: Html = ECRANS
        .iter()
        .enumerate()
        .map(|(i, (fichier, cle))| {
            html! {
                <figure class="shot" key={*cle} style={format!("--i: {i}")}>
                    <div class="shot__device" data-incline="" data-lueur="">
                        <img
                            src={format!("./captures/{fichier}")}
                            alt={i18n.t(&format!("gallery.{cle}.alt"))}
                            width="390"
                            height="844"
                            loading="lazy"
                            decoding="async"
                        />
                    </div>
                    <figcaption>
                        <h3>{ i18n.t(&format!("gallery.{cle}.title")) }</h3>
                        <p>{ i18n.t(&format!("gallery.{cle}.body")) }</p>
                    </figcaption>
                </figure>
            }
        })
        .collect();

    html! {
        <section class="gallery" id="images">
            <div class="section-head" data-apparait="titre">
                <h2>{ mots(&i18n.t("gallery.title")) }</h2>
                <p>{ i18n.t("gallery.lead") }</p>
            </div>
            <div class="gallery__grid" data-apparait="eventail">{ ecrans }</div>
        </section>
    }
}
