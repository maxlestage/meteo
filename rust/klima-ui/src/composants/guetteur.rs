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

use klima_core::ciel::{Annonce, CielObserve};
use klima_core::format::Formats;
use klima_core::i18n::params;
use klima_core::radar::{Prevision, direction};
use klima_core::veille::{
    Immediat, Precipitation, Suite, Veille, aveugle, mouille, observer, prochaine_lecture, radariser, veille,
};
use yew::prelude::*;

use crate::crochets::veille::EtatVeille;
use crate::dates;
use crate::i18n::{I18n, use_i18n};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub etat: EtatVeille,
    /// Le ciel de l'aéroport le plus proche (`Forecast::ciel`) : ce qu'il
    /// voit tomber mouille les quarts que son bulletin couvre, et ce que ses
    /// prévisionnistes annoncent est dit.
    #[prop_or_default]
    pub ciel: Option<CielObserve>,
    /// Ce que voit le radar et ce qu'il prévoit (`EtatPrevision::radar`) :
    /// l'heure qui vient en est refaite.
    #[prop_or_default]
    pub radar: Option<Prevision>,
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
        .and_then(|(maintenant, q)| {
            // Le radar d'abord — il voit les averses que les modèles ratent.
            // Quand il voit la ville, il fait foi pour ce qui tombe ; sinon,
            // ce que voit l'aéroport.
            let decalage = q.utc_offset_seconds * 1000;
            let ciel = props.ciel.as_ref().filter(|_| props.radar.as_ref().is_none_or(|r| r.maintenant.is_none()));
            let radar: Vec<(i64, f64)> = props
                .radar
                .iter()
                .flat_map(|r| r.quarts.iter().map(|(t, d)| (t + decalage, *d)))
                .collect();
            let serie = radariser(&q.quarts, &radar);
            let lu = veille(&observer(&serie, maintenant, ciel), maintenant)?;
            Some((lu, aveugle(&serie, maintenant, ciel)))
        });

    let corps = match (&vu, etat.loading) {
        (Some((vu, aveugle)), _) => corps(&i18n, vu, props.ciel.as_ref(), *aveugle),
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

    // Le radar : d'où viennent les averses, et la mention de sa source.
    let radar = props.radar.as_ref().filter(|r| r.maintenant.is_some()).map(|r| {
        let decalage = etat.quarts.as_ref().map_or(0, |q| q.utc_offset_seconds * 1000);
        let locale = i18n.locale();
        let pluie = r.quarts.iter().any(|(_, d)| *d >= klima_core::radar::seuils::DEBIT_MOUILLE);
        let mouvement = r.deplacement.filter(|_| pluie).map(|(vitesse, cap)| {
            i18n.with(
                "veille.radar.move",
                &params([
                    ("dir", i18n.t(direction(cap)).as_str().into()),
                    ("speed", i18n.f().unit(vitesse, "km/h", 0).as_str().into()),
                ]),
            )
        });
        let source = i18n.with(
            "veille.radar",
            &params([("time", dates::heure_minute(r.image + decalage, locale).as_str().into())]),
        );
        (mouvement, source)
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
            if let Some((Some(mouvement), _)) = &radar {
                <p class="guetteur__mouvement">{ mouvement.clone() }</p>
            }
            if let Some(releve) = releve {
                <p class="guetteur__releve">{ releve }</p>
            }
            if let Some((_, source)) = &radar {
                <p class="guetteur__releve guetteur__radar">{ source.clone() }</p>
            }
        </section>
    }
}

/// Les deux bulles et la grille des huit quarts.
fn corps(i18n: &I18n, vu: &Veille, ciel: Option<&CielObserve>, aveugle: bool) -> Html {
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

    // Ce que les prévisionnistes de l'aéroport annoncent. Quand ils annoncent
    // que quelque chose tombera, le guetteur ne promet plus de sec : la
    // prévision au quart d'heure, qui ne l'a pas vu, ne vaut pas mieux qu'eux.
    let annonce = ciel.and_then(|c| c.annonce().map(|a| (c, a))).map(|(c, a): (&CielObserve, Annonce)| {
        i18n.with(
            if a.passagere { "veille.airport.tempo" } else { "veille.airport.becmg" },
            &params([
                ("station", c.nom.as_str().into()),
                ("kind", genre(&a.precipitation).as_str().into()),
                ("time", heure(a.jusqu_a).as_str().into()),
            ]),
        )
    });

    let fin = heure(vu.fin_fenetre);
    let suite = match &vu.suite {
        // Les modèles n'ont pas vu ce qui tombe : la fin qu'ils donneraient
        // n'en est pas une.
        Suite::Accalmie { .. } if aveugle => i18n.t("veille.next.unseen"),
        Suite::Sec if annonce.is_some() => String::new(),
        Suite::Accalmie { fin: arret, reprise: None } if annonce.is_some() => {
            i18n.with("veille.next.stop", &params([("time", heure(*arret).as_str().into())]))
        }
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
                    if let Some(annonce) = annonce {
                        { " " }<strong class="guetteur__annonce">{ annonce }</strong>
                    }
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
