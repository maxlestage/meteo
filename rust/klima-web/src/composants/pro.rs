//! Ce que l'abonnement ajoute, et où.
//!
//! Miroir de `web/src/components/ProNote.tsx`.
//!
//! L'application web n'a pas de boutique : elle ne peut pas vendre, et lui
//! greffer des comptes pour encaisser un euro par mois coûterait plus que ça
//! ne rapporterait. Elle dit donc simplement ce qui existe ailleurs.
//!
//! La liste vient de `FEATURES`, dans le cœur partagé : une fonction ajoutée
//! au palier payant apparaît ici sans qu'on y touche, et ne peut pas être
//! oubliée.

use klima_core::i18n::params;
use klima_core::plan::{FEATURES, Feature, Plan, limits_for};
use yew::prelude::*;

use crate::i18n::use_i18n;

/// Ce que le web donne malgré tout.
///
/// Le recoupement des instituts se paie dans l'application, mais reste offert
/// ici : c'est l'argument qui vend le produit, et le montrer à l'œuvre
/// convainc mieux que le décrire. L'annoncer comme payant à quelqu'un qui l'a
/// sous les yeux serait au mieux confus, au pire malhonnête.
const OFFERT_SUR_LE_WEB: [Feature; 1] = [Feature::Recoupement];

#[function_component]
pub fn NotePro() -> Html {
    let i18n = use_i18n();
    let libre = limits_for(Plan::Libre);

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
        </section>
    }
}
