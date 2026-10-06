//! Ce qu'on voit tomber : une ligne, sous la température, quand l'aéroport le
//! plus proche voit tomber quelque chose.
//!
//! Partagé par l'application et la vitrine ; miroir de la ligne de
//! `HeroView.swift`. Le ciel observé ne se montre que quand il mouille : un
//! aéroport qui ne voit rien tomber ne dit rien de plus que la prévision. La
//! mention de la source l'accompagne, comme pour toute source utilisée.

use klima_core::ciel::CielObserve;
use klima_core::i18n::params;
use yew::prelude::*;

use crate::dates;
use crate::i18n::use_i18n;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub ciel: Option<CielObserve>,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component]
pub fn CielVu(props: &Props) -> Html {
    let i18n = use_i18n();
    let Some((ciel, precipitation)) = props.ciel.as_ref().and_then(|c| Some((c, c.precipitation()?))) else {
        return Html::default();
    };
    let texte = i18n.with(
        "ciel.observed",
        &params([
            ("station", ciel.nom.as_str().into()),
            ("distance", (ciel.distance_km as i32).into()),
            ("time", dates::heure_minute(ciel.time, i18n.locale()).as_str().into()),
            ("kind", i18n.t(&precipitation.key()).as_str().into()),
        ]),
    );
    html! {
        <p class={classes!("vu", props.class.clone())} role="status">
            <span class="vu__goutte" aria-hidden="true" />
            <span class="vu__texte">{ texte }</span>
            <small class="vu__source">{ i18n.t("ciel.credit") }</small>
        </p>
    }
}
