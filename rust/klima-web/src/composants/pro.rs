//! Ce que l'abonnement ajoute, et où.
//!
//! L'application web n'a pas de boutique : elle ne peut pas vendre, et lui
//! greffer des comptes pour encaisser un euro par mois coûterait plus que ça
//! ne rapporterait. Elle dit donc ce qui existe ailleurs.
//!
//! La liste vient de `FEATURES`, dans le cœur partagé : une fonction ajoutée
//! au palier payant apparaît ici sans qu'on y touche, et ne peut pas être
//! oubliée.
//!
//! **L'accès de test.** Pendant l'essai, une adresse invitée fait reconnaître
//! le palier payant par le relais. Ce que cela change ici est modeste et la
//! section le dit : sur le web, le recoupement et l'air sont déjà offerts — les
//! alertes n'ont pas d'implémentation web, elles vivent dans l'application
//! iPhone. Afficher « Pro actif » en laissant croire le contraire
//! serait la seule chose à ne pas faire.

use klima_core::i18n::params;
use klima_core::plan::{FEATURES, Feature, Plan, limits_for};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use klima_ui::crochets::palier::Palier;
use klima_ui::i18n::use_i18n;

/// Ce que le web donne malgré tout.
///
/// Le recoupement des sources et l'air se paient dans l'application, mais
/// restent offerts ici : c'est ce qui vend le produit, et le montrer à
/// l'œuvre convainc mieux que le décrire. L'annoncer comme payant à quelqu'un
/// qui l'a sous les yeux serait au mieux confus, au pire malhonnête.
const OFFERT_SUR_LE_WEB: [Feature; 2] = [Feature::Recoupement, Feature::Air];

#[derive(Properties, PartialEq)]
pub struct Props {
    pub palier: Palier,
}

#[function_component]
pub fn NotePro(props: &Props) -> Html {
    let i18n = use_i18n();
    let libre = limits_for(Plan::Libre);
    let palier = &props.palier;

    let manquantes: Html = FEATURES
        .into_iter()
        .filter(|feature| !OFFERT_SUR_LE_WEB.contains(feature))
        .map(|feature| html! { <li key={feature.code()}>{ i18n.t(feature.feature_key()) }</li> })
        .collect();

    html! {
        <section class="pro" aria-labelledby="pro-title">
            <h2 id="pro-title">{ i18n.t("plan.pro") }</h2>
            <p class="pro__lead">{ i18n.t("pro.lead") }</p>
            <ul class="pro__features">{ manquantes }</ul>
            <p class="pro__free">
                { i18n.with("pro.free", &params([
                    ("parcelles", libre.parcelles.unwrap_or(0).into()),
                    ("jours", (libre.jours as usize).into()),
                ])) }
            </p>
            <AccesDEssai palier={palier.clone()} />
        </section>
    }
}

/// Le champ d'accès de test, et seulement s'il y a un relais pour répondre.
#[function_component]
fn AccesDEssai(props: &Props) -> Html {
    let i18n = use_i18n();
    let palier = &props.palier;
    let saisie = use_state(|| palier.courriel.clone());

    if !palier.possible {
        return Html::default();
    }

    let sur_saisie = {
        let saisie = saisie.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(champ) = event.target_dyn_into::<HtmlInputElement>() {
                saisie.set(champ.value());
            }
        })
    };

    let verifier = {
        let verifier = palier.verifier.clone();
        let saisie = saisie.clone();
        Callback::from(move |_| verifier.emit((*saisie).clone()))
    };

    let retirer = {
        let verifier = palier.verifier.clone();
        let saisie = saisie.clone();
        Callback::from(move |_| {
            saisie.set(String::new());
            verifier.emit(String::new());
        })
    };

    if palier.plan == Plan::Pro {
        return html! {
            <div class="pro__grant">
                <p class="pro__granted">{ i18n.t("pro.grant.active") }</p>
                // Ce que l'accès change ici, sans le surestimer.
                <p class="pro__granted-note">{ i18n.t("pro.grant.web") }</p>
                <button type="button" class="button" onclick={retirer}>
                    { i18n.t("pro.grant.forget") }
                </button>
            </div>
        };
    }

    let vide = saisie.trim().is_empty();

    html! {
        <div class="pro__grant">
            <h3 class="pro__grant-title">{ i18n.t("pro.grant.title") }</h3>
            <p class="pro__grant-hint">{ i18n.t("pro.grant.hint") }</p>
            <div class="pro__grant-row">
                <input
                    class="pro__grant-field"
                    type="email"
                    inputmode="email"
                    autocomplete="email"
                    autocapitalize="none"
                    spellcheck="false"
                    value={(*saisie).clone()}
                    oninput={sur_saisie}
                    placeholder={i18n.t("pro.grant.field")}
                    aria-label={i18n.t("pro.grant.field")}
                />
                <button
                    type="button"
                    class="button"
                    onclick={verifier}
                    disabled={palier.verification || vide}
                >
                    { i18n.t(if palier.verification { "pro.grant.checking" } else { "pro.grant.check" }) }
                </button>
            </div>
            if palier.refuse {
                // On ne dit pas « adresse inconnue » : le relais ne le dit pas
                // non plus, et pour la même raison — ce serait donner de quoi
                // énumérer la liste une adresse à la fois.
                <p class="pro__grant-refused">{ i18n.t("pro.grant.refused") }</p>
            }
        </div>
    }
}
