//! La marque de Klima.
//!
//! Un K dont la hampe porte une goutte et dont les bras sont taillés en lames
//! de feuille : la lettre du nom, la pluie et le vivant dans un seul signe.
//! La géométrie est celle des gabarits de `site/public` — favicon, icônes web,
//! icône iOS et watchOS, aperçus de lien partagé et ce composant sortent du
//! même dessin, dans le même repère de 1024.

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MarqueProps {
    #[prop_or(32)]
    pub size: u32,
    #[prop_or_default]
    pub title: Option<AttrValue>,
}

#[function_component]
pub fn Marque(props: &MarqueProps) -> Html {
    let size = props.size.to_string();
    let (role, hidden) = match &props.title {
        Some(_) => ("img", None),
        None => ("presentation", Some(AttrValue::from("true"))),
    };

    html! {
        <svg
            viewBox="0 0 1024 1024"
            width={size.clone()}
            height={size}
            role={role}
            aria-label={props.title.clone()}
            aria-hidden={hidden}
            class="brand-mark"
        >
            <defs>
                <linearGradient id="klimaTile" x1="0" y1="0" x2="0.4" y2="1">
                    <stop offset="0%" stop-color="#4e7d33" />
                    <stop offset="100%" stop-color="#22401a" />
                </linearGradient>
            </defs>

            // Le carreau : la parcelle sur laquelle la lettre est posée.
            <rect width="1024" height="1024" rx="229" fill="url(#klimaTile)" />

            // La hampe.
            <rect x="322" y="276" width="96" height="560" rx="48" fill="#f2f6ec" />

            // Les deux bras, taillés en lames : celui du haut prend la lumière.
            <path d="M418 556c118-36 214-124 286-244 22 148-70 268-206 316Z" fill="#f2f6ec" />
            <path d="M418 556c118 36 214 124 286 244 22-148-70-268-206-316Z" fill="#b9d98f" />

            // La goutte, posée sur la hampe.
            <circle cx="370" cy="240" r="52" fill="#7fd0f5" />
        </svg>
    }
}

#[derive(Properties, PartialEq)]
pub struct LockupProps {
    #[prop_or(30)]
    pub size: u32,
}

/// Marque et nom, côte à côte.
#[function_component]
pub fn MarqueEtNom(props: &LockupProps) -> Html {
    html! {
        <span class="brand">
            <Marque size={props.size} />
            <span class="brand__name">{ "Klima" }</span>
        </span>
    }
}
