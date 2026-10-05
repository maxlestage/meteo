//! Le guetteur : la demi-heure en cours et les deux heures à venir, dites
//! comme un message qu'on reçoit.
//!
//! Partagé par l'application et la vitrine. Il parle en deux bulles — la
//! demi-heure, puis les deux heures —, montre les huit quarts sur une grille
//! qui tient dans la largeur (rien ne défile de côté), et dit quand il a lu et
//! quand il relira. Miroir de `GuetteurCardView.swift`.
//!
//! Les heures sont celles de la ville ; les phrases viennent du catalogue
//! partagé, le domaine ne rendant que des états.

use klima_core::format::Formats;
use klima_core::i18n::params;
use klima_core::veille::{Immediat, Precipitation, Suite, Veille, mouille, prochaine_lecture, veille};
use yew::prelude::*;

use crate::crochets::veille::EtatVeille;
use crate::dates;
use crate::i18n::{I18n, use_i18n};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub etat: EtatVeille,
    /// Classes de l'hôte : `card` dans l'application, rien sur la vitrine.
    #[prop_or_default]
    pub class: Classes,
}

/// Le débit qui remplit une barre (mm/h) : au-delà, c'est de la forte pluie,
/// et la barre est pleine.
const DEBIT_PLEIN: f64 = 8.0;

#[function_component]
pub fn Guetteur(props: &Props) -> Html {
    let i18n = use_i18n();
    let etat = &props.etat;

    let vu = etat
        .maintenant_a_la_ville()
        .zip(etat.quarts.as_ref())
        .and_then(|(maintenant, q)| veille(&q.quarts, maintenant));

    let corps = match (&vu, etat.loading) {
        (Some(vu), _) => corps(&i18n, vu),
        (None, true) => html! { <p class="guetteur__bulle guetteur__bulle--attente">{ i18n.t("veille.loading") }</p> },
        (None, false) => html! { <p class="guetteur__bulle">{ i18n.t("veille.unavailable") }</p> },
    };

    // « Relu à 16:07 · prochaine lecture à 16:16 », à l'heure de la ville.
    let releve = etat.lu_a.zip(etat.quarts.as_ref()).map(|(lu_a, q)| {
        let decalage = q.utc_offset_seconds * 1000;
        let locale = i18n.locale();
        i18n.with(
            "veille.checked",
            &params([
                ("time", dates::heure_minute(lu_a + decalage, locale).as_str().into()),
                ("next", dates::heure_minute(prochaine_lecture(lu_a) + decalage, locale).as_str().into()),
            ]),
        )
    });

    html! {
        <section class={classes!("guetteur", props.class.clone())} aria-label={i18n.t("veille.title")}>
            <header class="guetteur__tete">
                <Avatar />
                <div>
                    <h2 class="guetteur__nom">{ i18n.t("veille.title") }</h2>
                    <p class="guetteur__devise">{ i18n.t("veille.subtitle") }</p>
                </div>
                <span class={classes!("guetteur__direct", etat.loading.then_some("is-reading"))} aria-hidden="true" />
            </header>
            { corps }
            if let Some(releve) = releve {
                <p class="guetteur__releve">{ releve }</p>
            }
        </section>
    }
}

/// Les deux bulles et la grille des huit quarts.
fn corps(i18n: &I18n, vu: &Veille) -> Html {
    let f = i18n.f();
    let locale = i18n.locale();
    let heure = |ms: i64| dates::heure_minute(ms, locale);
    let genre = |p: &Precipitation| i18n.t(&p.key());
    let quantite = |mm: f64| f.unit(mm, "mm", 1);

    let immediat = match &vu.immediat {
        Immediat::Sec => i18n.t("veille.now.dry"),
        Immediat::Commence { debut, precipitation } => i18n.with(
            "veille.now.starts",
            &params([("time", heure(*debut).as_str().into()), ("kind", genre(precipitation).as_str().into())]),
        ),
        Immediat::Continue { precipitation } => {
            i18n.with("veille.now.continues", &params([("kind", genre(precipitation).as_str().into())]))
        }
        Immediat::Cesse { fin } => {
            i18n.with("veille.now.stops", &params([("time", heure(*fin).as_str().into())]))
        }
    };

    let fin = heure(vu.fin_fenetre);
    let suite = match &vu.suite {
        Suite::Sec => i18n.with("veille.next.dry", &params([("end", fin.as_str().into())])),
        Suite::Episode { debut, fin: Some(arret), precipitation, cumul } => i18n.with(
            "veille.next.episode",
            &params([
                ("start", heure(*debut).as_str().into()),
                ("end", heure(*arret).as_str().into()),
                ("kind", genre(precipitation).as_str().into()),
                ("amount", quantite(*cumul).as_str().into()),
            ]),
        ),
        Suite::Episode { debut, fin: None, precipitation, .. } => i18n.with(
            "veille.next.episodeOpen",
            &params([
                ("start", heure(*debut).as_str().into()),
                ("end", fin.as_str().into()),
                ("kind", genre(precipitation).as_str().into()),
            ]),
        ),
        Suite::Persiste { precipitation, cumul } => i18n.with(
            "veille.next.persists",
            &params([
                ("end", fin.as_str().into()),
                ("kind", genre(precipitation).as_str().into()),
                ("amount", quantite(*cumul).as_str().into()),
            ]),
        ),
        Suite::Accalmie { fin: arret, reprise: None } => i18n.with(
            "veille.next.lull",
            &params([("time", heure(*arret).as_str().into()), ("end", fin.as_str().into())]),
        ),
        Suite::Accalmie { fin: arret, reprise: Some(reprise) } => i18n.with(
            "veille.next.lullReturn",
            &params([("time", heure(*arret).as_str().into()), ("back", heure(*reprise).as_str().into())]),
        ),
    };

    let rafales = vu.rafales.map(|(rafale, quand)| {
        i18n.with(
            "veille.gusts",
            &params([
                ("gusts", f.unit(rafale, "km/h", 0).as_str().into()),
                ("time", heure(quand).as_str().into()),
            ]),
        )
    });
    let dernier = &vu.quarts[vu.quarts.len() - 1];
    let temperature = i18n.with(
        "veille.temperature",
        &params([
            ("temp", f.temperature(dernier.temperature).as_str().into()),
            ("time", heure(dernier.time).as_str().into()),
            ("feels", f.temperature(dernier.apparent_temperature).as_str().into()),
        ]),
    );

    html! {
        <>
            <div class="guetteur__fil">
                <p class="guetteur__bulle">
                    <span class="guetteur__quand">{ i18n.t("veille.now") }</span>
                    { immediat }
                </p>
                <p class="guetteur__bulle">
                    <span class="guetteur__quand">{ i18n.t("veille.next") }</span>
                    { suite }
                    if let Some(rafales) = rafales {
                        { " " }{ rafales }
                    }
                    { " " }{ temperature }
                </p>
            </div>
            { grille(vu, f, locale, i18n) }
        </>
    }
}

/// Huit colonnes, une par quart : la barre est aussi haute que le débit, le
/// premier quart est marqué « maintenant ». Une heure sur deux est écrite,
/// pour que les étiquettes tiennent sur un téléphone.
fn grille(vu: &Veille, f: Formats, locale: &'static str, i18n: &I18n) -> Html {
    let colonnes: Html = vu
        .quarts
        .iter()
        .enumerate()
        .map(|(i, quart)| {
            let debit = quart.precipitation * 4.0;
            let hauteur = (debit / DEBIT_PLEIN * 100.0).clamp(4.0, 100.0);
            let etiquette = dates::heure_minute(quart.time, locale);
            html! {
                <li
                    key={quart.time}
                    class={classes!("guetteur__quart", (i == 0).then_some("is-now"))}
                    title={format!("{etiquette} · {}", f.unit(quart.precipitation, "mm", 1))}
                >
                    <span class="guetteur__colonne">
                        <span
                            class={classes!("guetteur__barre", mouille(quart).then_some("is-wet"))}
                            style={format!("height: {hauteur:.0}%")}
                        />
                    </span>
                    <span class="guetteur__heure">
                        { if i % 2 == 0 { etiquette } else { String::new() } }
                    </span>
                </li>
            }
        })
        .collect();

    html! {
        <ol class="guetteur__grille" aria-label={i18n.t("veille.chart")}>{ colonnes }</ol>
    }
}

/// Le guetteur lui-même : un œil rond qui balaie le ciel, comme un radar.
#[function_component]
fn Avatar() -> Html {
    html! {
        <svg class="guetteur__avatar" viewBox="0 0 48 48" aria-hidden="true">
            <circle cx="24" cy="24" r="22" fill="#2b3d52" stroke="rgba(255,255,255,0.25)" />
            <circle cx="24" cy="24" r="14" fill="none" stroke="rgba(126,208,245,0.35)" />
            <circle cx="24" cy="24" r="7" fill="none" stroke="rgba(126,208,245,0.5)" />
            <path class="guetteur__balayage" d="M24 24 L24 2 A22 22 0 0 1 43 13 Z" fill="rgba(126,208,245,0.35)" />
            <circle cx="31" cy="15" r="2.4" fill="#7fd0f5" />
            <circle cx="24" cy="24" r="2.6" fill="#e8edf2" />
        </svg>
    }
}
