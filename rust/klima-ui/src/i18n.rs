//! La langue de l'interface, portée par un contexte.
//!
//! Miroir de `core/src/ui/i18n.tsx`.
//!
//! Celle choisie à la visite précédente, sinon celle du navigateur, sinon la
//! langue de référence. Le choix est écrit dans le stockage local — et son
//! échec est sans conséquence : en navigation privée, la langue tient le temps
//! de la visite.

use klima_core::format::Formats;
use klima_core::i18n::{
    Language, MessageSet, Params, REFERENCE_LANGUAGE, Translator, detect_language,
};
use klima_core::messages::SHARED_MESSAGES;
use yew::prelude::*;

use crate::storage;

const STORAGE_KEY: &str = "klima.language";

/// Les catalogues d'une interface, dans l'ordre de consultation.
///
/// Comparés par adresse et non par contenu : ce sont des tables immuables
/// montées une fois, et relire quatre-vingt-cinq libellés à chaque rendu pour
/// conclure qu'ils n'ont pas changé serait du travail pour rien.
#[derive(Clone, Copy, Debug)]
pub struct Catalogues(pub &'static [&'static MessageSet]);

impl PartialEq for Catalogues {
    fn eq(&self, autre: &Self) -> bool {
        std::ptr::eq(self.0, autre.0)
    }
}

/// Ce que le contexte donne à toute l'interface.
#[derive(Clone, PartialEq)]
pub struct I18n {
    pub language: Language,
    pub set_language: Callback<Language>,
    catalogues: Catalogues,
}

impl I18n {
    /// Traduit une clé. Les catalogues sont lus dans l'ordre : celui de
    /// l'application d'abord, le partagé ensuite.
    pub fn t(&self, key: &str) -> String {
        self.translator().t(key)
    }

    /// Traduit une clé en bouchant ses trous.
    pub fn with(&self, key: &str, params: &Params) -> String {
        self.translator().with(key, params)
    }

    /// Nombres, unités et pourcentages dans la langue courante.
    pub fn f(&self) -> Formats {
        Formats::new(self.language)
    }

    /// Étiquette de locale, pour le formatage des dates par le navigateur.
    pub fn locale(&self) -> &'static str {
        self.language.locale()
    }

    /// Le traducteur du cœur, pour les fonctions qui formulent elles-mêmes —
    /// `describe_blocker`, par exemple.
    pub fn translator(&self) -> Translator<'static> {
        let mut catalogues: Vec<&'static MessageSet> = self.catalogues.0.to_vec();
        // Le catalogue partagé vient en dernier : une interface peut redire un
        // libellé commun, jamais l'inverse.
        catalogues.push(&SHARED_MESSAGES);
        Translator::new(self.language, &catalogues)
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Les catalogues propres à l'interface. Le partagé est ajouté derrière.
    pub catalogues: Catalogues,
    pub children: Html,
}

#[function_component]
pub fn I18nProvider(props: &Props) -> Html {
    let language = use_state(depart);

    let set_language = {
        let language = language.clone();
        Callback::from(move |next: Language| language.set(next))
    };

    // La langue de la page suit celle de l'interface : c'est ce qui permet au
    // navigateur de proposer la traduction, et aux lecteurs d'écran de choisir
    // la bonne voix.
    use_effect_with(*language, |language| {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(html) = document.document_element() {
                let _ = html.set_attribute("lang", language.code());
            }
        }
        storage::set(STORAGE_KEY, language.code());
    });

    let contexte = I18n { language: *language, set_language, catalogues: props.catalogues };

    html! {
        <ContextProvider<I18n> context={contexte}>
            { props.children.clone() }
        </ContextProvider<I18n>>
    }
}

/// Le contexte de langue. Absent, c'est un montage incomplet : la langue de
/// référence vaut mieux qu'un écran blanc.
#[hook]
pub fn use_i18n() -> I18n {
    use_context::<I18n>().unwrap_or_else(|| I18n {
        language: REFERENCE_LANGUAGE,
        set_language: Callback::noop(),
        catalogues: Catalogues(&[]),
    })
}

fn depart() -> Language {
    storage::get(STORAGE_KEY)
        .and_then(|code| Language::parse(&code))
        .unwrap_or_else(|| detect_language(&langues_du_navigateur()))
}

fn langues_du_navigateur() -> Vec<String> {
    let Some(window) = web_sys::window() else { return Vec::new() };
    let navigator = window.navigator();

    let liste: Vec<String> = navigator
        .languages()
        .iter()
        .filter_map(|valeur| valeur.as_string())
        .collect();

    if liste.is_empty() { navigator.language().into_iter().collect() } else { liste }
}
