//! La liste des jours, avec la barre d'amplitude thermique de la semaine.
//!
//! Toutes les barres se lisent sur la même échelle — celle de la semaine —
//! sinon une journée douce et une journée froide auraient la même barre à deux
//! endroits différents, et la comparaison d'un coup d'œil serait fausse.

use klima_core::agro::DailySample;
use klima_core::weather::weather_condition;
use yew::prelude::*;

use klima_ui::composants::pictogramme::Pictogramme;
use klima_ui::dates;
use klima_ui::i18n::use_i18n;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub days: Vec<DailySample>,
    /// Température du moment, repérée sur la barre d'aujourd'hui.
    pub current_temperature: f64,
}

#[function_component]
pub fn Jours(props: &Props) -> Html {
    let i18n = use_i18n();
    let f = i18n.f();

    let semaine_basse = props
        .days
        .iter()
        .map(|d| d.temperature_min)
        .fold(f64::INFINITY, f64::min);
    let semaine_haute = props
        .days
        .iter()
        .map(|d| d.temperature_max)
        .fold(f64::NEG_INFINITY, f64::max);
    let amplitude = (semaine_haute - semaine_basse).max(1.0);

    let lignes: Html = props
        .days
        .iter()
        .enumerate()
        .map(|(index, day)| {
            let condition = weather_condition(day.weather_code);
            let gauche = (day.temperature_min - semaine_basse) / amplitude * 100.0;
            let largeur =
                ((day.temperature_max - day.temperature_min) / amplitude * 100.0).max(6.0);

            html! {
                <li class="days__row" key={day.date}>
                    <span class="days__name">
                        if index == 0 {
                            { i18n.t("daily.today") }
                        } else {
                            { dates::capitale(&dates::jour_court(day.date, i18n.locale())) }
                        }
                    </span>

                    <span class="days__weather">
                        <Pictogramme
                            icon={condition.icon}
                            size={24}
                            title={i18n.t(condition.label_key)}
                        />
                        <span class="days__rain">
                            if day.precipitation_probability_max >= 10.0 {
                                { f.percent(day.precipitation_probability_max) }
                            }
                        </span>
                    </span>

                    <span class="days__low">{ f.temperature(day.temperature_min) }</span>
                    <span class="days__bar">
                        <span
                            class="days__fill"
                            style={format!("left: {gauche}%; width: {largeur}%")}
                        />
                        if index == 0 {
                            <span
                                class="days__now"
                                style={format!(
                                    "left: {}%",
                                    ((props.current_temperature - semaine_basse) / amplitude
                                        * 100.0)
                                        .clamp(0.0, 100.0)
                                )}
                            />
                        }
                    </span>
                    <span class="days__high">{ f.temperature(day.temperature_max) }</span>
                </li>
            }
        })
        .collect();

    html! {
        <section class="card" aria-label={i18n.t("daily.title")}>
            <h2 class="card__label">{ i18n.t("daily.title") }</h2>
            <ul class="days">{ lignes }</ul>
        </section>
    }
}
