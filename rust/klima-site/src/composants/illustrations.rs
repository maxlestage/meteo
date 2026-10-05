//! Illustrations vectorielles, dessinées à la main.
//!
//! Pas de photographies : rien à licencier, rien à charger, et le trait reste
//! net à toutes les tailles. Les animations sont en CSS et se coupent
//! d'elles-mêmes si le système demande moins de mouvement.

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CielProps {
    /// Vrai le jour : le soleil remplace la lune.
    #[prop_or(true)]
    pub is_day: bool,
    /// Vrai quand la pluie tombe sur la ville.
    #[prop_or(false)]
    pub raining: bool,
}

/// Scène de tête : un bandeau large, dessiné au format où il s'affiche. Le
/// gabarit est volontairement plat (420 × 96) — un carré rogné dans une bande
/// perdrait le soleil et l'horizon.
#[function_component]
pub fn SceneDeCiel(props: &CielProps) -> Html {
    let pluie: Html = (0..14)
        .map(|i| {
            let x = 30 + i * 27;
            html! {
                <line
                    x1={x.to_string()} y1="40"
                    x2={(x - 3).to_string()} y2="50"
                    style={format!("animation-delay: {}s", f64::from(i % 7) * 0.18)}
                />
            }
        })
        .collect();

    // Une ligne d'immeubles : hauteurs et largeurs fixées une fois pour
    // toutes, pour que la silhouette ne change pas d'un rendu à l'autre.
    const IMMEUBLES: [(i32, i32, i32); 14] = [
        (0, 30, 22), (30, 22, 34), (52, 34, 18), (86, 26, 40), (112, 40, 26),
        (152, 24, 30), (176, 30, 44), (206, 36, 22), (242, 22, 36), (264, 34, 28),
        (298, 26, 42), (324, 40, 20), (364, 24, 32), (388, 32, 26),
    ];
    let immeubles: Html = IMMEUBLES
        .iter()
        .map(|&(x, largeur, hauteur)| {
            html! {
                <rect
                    x={x.to_string()} y={(96 - hauteur).to_string()}
                    width={(largeur - 2).to_string()} height={hauteur.to_string()}
                    rx="1.5"
                />
            }
        })
        .collect();

    // Quelques fenêtres allumées, qui s'éteignent et se rallument à leur
    // rythme — la nuit seulement : le jour, on ne les voit pas.
    let fenetres: Html = IMMEUBLES
        .iter()
        .enumerate()
        .flat_map(|(i, &(x, largeur, hauteur))| {
            let colonnes = ((largeur - 6) / 7).max(1);
            let rangees = ((hauteur - 8) / 8).max(1);
            (0..colonnes * rangees)
                // Une fenêtre sur trois, choisie sans hasard.
                .filter(move |k| (*k as usize + i) % 3 == 0)
                .map(move |k| {
                    let cx = x + 4 + (k % colonnes) * 7;
                    let cy = 96 - hauteur + 5 + (k / colonnes) * 8;
                    html! {
                        <rect
                            x={cx.to_string()} y={cy.to_string()} width="3" height="3.5"
                            style={format!("animation-delay: {}s", f64::from((k + i as i32) % 9) * 0.7)}
                        />
                    }
                })
        })
        .collect();

    html! {
        <svg
            class="scene"
            viewBox="0 0 420 96"
            role="img"
            aria-label="La ville sous un ciel changeant"
            preserveAspectRatio="xMidYMid slice"
        >
            <defs>
                <linearGradient id="skyGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="#3c556e" />
                    <stop offset="100%" stop-color="#1d2836" />
                </linearGradient>
                <radialGradient id="sunGlow">
                    <stop offset="0%" stop-color="#f7c948" stop-opacity="0.45" />
                    <stop offset="100%" stop-color="#f7c948" stop-opacity="0" />
                </radialGradient>
                <linearGradient id="cityGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="#2a3a4d" />
                    <stop offset="100%" stop-color="#141d28" />
                </linearGradient>
            </defs>

            <rect width="420" height="96" fill="url(#skyGradient)" />

            if props.is_day {
                <g class="scene__sun">
                    <circle cx="352" cy="30" r="13" fill="#f7c948" />
                    <circle cx="352" cy="30" r="30" fill="url(#sunGlow)" />
                </g>
            } else {
                <g class="scene__moon">
                    <path d="M360 38a14 14 0 1 1-12-18 11 11 0 0 0 12 18Z" fill="#e8edf2" />
                </g>
            }

            // Trois nuages à des vitesses différentes : le ciel n'est jamais figé.
            <g class="scene__cloud scene__cloud--slow" fill="#c6d6e4" opacity="0.3">
                <ellipse cx="88" cy="30" rx="34" ry="10" />
                <ellipse cx="112" cy="25" rx="22" ry="11" />
                <ellipse cx="64" cy="26" rx="18" ry="9" />
            </g>
            <g class="scene__cloud scene__cloud--fast" fill="#c6d6e4" opacity="0.22">
                <ellipse cx="238" cy="19" rx="27" ry="8" />
                <ellipse cx="258" cy="16" rx="17" ry="8" />
            </g>
            <g class="scene__cloud scene__cloud--slower" fill="#c6d6e4" opacity="0.16">
                <ellipse cx="168" cy="44" rx="30" ry="8" />
                <ellipse cx="192" cy="41" rx="19" ry="8" />
            </g>

            if props.raining {
                <g
                    class="scene__rain"
                    stroke="#7fd0f5"
                    stroke-width="1.6"
                    stroke-linecap="round"
                >
                    { pluie }
                </g>
            }

            // La ville : une ligne d'immeubles, et leurs fenêtres la nuit.
            <g fill="url(#cityGradient)">{ immeubles }</g>
            if !props.is_day {
                <g class="scene__windows" fill="#f7d77a">{ fenetres }</g>
            }
        </svg>
    }
}

/// La pluie qui vient : un parapluie, et douze barres — une par heure —
/// aussi hautes que le risque. La même lecture que dans l'application.
#[function_component]
pub fn ScenePluie() -> Html {
    // Un après-midi type : sec, puis une averse vers 17 h, puis l'accalmie.
    const RISQUES: [i32; 12] = [8, 10, 12, 18, 35, 62, 84, 78, 46, 24, 12, 8];
    let barres: Html = RISQUES
        .iter()
        .enumerate()
        .map(|(i, &risque)| {
            let hauteur = risque.max(4);
            let mouillee = risque >= 50;
            html! {
                <rect
                    class="rain-scene__bar"
                    x={(24 + i as i32 * 24).to_string()} y={(148 - hauteur).to_string()}
                    width="16" height={hauteur.to_string()} rx="3"
                    fill={if mouillee { "#7fd0f5" } else { "rgba(255,255,255,0.28)" }}
                    style={format!("animation-delay: {}s", i as f64 * 0.08)}
                />
            }
        })
        .collect();

    html! {
        <svg
            class="rain-scene"
            viewBox="0 0 320 160"
            role="img"
            aria-label="Le risque de pluie heure par heure, sur douze heures"
        >
            // Le parapluie.
            <g transform="translate(250 18)">
                <path d="M0 26 A30 26 0 0 1 60 26 Q52 20 45 26 Q37 20 30 26 Q22 20 15 26 Q8 20 0 26 Z" fill="#4aa3d8" />
                <path d="M30 26 V58 a6 6 0 0 1 -12 0" fill="none" stroke="#e8edf2" stroke-width="2.6" stroke-linecap="round" />
            </g>
            <line x1="20" y1="148" x2="308" y2="148" stroke="rgba(255,255,255,0.3)" stroke-width="1" />
            { barres }
        </svg>
    }
}

/// L'échelle UV de l'Organisation mondiale de la santé : cinq bandes, et le
/// curseur du jour.
#[function_component]
pub fn EchelleUv() -> Html {
    const BANDES: [(&str, i32); 5] =
        [("#7ed07a", 3), ("#f0c14b", 3), ("#ef8a5a", 2), ("#d9534f", 3), ("#9b59b6", 2)];
    let mut debut = 0;
    let bandes: Html = BANDES
        .iter()
        .map(|&(couleur, largeur)| {
            let x = 20 + debut * 21;
            debut += largeur;
            html! { <rect x={x.to_string()} y="112" width={(largeur * 21).to_string()} height="14" fill={couleur} /> }
        })
        .collect();

    html! {
        <svg
            class="uv-scene"
            viewBox="0 0 320 160"
            role="img"
            aria-label="L’échelle UV de l’OMS, de faible à extrême"
        >
            <g class="uv-scene__sun">
                <circle cx="160" cy="56" r="22" fill="#f7c948" />
                <g stroke="#f7c948" stroke-width="3" stroke-linecap="round">
                    <line x1="160" y1="14" x2="160" y2="24" />
                    <line x1="160" y1="88" x2="160" y2="98" />
                    <line x1="118" y1="56" x2="128" y2="56" />
                    <line x1="192" y1="56" x2="202" y2="56" />
                    <line x1="130" y1="26" x2="137" y2="33" />
                    <line x1="183" y1="79" x2="190" y2="86" />
                    <line x1="130" y1="86" x2="137" y2="79" />
                    <line x1="183" y1="33" x2="190" y2="26" />
                </g>
            </g>
            <g>{ bandes }</g>
            // Le curseur : un indice 6, « élevé ».
            <path class="uv-scene__cursor" d="M146 108 l6 -10 l6 10 Z" fill="#e8edf2" />
            <text x="20" y="146" class="uv-scene__label">{ "0" }</text>
            <text x="300" y="146" class="uv-scene__label" text-anchor="end">{ "11+" }</text>
        </svg>
    }
}
