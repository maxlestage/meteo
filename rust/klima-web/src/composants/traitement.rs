//! La carte des conditions de pulvérisation.
//!
//! Miroir de `web/src/components/SprayCard.tsx` : la prochaine fenêtre en
//! clair, puis une frise des vingt-quatre prochaines heures.

use klima_core::agro::{HourlySample, SprayOpportunity, SprayVerdict, spray_windows};
use klima_core::format::describe_blocker;
use klima_core::i18n::params;
use yew::prelude::*;

use klima_ui::dates;
use klima_ui::i18n::use_i18n;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub hours: Vec<HourlySample>,
    pub next_spray: Option<SprayOpportunity>,
}

#[function_component]
pub fn Traitement(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let traducteur = i18n.translator();

    let fenetres: Vec<_> = spray_windows(&props.hours).into_iter().take(24).collect();

    // Le premier motif de blocage des 24 h résume la situation.
    let blocage = fenetres
        .iter()
        .find(|w| w.verdict == SprayVerdict::Defavorable && !w.blockers.is_empty())
        .map(|w| describe_blocker(&w.blockers[0], &traducteur, f));

    let frise: Html = fenetres
        .iter()
        .map(|window| {
            let motifs: Vec<String> = window
                .blockers
                .iter()
                .map(|blocker| describe_blocker(blocker, &traducteur, f))
                .collect();
            let infobulle = format!(
                "{} — {}/100{}",
                dates::heure(window.time, i18n.locale()),
                window.score,
                if motifs.is_empty() {
                    String::new()
                } else {
                    format!(" · {}", motifs.join(" · "))
                }
            );

            html! {
                <span
                    key={window.time}
                    class={format!("spray__hour spray__hour--{}", window.verdict.code())}
                    title={infobulle}
                />
            }
        })
        .collect();

    html! {
        <section class="card" aria-label={i18n.t("spray.title")}>
            <h2 class="card__label">{ i18n.t("spray.title") }</h2>

            <p class="spray__headline">
                if let Some(spray) = &props.next_spray {
                    { format!(
                        "{} → {}",
                        dates::capitale(&dates::jour_et_heure(spray.start, i18n.locale())),
                        dates::heure(spray.end, i18n.locale())
                    ) }
                } else {
                    { i18n.t("spray.none") }
                }
            </p>

            <p class="spray__caption">
                if let Some(spray) = &props.next_spray {
                    { i18n.with("spray.score", &params([("score", spray.score.into())])) }
                } else if let Some(motif) = &blocage {
                    { i18n.with(
                        "spray.mainBlocker",
                        &params([("blocker", motif.to_lowercase().as_str().into())]),
                    ) }
                } else {
                    { i18n.t("spray.unsuitable") }
                }
            </p>

            <div class="spray__strip">{ frise }</div>
            <div class="spray__scale">
                <span>{ i18n.t("spray.now") }</span>
                <span>{ i18n.t("spray.plus12") }</span>
                <span>{ i18n.t("spray.plus24") }</span>
            </div>
        </section>
    }
}
