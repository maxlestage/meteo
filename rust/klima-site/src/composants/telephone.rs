//! L'aperçu de l'application iOS, alimenté par la commune choisie plus haut :
//! ce que le visiteur voit ici, il le retrouve sur son téléphone.
//!

use klima_api::today::DayDigest;
use klima_core::meteo::CurrentSample;
use klima_core::ville::Pluie;
use klima_core::calendar::hour_of;
use klima_core::position::Parcelle;
use klima_core::weather::weather_condition;
use klima_ui::composants::pictogramme::Pictogramme;
use klima_ui::i18n::use_i18n;
use yew::prelude::*;

use crate::composants::compte::{Compte, Forme};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub parcelle: Parcelle,
    pub digest: Option<DayDigest>,
    pub current: Option<CurrentSample>,
}

#[function_component]
pub fn Telephone(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();
    let condition = props.current.as_ref().map(|c| weather_condition(c.weather_code));

    let heures: Html = props
        .digest
        .as_ref()
        .map(|digest| {
            digest
                .remaining_hours
                .iter()
                .take(5)
                .enumerate()
                .map(|(i, hour)| {
                    html! {
                        <div class="phone__hour" key={hour.time} style={format!("--i: {i}")}>
                            <span>{ hour_of(hour.time) }</span>
                            <Pictogramme
                                icon={weather_condition(hour.weather_code).icon}
                                is_day={hour.is_day}
                                size={18}
                            />
                            <span>{ f.temperature(hour.temperature) }</span>
                        </div>
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    html! {
        <div class="phone" aria-hidden="true" data-incline="" data-lueur="">
            <div class="phone__screen">
                <div class="phone__status">
                    <span>{ "9:41" }</span>
                    <span class="phone__signal" />
                </div>

                <p class="phone__place">{ &props.parcelle.name }</p>
                <p class="phone__temperature">
                    { match &props.current {
                        Some(current) => html! {
                            <Compte valeur={current.temperature} forme={Forme::Temperature} />
                        },
                        None => html! { "—" },
                    } }
                </p>
                <p class="phone__condition">
                    { match &condition {
                        Some(condition) => i18n.t(condition.label_key),
                        None => i18n.t("phone.loading"),
                    } }
                </p>
                <p class="phone__range">
                    { match &props.digest {
                        Some(digest) => format!(
                            "↑ {}   ↓ {}",
                            f.temperature(digest.temperature_max),
                            f.temperature(digest.temperature_min)
                        ),
                        None => String::new(),
                    } }
                </p>

                <div class="phone__card">
                    <p class="phone__label">{ i18n.t("phone.conditions") }</p>
                    <div class="phone__hours">{ heures }</div>
                </div>

                <div class="phone__tiles">
                    <div class="phone__tile">
                        <p class="phone__label">{ i18n.t("phone.rain") }</p>
                        <p class="phone__value">
                            { match props.digest.as_ref().map(|d| &d.pluie) {
                                Some(Pluie::Aucune { .. }) => i18n.t("phone.rain.none"),
                                Some(Pluie::EnCours { .. }) => i18n.t("phone.rain.now"),
                                Some(Pluie::Prevue { debut, .. }) => {
                                    klima_ui::dates::heure(*debut, i18n.locale())
                                }
                                None => "—".to_owned(),
                            } }
                        </p>
                    </div>
                    <div class="phone__tile">
                        <p class="phone__label">{ i18n.t("phone.feels") }</p>
                        <p class="phone__value">
                            { match &props.current {
                                Some(current) => html! {
                                    <Compte valeur={current.apparent_temperature} forme={Forme::Temperature} />
                                },
                                None => html! { "—" },
                            } }
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}
