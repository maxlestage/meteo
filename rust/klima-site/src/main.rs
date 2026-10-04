//! Le montage de la vitrine, en Yew et WebAssembly.
//!
//! L'acheminement des appels météo suit la même règle que l'application :
//! sans relais configuré, le navigateur interroge les fournisseurs lui-même ;
//! avec un relais, tout passe par lui. Le réglage est lu à la compilation
//! (`KLIMA_RELAY`) : un binaire WebAssembly n'a pas d'environnement à
//! l'exécution.

mod app;
mod composants;
mod crochets;
mod messages;

use klima_core::endpoints::{Endpoints, relay_from};
use klima_core::i18n::MessageSet;
use klima_ui::i18n::{Catalogues, I18nProvider};
use yew::prelude::*;

fn catalogues() -> Catalogues {
    static CATALOGUES: std::sync::LazyLock<[&'static MessageSet; 1]> =
        std::sync::LazyLock::new(|| [&messages::SITE_MESSAGES]);
    Catalogues(&*CATALOGUES)
}

#[function_component]
fn Racine() -> Html {
    let endpoints = acheminement();

    html! {
        <I18nProvider catalogues={catalogues()}>
            <app::App {endpoints} />
        </I18nProvider>
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
