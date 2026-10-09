//! Le ciel derrière la page : une journée qui passe à mesure qu'on lit.
//!
//! En haut, le soleil et quelques nuages qui traversent ; en descendant, le
//! soleil se couche, le ciel rougit puis s'assombrit, les étoiles s'allument ;
//! au pied de la page, la ville éclaire ses fenêtres.
//!
//! Tout est fait pour ne pas gêner la lecture : des formes molles et pâles,
//! jamais un contour net sous le texte — la ligne d'immeubles fixe essayée
//! avant transparaissait sous les cartes comme une tache. La ville est donc au
//! pied de la page, dans le flux, là où rien ne se lit par-dessus.
//!
//! Le défilement ne fait que poser deux variables CSS sur le calque du ciel,
//! une fois par image au plus : le reste de la page n'est pas recalculé. Pour
//! qui demande moins de mouvement, le ciel reste celui du jour, immobile.

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{AddEventListenerOptions, HtmlElement};
use yew::prelude::*;

/// Une suite pseudo-aléatoire fixe : le ciel est le même à chaque visite, et
/// d'un rendu à l'autre.
struct Graine(u32);

impl Graine {
    fn suivant(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(self.0 >> 8) / f64::from(1u32 << 24)
    }
}

/// Avancée dans la page, de 0 (en haut) à 1 (en bas).
///
/// Le film de grains ne compte pas : la page s'y fige sur un ciel de nuit, et
/// la journée reprend où elle en était quand il s'achève. Sans cela, sa
/// longue course ferait tomber la nuit avant la suite de la page.
fn avancee() -> Option<f64> {
    let fenetre = web_sys::window()?;
    let document = fenetre.document()?;
    let racine = document.document_element()?;
    let ecran = fenetre.inner_height().ok()?.as_f64()?;
    let mut hauteur = f64::from(racine.scroll_height()) - ecran;
    let mut y = fenetre.scroll_y().ok()?;
    if let Some(film) = document.query_selector(".film").ok().flatten() {
        let cadre = film.get_bounding_client_rect();
        let course = (cadre.height() - ecran).max(0.0);
        let debut = cadre.top() + y;
        y -= (y - debut).clamp(0.0, course);
        hauteur -= course;
    }
    Some(if hauteur > 1.0 { (y / hauteur).clamp(0.0, 1.0) } else { 0.0 })
}

fn moins_de_mouvement() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten())
        .map(|m| m.matches())
        .unwrap_or(false)
}

/// Pose l'heure du ciel : `--jour` va du plein jour (0) à la nuit (1), et
/// `--crepuscule` culmine à mi-page, quand le soleil touche l'horizon.
fn poser(ciel: &HtmlElement, jour: f64) {
    let crepuscule = (1.0 - (2.0 * jour - 1.0).abs()).max(0.0);
    let style = ciel.style();
    let _ = style.set_property("--jour", &format!("{jour:.3}"));
    let _ = style.set_property("--crepuscule", &format!("{crepuscule:.3}"));
}

#[function_component]
pub fn Ciel() -> Html {
    let node = use_node_ref();

    {
        let node = node.clone();
        use_effect_with((), move |()| {
            let fenetre = web_sys::window();
            let element = node.cast::<HtmlElement>();
            let mut nettoyage: Option<Box<dyn FnOnce()>> = None;

            if let (Some(fenetre), Some(ciel)) = (fenetre, element) {
                if !moins_de_mouvement() {
                    poser(&ciel, avancee().unwrap_or(0.0));

                    // Une seule mise à jour par image : le défilement envoie
                    // bien plus d'évènements que l'écran n'affiche d'images.
                    let en_attente = std::rc::Rc::new(std::cell::Cell::new(false));
                    let dessiner = {
                        let ciel = ciel.clone();
                        let en_attente = en_attente.clone();
                        Closure::<dyn Fn()>::new(move || {
                            en_attente.set(false);
                            poser(&ciel, avancee().unwrap_or(0.0));
                        })
                    };
                    let au_defilement = {
                        let fenetre = fenetre.clone();
                        let dessiner = dessiner.as_ref().unchecked_ref::<js_sys::Function>().clone();
                        Closure::<dyn Fn()>::new(move || {
                            if !en_attente.replace(true) {
                                let _ = fenetre.request_animation_frame(&dessiner);
                            }
                        })
                    };

                    let options = AddEventListenerOptions::new();
                    options.set_passive(true);
                    let rappel = au_defilement.as_ref().unchecked_ref::<js_sys::Function>().clone();
                    for evenement in ["scroll", "resize"] {
                        let _ = fenetre.add_event_listener_with_callback_and_add_event_listener_options(
                            evenement, &rappel, &options,
                        );
                    }

                    nettoyage = Some(Box::new(move || {
                        for evenement in ["scroll", "resize"] {
                            let _ = fenetre.remove_event_listener_with_callback(evenement, &rappel);
                        }
                        drop(au_defilement);
                        drop(dessiner);
                    }));
                }
            }

            move || {
                if let Some(nettoyer) = nettoyage {
                    nettoyer();
                }
            }
        });
    }

    // Les étoiles : placées une fois pour toutes, dans les deux tiers hauts
    // du ciel. Une sur trois scintille, à son rythme.
    let mut graine = Graine(20_260_512);
    let etoiles: Html = (0..44)
        .map(|i| {
            let x = graine.suivant() * 100.0;
            let y = graine.suivant() * 68.0;
            let r = 0.4 + graine.suivant() * 0.9;
            let delai = graine.suivant() * 6.0;
            html! {
                <circle
                    cx={format!("{x:.2}%")} cy={format!("{y:.2}%")} r={format!("{r:.2}")}
                    class={classes!((i % 3 == 0).then_some("ciel__scintille"))}
                    style={format!("animation-delay: -{delai:.2}s")}
                />
            }
        })
        .collect();

    // Quatre nuages, à des hauteurs et des vitesses différentes ; partis
    // d'avance, pour que l'écran n'arrive jamais sur un ciel vide.
    const NUAGES: [(f64, f64, f64, f64); 4] = [
        // (hauteur en % de l'écran, largeur en vw, durée en s, avance en s)
        (8.0, 52.0, 140.0, 30.0),
        (30.0, 38.0, 110.0, 80.0),
        (54.0, 60.0, 180.0, 140.0),
        (72.0, 34.0, 95.0, 20.0),
    ];
    let nuages: Html = NUAGES
        .iter()
        .map(|&(haut, largeur, duree, avance)| {
            html! {
                <svg
                    class="ciel__nuage"
                    viewBox="0 0 200 80"
                    style={format!(
                        "top: {haut}%; width: max({largeur}vw, 280px); \
                         animation-duration: {duree}s; animation-delay: -{avance}s"
                    )}
                >
                    <ellipse cx="100" cy="50" rx="92" ry="26" fill="url(#cielNuage)" />
                    <ellipse cx="74" cy="36" rx="44" ry="28" fill="url(#cielNuage)" />
                    <ellipse cx="124" cy="30" rx="52" ry="30" fill="url(#cielNuage)" />
                </svg>
            }
        })
        .collect();

    html! {
        <div class="ciel" ref={node} aria-hidden="true">
            <div class="ciel__nuit"></div>
            <div class="ciel__crepuscule"></div>
            <div class="ciel__soleil"></div>
            <svg class="ciel__etoiles">{ etoiles }</svg>
            <div class="ciel__filante"></div>
            <svg class="ciel__lune" viewBox="0 0 40 40">
                <path d="M28 30a14 14 0 1 1-6-26 11 11 0 0 0 6 26Z" fill="#e8edf2" />
            </svg>
            // Le dégradé des nuages, défini une fois et partagé.
            <svg class="ciel__defs" width="0" height="0">
                <defs>
                    <radialGradient id="cielNuage">
                        <stop offset="0%" stop-color="#ffffff" stop-opacity="1" />
                        <stop offset="60%" stop-color="#ffffff" stop-opacity="0.55" />
                        <stop offset="100%" stop-color="#ffffff" stop-opacity="0" />
                    </radialGradient>
                </defs>
            </svg>
            { nuages }
        </div>
    }
}

/// La ville au pied de la page : la nuit est tombée quand on y arrive, et les
/// fenêtres s'allument une à une sous les yeux du lecteur. Pleine largeur,
/// dans le flux : rien ne se lit par-dessus.
#[function_component]
pub fn Horizon() -> Html {
    // Une ligne d'immeubles sur 1200 unités, tirée une fois pour toutes.
    let mut graine = Graine(48_856);
    let mut immeubles = Vec::new();
    let mut x = 0.0;
    while x < 1200.0 {
        let largeur = 26.0 + (graine.suivant() * 34.0).round();
        let hauteur = 34.0 + (graine.suivant() * 78.0).round();
        immeubles.push((x, largeur, hauteur));
        x += largeur + 2.0;
    }

    let blocs: Html = immeubles
        .iter()
        .map(|&(x, largeur, hauteur)| {
            html! {
                <rect
                    x={format!("{x}")} y={format!("{}", 140.0 - hauteur)}
                    width={format!("{largeur}")} height={format!("{hauteur}")} rx="2"
                />
            }
        })
        .collect();

    let mut fenetres = Vec::new();
    for (i, &(x, largeur, hauteur)) in immeubles.iter().enumerate() {
        let colonnes = ((largeur - 8.0) / 8.0).floor().max(1.0) as usize;
        let rangees = ((hauteur - 10.0) / 10.0).floor().max(1.0) as usize;
        for k in 0..colonnes * rangees {
            // Une fenêtre sur trois est allumée ; une sur quatre de celles-là
            // s'éteint de temps en temps.
            if graine.suivant() > 0.34 {
                continue;
            }
            let fx = x + 5.0 + (k % colonnes) as f64 * 8.0;
            let fy = 140.0 - hauteur + 6.0 + (k / colonnes) as f64 * 10.0;
            let vacille = (k + i) % 4 == 0;
            let delai = graine.suivant() * 9.0;
            // Quand elle s'allume, une fois la ville à l'écran : au hasard,
            // dans les trois secondes, les plus à gauche un peu plus tôt.
            let allumage = graine.suivant() * 2.2 + fx / 1200.0 * 0.8;
            fenetres.push(html! {
                <rect
                    x={format!("{fx}")} y={format!("{fy}")} width="3.5" height="4.5"
                    class={classes!(vacille.then_some("horizon__vacille"))}
                    style={format!("--allume: {allumage:.2}s; --vacille: -{delai:.2}s")}
                />
            });
        }
    }

    // Les plus hauts portent un feu rouge, qui clignote.
    let mut hauts: Vec<_> = immeubles.iter().collect();
    hauts.sort_by(|a, b| b.2.total_cmp(&a.2));
    let feux: Html = hauts
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, &&(x, largeur, hauteur))| {
            html! {
                <circle
                    cx={format!("{}", x + largeur / 2.0)} cy={format!("{}", 136.0 - hauteur)} r="2"
                    style={format!("animation-delay: -{}s", i as f64 * 0.7)}
                />
            }
        })
        .collect();

    html! {
        <div class="horizon" aria-hidden="true" data-apparait="ville">
            <svg viewBox="0 0 1200 140" preserveAspectRatio="xMidYMax slice">
                <defs>
                    <linearGradient id="horizonFacade" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stop-color="#1b2738" />
                        <stop offset="100%" stop-color="#0b111a" />
                    </linearGradient>
                </defs>
                <g fill="url(#horizonFacade)">{ blocs }</g>
                <g class="horizon__fenetres" fill="#f7d77a">{ fenetres }</g>
                <g class="horizon__feux" fill="#ef5a4f">{ feux }</g>
            </svg>
        </div>
    }
}
