//! Le pictogramme météo.
//!
//! Miroir de `core/src/ui/WeatherIcon.tsx`. Peu de formes, un trait large,
//! dans l'esprit des symboles du système : le même dessin tient dans une
//! colonne de bandeau horaire et dans une ligne de prévision.

use klima_core::weather::ConditionIcon;
use yew::prelude::*;

const CLOUD: &str = "M7.1 18.4h9.8a4.1 4.1 0 0 0 .4-8.2 5.7 5.7 0 0 0-10.9-1.2 4.5 4.5 0 0 0 .7 9.4Z";
const MOON: &str = "M17.6 14.9A6.6 6.6 0 0 1 9.1 6.4a6.6 6.6 0 1 0 8.5 8.5Z";

#[derive(Properties, PartialEq)]
pub struct Props {
    pub icon: ConditionIcon,
    /// Décline le pictogramme en version nuit (lune au lieu du soleil).
    #[prop_or(true)]
    pub is_day: bool,
    #[prop_or(28)]
    pub size: u32,
    #[prop_or_default]
    pub title: Option<AttrValue>,
}

#[function_component]
pub fn Pictogramme(props: &Props) -> Html {
    let size = props.size.to_string();
    let (role, hidden) = match &props.title {
        Some(_) => ("img", None),
        None => ("presentation", Some(AttrValue::from("true"))),
    };

    html! {
        <svg
            class="wicon"
            viewBox="0 0 24 24"
            width={size.clone()}
            height={size}
            role={role}
            aria-label={props.title.clone()}
            aria-hidden={hidden}
        >
            { formes(props.icon, props.is_day) }
        </svg>
    }
}

fn formes(icon: ConditionIcon, is_day: bool) -> Html {
    let nuage = html! { <path d={CLOUD} class="wicon__cloud" /> };

    match icon {
        ConditionIcon::Clear => {
            if is_day {
                soleil(12.0, 12.0, 4.2)
            } else {
                html! { <path d={MOON} class="wicon__moon" /> }
            }
        }
        ConditionIcon::Partly => html! {
            <>
                if is_day {
                    { soleil(8.5, 8.0, 3.2) }
                } else {
                    <path
                        d={MOON}
                        class="wicon__moon"
                        transform="translate(-3.5 -3.5) scale(0.72) translate(6 6)"
                    />
                }
                { nuage }
            </>
        },
        ConditionIcon::Cloudy => nuage,
        ConditionIcon::Fog => html! {
            <>
                { nuage }
                { trait_(5.5, 20.6, 18.5, 20.6, "wicon__fog") }
                { trait_(8.0, 22.8, 16.0, 22.8, "wicon__fog") }
            </>
        },
        ConditionIcon::Drizzle => html! {
            <>{ nuage }{ goutte(9.5) }{ goutte(14.5) }</>
        },
        ConditionIcon::Rain => html! {
            <>{ nuage }{ goutte(7.5) }{ goutte(12.0) }{ goutte(16.5) }</>
        },
        ConditionIcon::Showers => html! {
            <>
                if is_day { { soleil(8.5, 8.0, 3.2) } }
                { nuage }
                { goutte(9.5) }
                { goutte(15.0) }
            </>
        },
        ConditionIcon::Snow => html! {
            <>{ nuage }{ flocon(9.0) }{ flocon(15.0) }</>
        },
        ConditionIcon::Thunder => html! {
            <>
                { nuage }
                <path d="M12.9 19.2h2.6l-4.4 4.6 1-3.2H9.6l3.9-4.3Z" class="wicon__bolt" />
            </>
        },
    }
}

fn soleil(cx: f64, cy: f64, r: f64) -> Html {
    let rayons: Html = (0..8)
        .map(|i| {
            let angle = f64::from(i) * std::f64::consts::FRAC_PI_4;
            let interieur = r + 1.4;
            let exterieur = r + 3.2;
            html! {
                <line
                    x1={fmt(cx + angle.cos() * interieur)}
                    y1={fmt(cy + angle.sin() * interieur)}
                    x2={fmt(cx + angle.cos() * exterieur)}
                    y2={fmt(cy + angle.sin() * exterieur)}
                    class="wicon__ray"
                />
            }
        })
        .collect();

    html! {
        <>
            <circle cx={fmt(cx)} cy={fmt(cy)} r={fmt(r)} class="wicon__sun" />
            { rayons }
        </>
    }
}

fn goutte(x: f64) -> Html {
    html! { <line x1={fmt(x)} y1="20" x2={fmt(x - 1.1)} y2="23.2" class="wicon__drop" /> }
}

fn flocon(x: f64) -> Html {
    html! { <circle cx={fmt(x)} cy="21.6" r="1.15" class="wicon__flake" /> }
}

fn trait_(x1: f64, y1: f64, x2: f64, y2: f64, class: &'static str) -> Html {
    html! { <line x1={fmt(x1)} y1={fmt(y1)} x2={fmt(x2)} y2={fmt(y2)} {class} /> }
}

/// Un nombre dans un attribut SVG : trois décimales suffisent, et les zéros
/// inutiles alourdissent le document.
fn fmt(value: f64) -> String {
    let arrondi = (value * 1000.0).round() / 1000.0;
    format!("{arrondi}")
}
