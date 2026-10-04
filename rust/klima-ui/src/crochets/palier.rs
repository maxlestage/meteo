//! Le palier que le déploiement accorde, vu du navigateur.
//!
//! Miroir Swift : `ios/Kliima/Services/PlanGrant.swift`. Même mécanique — une
//! adresse présentée, un verdict rendu par le relais — et la même prudence :
//! sans relais configuré, aucun appel n'est fait et rien ne change.
//!
//! L'adresse n'est pas un compte. Le relais ne la retient pas, rien n'en dépend
//! qu'une question posée à l'ouverture, et l'effacer suffit à se retirer.
//! Ce n'est pas non plus une identification : personne ne vérifie que celui qui
//! présente une adresse la relève. Pour un essai fermé, c'est assumé.
//!
//! **Un écart avec iOS, et il est voulu.** Là-bas le palier est recopié sur le
//! disque et survit à un relais muet : des fonctions en dépendent, et un
//! testeur ne doit pas les perdre dans un tunnel. Ici rien n'est verrouillé —
//! l'adresse seule est mémorisée, le palier se redemande à chaque ouverture, et
//! un relais injoignable laisse simplement le champ. Mémoriser un palier que
//! rien ne consulte serait de l'état pour l'état. En revanche, dans la session,
//! un silence ne retire rien : vérifier hors réseau ne doit pas afficher un
//! refus qui n'a pas eu lieu.

use klima_api::plan::{Verdict, plan_url};
use klima_core::plan::Plan;
use yew::prelude::*;

use crate::reseau;
use crate::storage;

/// Clé de mémorisation de l'adresse d'essai.
const STORAGE_KEY: &str = "klima.courriel";

#[derive(Clone, PartialEq)]
struct Etat {
    plan: Plan,
    courriel: String,
    verification: bool,
    /// Vrai quand une adresse a été présentée et n'a rien ouvert.
    refuse: bool,
}

#[derive(Clone, PartialEq)]
pub struct Palier {
    pub plan: Plan,
    pub courriel: String,
    pub verification: bool,
    pub refuse: bool,
    /// Enregistre l'adresse et redemande le palier. Une saisie vide efface.
    pub verifier: Callback<String>,
    /// Vrai s'il y a un relais à interroger. Sans lui, l'interface n'affiche
    /// même pas le champ : un champ qui ne peut rien ouvrir est une promesse
    /// en l'air.
    pub possible: bool,
}

#[hook]
pub fn use_palier(relais: Option<String>) -> Palier {
    let etat = use_state(|| Etat {
        plan: Plan::Libre,
        courriel: storage::get(STORAGE_KEY).unwrap_or_default(),
        verification: false,
        refuse: false,
    });
    // Redemander la même adresse doit repartir : une invitation peut arriver
    // après coup, et sans cela l'écran resterait sur son refus.
    let nonce = use_state(|| 0_u32);

    {
        let etat = etat.clone();
        let relais = relais.clone();
        let courriel = etat.courriel.clone();
        use_effect_with((relais, courriel, *nonce), move |(relais, courriel, _)| {
            let Some(relais) = relais.clone() else { return };
            let courriel = courriel.trim().to_owned();

            // On demande même sans adresse. Un relais réglé sur « tous »
            // accorde à qui demande, sans rien présenter : c'est ainsi que
            // l'iPhone l'obtient, et s'abstenir ici faisait répondre les deux
            // plateformes différemment à la même variable — le palier
            // s'ouvrait sur le téléphone et pas sur le site.
            let presente = (!courriel.is_empty()).then(|| courriel.clone());

            wasm_bindgen_futures::spawn_local(async move {
                let url = plan_url(&relais, presente.as_deref());
                // Un relais muet ne retire rien : on garde ce qu'on savait.
                // Et « refusée » ne se dit que d'une adresse réellement
                // présentée : sans rien à refuser, un « libre » est la réponse
                // normale d'un relais qui attend qu'on se nomme.
                let (plan, refuse) = match reseau::plan(&url).await {
                    Verdict::Accorde => (Plan::Pro, false),
                    Verdict::Refuse => (Plan::Libre, presente.is_some()),
                    Verdict::Injoignable => (etat.plan, false),
                };
                etat.set(Etat { plan, refuse, courriel, verification: false });
            });
        });
    }

    let verifier = {
        let etat = etat.clone();
        let nonce = nonce.clone();
        Callback::from(move |saisie: String| {
            let propre = saisie.trim().to_owned();
            storage::set(STORAGE_KEY, &propre);

            if propre.is_empty() {
                // Se retirer est immédiat et ne demande rien à personne.
                etat.set(Etat {
                    plan: Plan::Libre,
                    courriel: String::new(),
                    verification: false,
                    refuse: false,
                });
                return;
            }

            etat.set(Etat {
                plan: etat.plan,
                courriel: propre,
                verification: true,
                refuse: false,
            });
            nonce.set(*nonce + 1);
        })
    };

    Palier {
        plan: etat.plan,
        courriel: etat.courriel.clone(),
        verification: etat.verification,
        refuse: etat.refuse,
        verifier,
        possible: relais.is_some(),
    }
}
