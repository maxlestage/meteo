//! Le montage de l'application web, en Yew et WebAssembly.
//!
//! Miroir de `web/src/main.tsx`.
//!
//! ## L'acheminement des appels météo
//!
//! Sans relais configuré, chaque navigateur interroge les fournisseurs
//! lui-même : c'est le mode de développement, et il reste sur le plan gratuit
//! d'Open-Meteo, réservé à un usage non commercial. Avec un relais, les appels
//! passent par lui — clé commerciale, cache mutualisé, et MET Norway devient
//! accessible au web puisque c'est le serveur qui se nomme.
//!
//! Le réglage est lu à la compilation (`KLIMA_RELAY`), comme la variable
//! `VITE_KLIMA_RELAY` du côté TypeScript : un binaire WebAssembly n'a pas
//! d'environnement à l'exécution.

mod app;
mod composants;
mod crochets;
mod dates;
mod horloge;
mod i18n;
mod messages;
mod reseau;
mod storage;

use klima_core::endpoints::{Endpoints, relay_from};
use yew::prelude::*;

#[function_component]
fn Racine() -> Html {
    let endpoints = acheminement();

    html! {
        <i18n::I18nProvider>
            <app::App {endpoints} />
        </i18n::I18nProvider>
    }
}

fn acheminement() -> Endpoints {
    let origine = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();

    match relay_from(option_env!("KLIMA_RELAY"), &origine) {
        Some(relais) => Endpoints::relais(&relais),
        None => Endpoints::direct(),
    }
}

fn main() {
    yew::Renderer::<Racine>::new().render();
}
