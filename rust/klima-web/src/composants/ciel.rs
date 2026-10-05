//! Le ciel vivant : derrière l'application, le temps qu'il fait dans la ville
//! affichée.
//!
//! Le jour, un soleil qui respire ; la nuit, des étoiles et la lune ; des
//! nuages d'autant plus nombreux et sombres que le ciel est couvert ; et ce qui
//! tombe — bruine, pluie, averses, neige —, les éclairs d'un orage, les bancs
//! d'un brouillard. Le tout d'après le code météo du moment : le fond dit la
//! même chose que la carte.
//!
//! Rien de net sous le texte : les cartes floutent ce qui passe derrière
//! elles. Tout bouge par `transform` et `opacity`, sur un calque fixe, et se
//! fige pour qui demande moins de mouvement.

use klima_core::weather::{ConditionIcon, weather_condition};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// `None` tant que la prévision n'est pas arrivée : un ciel de nuit calme.
    pub conditions: Option<(bool, u16)>,
}

/// Une suite pseudo-aléatoire fixe : le même ciel d'un rendu à l'autre.
struct Graine(u32);

impl Graine {
    fn suivant(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(self.0 >> 8) / f64::from(1u32 << 24)
    }
}

/// Ce qui tombe, et combien.
#[derive(Clone, Copy, PartialEq)]
enum Chute {
    Rien,
    Bruine,
    Pluie,
    Averse,
    Neige,
}

#[function_component]
pub fn CielVivant(props: &Props) -> Html {
    let (jour, icone) = match props.conditions {
        Some((jour, code)) => (jour, Some(weather_condition(code).icon)),
        None => (false, None),
    };

    // Le fond : trois dégradés superposés, et celui du moment apparaît en
    // fondu — un changement de ville ne fait pas sauter la couleur.
    let fond = match (jour, icone) {
        (false, _) | (_, None) => "nuit",
        (true, Some(ConditionIcon::Clear | ConditionIcon::Partly)) => "jour",
        (true, _) => "gris",
    };

    let nuages = match icone {
        None | Some(ConditionIcon::Clear) => 2,
        Some(ConditionIcon::Partly) => 3,
        Some(_) => 5,
    };
    let sombres = matches!(
        icone,
        Some(ConditionIcon::Rain | ConditionIcon::Showers | ConditionIcon::Thunder)
    );
    let chute = match icone {
        Some(ConditionIcon::Drizzle) => Chute::Bruine,
        Some(ConditionIcon::Rain) => Chute::Pluie,
        Some(ConditionIcon::Showers | ConditionIcon::Thunder) => Chute::Averse,
        Some(ConditionIcon::Snow) => Chute::Neige,
        _ => Chute::Rien,
    };
    let soleil = jour && matches!(icone, Some(ConditionIcon::Clear | ConditionIcon::Partly | ConditionIcon::Showers));
    let etoiles = !jour && matches!(icone, None | Some(ConditionIcon::Clear | ConditionIcon::Partly | ConditionIcon::Cloudy));

    html! {
        <div class="vivant" aria-hidden="true">
            <div class={classes!("vivant__fond", "vivant__fond--jour", (fond == "jour").then_some("is-on"))} />
            <div class={classes!("vivant__fond", "vivant__fond--gris", (fond == "gris").then_some("is-on"))} />
            <div class={classes!("vivant__fond", "vivant__fond--nuit", (fond == "nuit").then_some("is-on"))} />

            if soleil {
                <div class="vivant__soleil" />
            }
            if etoiles {
                { ciel_etoile(matches!(icone, Some(ConditionIcon::Cloudy))) }
            }
            { les_nuages(nuages, sombres) }
            if matches!(icone, Some(ConditionIcon::Fog)) {
                { le_brouillard() }
            }
            if chute != Chute::Rien {
                { ce_qui_tombe(chute) }
            }
            if matches!(icone, Some(ConditionIcon::Thunder)) {
                <div class="vivant__eclair" />
            }
        </div>
    }
}

fn ciel_etoile(voile: bool) -> Html {
    let mut graine = Graine(7_731);
    let etoiles: Html = (0..48)
        .map(|i| {
            let x = graine.suivant() * 100.0;
            let y = graine.suivant() * 62.0;
            let r = 0.5 + graine.suivant() * 1.0;
            let delai = graine.suivant() * 5.0;
            html! {
                <circle
                    cx={format!("{x:.2}%")} cy={format!("{y:.2}%")} r={format!("{r:.2}")}
                    class={classes!((i % 3 == 0).then_some("vivant__scintille"))}
                    style={format!("animation-delay: -{delai:.2}s")}
                />
            }
        })
        .collect();

    html! {
        <>
            <svg class={classes!("vivant__etoiles", voile.then_some("is-voile"))}>{ etoiles }</svg>
            <svg class="vivant__lune" viewBox="0 0 40 40">
                <path d="M28 30a14 14 0 1 1-6-26 11 11 0 0 0 6 26Z" fill="#e8edf2" />
            </svg>
        </>
    }
}

/// Des nuages à des hauteurs et des vitesses différentes, partis d'avance.
fn les_nuages(nombre: usize, sombres: bool) -> Html {
    const NUAGES: [(f64, f64, f64, f64); 5] = [
        // (haut en %, largeur en vw, durée en s, avance en s)
        (6.0, 58.0, 90.0, 20.0),
        (22.0, 44.0, 70.0, 50.0),
        (40.0, 64.0, 120.0, 95.0),
        (58.0, 40.0, 80.0, 10.0),
        (74.0, 54.0, 105.0, 60.0),
    ];
    let nuages: Html = NUAGES
        .iter()
        .take(nombre)
        .map(|&(haut, largeur, duree, avance)| {
            html! {
                <svg
                    class={classes!("vivant__nuage", sombres.then_some("is-sombre"))}
                    viewBox="0 0 200 80"
                    style={format!(
                        "top: {haut}%; width: max({largeur}vw, 300px); \
                         animation-duration: {duree}s; animation-delay: -{avance}s"
                    )}
                >
                    <ellipse cx="100" cy="50" rx="92" ry="26" fill="url(#vivantNuage)" />
                    <ellipse cx="74" cy="36" rx="44" ry="28" fill="url(#vivantNuage)" />
                    <ellipse cx="124" cy="30" rx="52" ry="30" fill="url(#vivantNuage)" />
                </svg>
            }
        })
        .collect();

    html! {
        <>
            <svg class="vivant__defs" width="0" height="0">
                <defs>
                    <radialGradient id="vivantNuage">
                        <stop offset="0%" stop-color="#ffffff" stop-opacity="1" />
                        <stop offset="60%" stop-color="#ffffff" stop-opacity="0.55" />
                        <stop offset="100%" stop-color="#ffffff" stop-opacity="0" />
                    </radialGradient>
                </defs>
            </svg>
            { nuages }
        </>
    }
}

fn le_brouillard() -> Html {
    html! {
        <>
            <div class="vivant__banc" style="top: 30%; animation-duration: 38s" />
            <div class="vivant__banc" style="top: 52%; animation-duration: 52s; animation-delay: -20s" />
            <div class="vivant__banc" style="top: 72%; animation-duration: 44s; animation-delay: -8s" />
        </>
    }
}

/// Les gouttes ou les flocons : placés une fois pour toutes, chacun à son
/// rythme, pour que la chute n'ait pas l'air d'un rideau.
fn ce_qui_tombe(chute: Chute) -> Html {
    let (nombre, classe, duree_min, duree_ecart) = match chute {
        Chute::Bruine => (36, "vivant__goutte vivant__goutte--fine", 1.1, 0.5),
        Chute::Pluie => (70, "vivant__goutte", 0.75, 0.35),
        Chute::Averse => (100, "vivant__goutte vivant__goutte--forte", 0.55, 0.25),
        Chute::Neige => (60, "vivant__flocon", 7.0, 6.0),
        Chute::Rien => return Html::default(),
    };
    let mut graine = Graine(42_424);
    let elements: Html = (0..nombre)
        .map(|_| {
            let x = graine.suivant() * 104.0 - 2.0;
            let duree = duree_min + graine.suivant() * duree_ecart;
            let delai = graine.suivant() * duree;
            let echelle = 0.6 + graine.suivant() * 0.6;
            html! {
                <span
                    class={classe}
                    style={format!(
                        "left: {x:.2}%; animation-duration: {duree:.2}s; \
                         animation-delay: -{delai:.2}s; --echelle: {echelle:.2}"
                    )}
                />
            }
        })
        .collect();

    html! { <div class="vivant__chute">{ elements }</div> }
}
