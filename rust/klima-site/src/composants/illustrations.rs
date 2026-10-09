//! Illustrations vectorielles, dessinées à la main.
//!
//! Pas de photographies : rien à licencier, rien à charger, et le trait reste
//! net à toutes les tailles. Chaque dessin bouge à sa façon — les gouttes
//! tombent, le parapluie se balance, la cloche sonne. Les animations sont en
//! CSS, ne partent qu'une fois le dessin à l'écran, et se coupent d'elles-mêmes
//! si le système demande moins de mouvement.

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CielProps {
    /// Vrai le jour : le soleil remplace la lune.
    #[prop_or(true)]
    pub is_day: bool,
    /// Vrai quand la pluie tombe sur la ville.
    #[prop_or(false)]
    pub raining: bool,
    /// Vrai quand il tonne : le ciel s'éclaire de temps en temps.
    #[prop_or(false)]
    pub orage: bool,
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

            // Le jour, trois oiseaux traversent, chacun à son heure.
            if props.is_day {
                <g class="scene__oiseaux" fill="none" stroke="#1d2836" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" opacity="0.55">
                    <g class="scene__oiseau" style="--y: 22px; animation-delay: -4s">
                        <path d="M0 0q3-3 6 0q3-3 6 0" />
                    </g>
                    <g class="scene__oiseau" style="--y: 32px; animation-delay: -9s; animation-duration: 31s">
                        <path d="M0 0q2.4-2.4 4.8 0q2.4-2.4 4.8 0" />
                    </g>
                    <g class="scene__oiseau" style="--y: 15px; animation-delay: -19s; animation-duration: 24s">
                        <path d="M0 0q2-2 4 0q2-2 4 0" />
                    </g>
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

            // L'orage : un éclair blanchit le ciel, de loin en loin.
            if props.orage {
                <rect class="scene__eclair" width="420" height="96" fill="#eaf4ff" />
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
            // Le parapluie, sous des gouttes qui s'y posent.
            <g class="rain-scene__gouttes" stroke="#7fd0f5" stroke-width="2" stroke-linecap="round">
                <line x1="262" y1="2" x2="261" y2="8" style="animation-delay: 0s" />
                <line x1="280" y1="2" x2="279" y2="8" style="animation-delay: -0.45s" />
                <line x1="298" y1="2" x2="297" y2="8" style="animation-delay: -0.9s" />
            </g>
            <g transform="translate(250 18)">
                <g class="rain-scene__parapluie">
                    <path d="M0 26 A30 26 0 0 1 60 26 Q52 20 45 26 Q37 20 30 26 Q22 20 15 26 Q8 20 0 26 Z" fill="#4aa3d8" />
                    <path d="M30 26 V58 a6 6 0 0 1 -12 0" fill="none" stroke="#e8edf2" stroke-width="2.6" stroke-linecap="round" />
                </g>
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
                <g class="uv-scene__rayons" stroke="#f7c948" stroke-width="3" stroke-linecap="round">
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

#[derive(Properties, PartialEq)]
pub struct IconeProps {
    /// La fonction illustrée : veille, rain, advice, uv, air, pollen, alerts — et,
    /// pour les cartes « Données », model, rules, hours.
    pub cle: AttrValue,
}

/// L'illustration d'une fonction, en tête de sa carte : un dessin par
/// question, aux couleurs de ce qu'elle annonce.
#[function_component]
pub fn IconeFonction(props: &IconeProps) -> Html {
    let dessin = match props.cle.as_str() {
        "rain" => html! {
            <>
                <path class="icone__nuage" d="M18 40h28a10 10 0 0 0 0-20 14 14 0 0 0-27-3A9 9 0 0 0 18 40Z" fill="#c6d6e4" />
                <g class="icone__gouttes" stroke="#7fd0f5" stroke-width="3.2" stroke-linecap="round">
                    <line x1="24" y1="46" x2="21" y2="54" />
                    <line x1="34" y1="46" x2="31" y2="54" />
                    <line x1="44" y1="46" x2="41" y2="54" />
                </g>
            </>
        },
        "advice" => html! {
            <g class="icone__parapluie">
                <path d="M8 32a24 22 0 0 1 48 0q-6-5-12 0-6-5-12 0-6-5-12 0-6-5-12 0Z" fill="#4aa3d8" />
                <path d="M32 10v2" stroke="#e8edf2" stroke-width="3" stroke-linecap="round" />
                <path d="M32 32v18a5 5 0 0 1-10 0" fill="none" stroke="#e8edf2" stroke-width="3.2" stroke-linecap="round" />
            </g>
        },
        "uv" => html! {
            <>
                <circle class="icone__coeur" cx="32" cy="32" r="11" fill="#f7c948" />
                <g class="icone__rayons" stroke="#f7c948" stroke-width="3.2" stroke-linecap="round">
                    <line x1="32" y1="6" x2="32" y2="13" />
                    <line x1="32" y1="51" x2="32" y2="58" />
                    <line x1="6" y1="32" x2="13" y2="32" />
                    <line x1="51" y1="32" x2="58" y2="32" />
                    <line x1="13.6" y1="13.6" x2="18.5" y2="18.5" />
                    <line x1="45.5" y1="45.5" x2="50.4" y2="50.4" />
                    <line x1="13.6" y1="50.4" x2="18.5" y2="45.5" />
                    <line x1="45.5" y1="18.5" x2="50.4" y2="13.6" />
                </g>
            </>
        },
        "air" => html! {
            <>
                <g class="icone__souffle" fill="none" stroke="#8fd3c8" stroke-width="3.2" stroke-linecap="round">
                    <path d="M8 24h30a7 7 0 1 0-7-7" pathLength="100" />
                    <path d="M8 34h40a7 7 0 1 1-7 7" pathLength="100" />
                    <path d="M8 44h18" pathLength="100" />
                </g>
                <g class="icone__poussieres" fill="#c6d6e4">
                    <circle cx="50" cy="20" r="2.4" />
                    <circle cx="56" cy="30" r="1.8" />
                    <circle cx="36" cy="50" r="2" />
                </g>
            </>
        },
        "pollen" => html! {
            <>
                <g class="icone__fleur">
                    <path d="M32 58V38" stroke="#7ed07a" stroke-width="3.2" stroke-linecap="round" />
                    <path d="M32 50c-8 0-12-4-13-10 7 0 12 3 13 10Z" fill="#7ed07a" />
                    <g fill="#f0c14b">
                        <circle cx="32" cy="14" r="7" />
                        <circle cx="43" cy="22" r="7" />
                        <circle cx="39" cy="34" r="7" />
                        <circle cx="25" cy="34" r="7" />
                        <circle cx="21" cy="22" r="7" />
                    </g>
                    <circle cx="32" cy="25" r="6" fill="#ef8a5a" />
                </g>
                <g class="icone__grains" fill="#f7d77a" opacity="0.8">
                    <circle cx="52" cy="10" r="1.6" />
                    <circle cx="56" cy="18" r="1.2" />
                    <circle cx="10" cy="12" r="1.4" />
                </g>
            </>
        },
        // Le guetteur : un radar qui balaie, et une averse repérée.
        "veille" => html! {
            <>
                <circle cx="32" cy="32" r="24" fill="none" stroke="#c6d6e4" stroke-width="2.4" opacity="0.5" />
                <circle cx="32" cy="32" r="15" fill="none" stroke="#7fd0f5" stroke-width="2.4" opacity="0.6" />
                <path class="feature__radar" d="M32 32 L32 8 A24 24 0 0 1 52.8 20 Z" fill="#7fd0f5" opacity="0.45" />
                <circle class="icone__echo" cx="44" cy="18" r="4" fill="#7fd0f5" />
                <circle cx="32" cy="32" r="3.4" fill="#e8edf2" />
            </>
        },
        // Les trois cartes « Données » : six modèles empilés, une règle
        // partagée, une horloge à l'heure de la ville.
        "model" => html! {
            <>
                <path class="icone__couche" d="M32 10 56 22 32 34 8 22Z" fill="#7fd0f5" />
                <path class="icone__couche" d="M8 32l24 12 24-12" fill="none" stroke="#c6d6e4" stroke-width="3.2" stroke-linejoin="round" stroke-linecap="round" />
                <path class="icone__couche" d="M8 42l24 12 24-12" fill="none" stroke="#9ed073" stroke-width="3.2" stroke-linejoin="round" stroke-linecap="round" />
            </>
        },
        "rules" => html! {
            <>
                <path d="M32 10v42M18 52h28" stroke="#c6d6e4" stroke-width="3.2" stroke-linecap="round" />
                <g class="icone__fleau">
                    <path d="M12 18h40" stroke="#c6d6e4" stroke-width="3.2" stroke-linecap="round" />
                    <path d="M12 18 5 34h14Z M52 18l-7 16h14Z" fill="none" stroke="#f0c14b" stroke-width="2.6" stroke-linejoin="round" />
                    <path d="M5 34a7 5 0 0 0 14 0Z M45 34a7 5 0 0 0 14 0Z" fill="#f0c14b" />
                </g>
                <circle cx="32" cy="10" r="3.4" fill="#9ed073" />
            </>
        },
        "hours" => html! {
            <>
                <circle cx="32" cy="32" r="23" fill="none" stroke="#c6d6e4" stroke-width="3.2" />
                <g stroke="#c6d6e4" stroke-width="2.6" stroke-linecap="round">
                    <line x1="32" y1="13" x2="32" y2="16" />
                    <line x1="51" y1="32" x2="48" y2="32" />
                    <line x1="32" y1="51" x2="32" y2="48" />
                    <line x1="13" y1="32" x2="16" y2="32" />
                </g>
                <line x1="32" y1="32" x2="32" y2="22" stroke="#e8edf2" stroke-width="3.4" stroke-linecap="round" />
                <line class="feature__aiguille" x1="32" y1="32" x2="44" y2="32" stroke="#9ed073" stroke-width="2.8" stroke-linecap="round" />
                <circle cx="32" cy="32" r="2.6" fill="#9ed073" />
            </>
        },
        _ => html! {
            <>
                <g class="icone__cloche">
                    <path d="M32 8a4 4 0 0 1 4 4v1.5A15 15 0 0 1 47 28v10l5 7H12l5-7V28a15 15 0 0 1 11-14.5V12a4 4 0 0 1 4-4Z" fill="#f0c14b" />
                    <path d="M26 49a6 6 0 0 0 12 0Z" fill="#f0c14b" />
                </g>
                <g class="icone__ondes" fill="none" stroke="#ef8a5a" stroke-width="3" stroke-linecap="round">
                    <path d="M8 22a26 26 0 0 1 6-10" />
                    <path d="M56 22a26 26 0 0 0-6-10" />
                </g>
            </>
        },
    };

    html! {
        <svg class="feature__icon" viewBox="0 0 64 64" aria-hidden="true">{ dessin }</svg>
    }
}
