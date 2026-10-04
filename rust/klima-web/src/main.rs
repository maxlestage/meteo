//! Le montage de l'application web, en Yew et WebAssembly.
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
mod messages;

use klima_core::endpoints::{Endpoints, relay_from};
use klima_core::i18n::MessageSet;
use klima_ui::i18n::{Catalogues, I18nProvider};
use yew::prelude::*;

/// Les catalogues de l'application, dans l'ordre : le sien, puis le partagé,
/// que le fournisseur ajoute derrière.
fn catalogues() -> Catalogues {
    static CATALOGUES: std::sync::LazyLock<[&'static MessageSet; 1]> =
        std::sync::LazyLock::new(|| [&messages::WEB_MESSAGES]);
    Catalogues(&*CATALOGUES)
}

#[function_component]
fn Racine() -> Html {
    let relais = relais();
    let endpoints = match &relais {
        Some(relais) => Endpoints::relais(relais),
        None => Endpoints::direct(),
    };

    html! {
        <I18nProvider catalogues={catalogues()}>
            <app::App {endpoints} {relais} />
        </I18nProvider>
    }
}

/// L'adresse du relais, s'il y en a un.
///
/// Elle sert deux fois : à fabriquer les quatre adresses des fournisseurs, et
/// à demander le palier accordé — une question qui ne passe par aucun
/// fournisseur, et qui n'a donc pas sa place dans `Endpoints`.
fn relais() -> Option<String> {
    let origine = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();

    relay_from(option_env!("KLIMA_RELAY"), &origine)
}

fn main() {
    yew::Renderer::<Racine>::new().render();
}
