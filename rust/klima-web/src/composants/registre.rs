//! Le registre : les conditions à l'heure d'un traitement.
//!
//! Tenir un registre phytosanitaire est une obligation. Klima **n'est pas ce
//! registre** : il fournit la partie pénible à reconstituer après coup — le
//! vent, ses rafales, la température et l'hygrométrie relevés à l'heure de
//! l'application. Le reste appartient à l'exploitant.
//!
//! C'est une fonction du palier payant, et la première à exister ailleurs que
//! dans le domaine : `klima_core::register` était écrit et testé des deux
//! côtés, sans aucune interface pour l'appeler.
//!
//! **Le fichier ne passe pas par le réseau.** Il est fabriqué dans la page et
//! enregistré depuis la page. Un document qu'on peut vous opposer n'a aucune
//! raison de faire un aller-retour par un serveur pour revenir à vous, et
//! Klima n'a aucune raison de savoir ce que vous avez traité.

use klima_core::agro::HourlySample;
use klima_core::plan::Plan;
use klima_core::register::{CsvOptions, REGISTER_COLUMNS, record_at, to_csv};
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, HtmlInputElement, HtmlSelectElement, Url};
use yew::prelude::*;

use klima_ui::dates;
use klima_ui::i18n::{I18n, use_i18n};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub hours: Vec<HourlySample>,
    pub parcelle: String,
    pub plan: Plan,
    /// L'heure de la parcelle, pour proposer celle qui vient de passer.
    pub maintenant: i64,
}

#[function_component]
pub fn Registre(props: &Props) -> Html {
    let i18n = use_i18n();

    // L'heure retenue ; vide tant que personne n'a choisi, et on propose alors
    // celle qui contient l'instant présent — c'est celle qu'on veut neuf fois
    // sur dix, puisqu'on remplit son registre en sortant du champ.
    let choix = use_state(|| None::<i64>);
    let produit = use_state(String::new);

    if props.plan != Plan::Pro {
        return Html::default();
    }

    let defaut = heure_courante(&props.hours, props.maintenant);
    let retenue = choix.or(defaut);

    let sur_heure = {
        let choix = choix.clone();
        Callback::from(move |event: Event| {
            if let Some(champ) = event.target_dyn_into::<HtmlSelectElement>() {
                choix.set(champ.value().parse::<i64>().ok());
            }
        })
    };

    let sur_produit = {
        let produit = produit.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(champ) = event.target_dyn_into::<HtmlInputElement>() {
                produit.set(champ.value());
            }
        })
    };

    let telecharger = {
        let hours = props.hours.clone();
        let parcelle = props.parcelle.clone();
        let produit = produit.clone();
        let i18n = i18n.clone();
        Callback::from(move |_| {
            let Some(at) = retenue else { return };
            let produit = produit.trim().to_owned();
            let produit = (!produit.is_empty()).then_some(produit);

            let Some(releve) = record_at(&hours, at, &parcelle, produit.as_deref()) else {
                return;
            };
            let csv = to_csv(&[releve], &options(&i18n));
            enregistrer(&csv, &nom_de_fichier(&parcelle, at));
        })
    };

    let lignes: Html = props
        .hours
        .iter()
        .map(|hour| {
            html! {
                <option key={hour.time} value={hour.time.to_string()}
                        selected={retenue == Some(hour.time)}>
                    { dates::jour_et_heure(hour.time, i18n.locale()) }
                </option>
            }
        })
        .collect();

    html! {
        <section class="registre" aria-labelledby="registre-title">
            <h2 id="registre-title">{ i18n.t("register.title") }</h2>
            <p class="registre__lead">{ i18n.t("register.hint") }</p>

            <label class="registre__label" for="registre-heure">
                { i18n.t("register.hour") }
            </label>
            <select id="registre-heure" class="registre__field" onchange={sur_heure}>
                { lignes }
            </select>

            <label class="registre__label" for="registre-produit">
                { i18n.t("register.product") }
            </label>
            <input
                id="registre-produit"
                class="registre__field"
                type="text"
                value={(*produit).clone()}
                oninput={sur_produit}
                placeholder={i18n.t("register.productPlaceholder")}
            />

            <button type="button" class="button registre__download" onclick={telecharger}
                    disabled={retenue.is_none()}>
                { i18n.t("register.download") }
            </button>
            <p class="registre__note">{ i18n.t("register.note") }</p>
        </section>
    }
}

/// L'heure de la série qui contient l'instant présent, s'il y en a une.
fn heure_courante(hours: &[HourlySample], maintenant: i64) -> Option<i64> {
    const HEURE_MS: i64 = 3_600_000;
    let creneau = maintenant.div_euclid(HEURE_MS);
    hours.iter().find(|hour| hour.time.div_euclid(HEURE_MS) == creneau).map(|hour| hour.time)
}

/// Les réglages du tableur du lecteur.
///
/// Le point-virgule et la virgule décimale en français et en espagnol : Excel
/// dans ces langues lit un fichier à virgules comme une seule colonne. En
/// anglais, l'inverse. Les en-têtes sont traduits — le domaine rend des clés.
fn options(i18n: &I18n) -> CsvOptions {
    let mut options =
        if i18n.locale().starts_with("en") { CsvOptions::anglais() } else { CsvOptions::default() };
    options.headers = Some(
        REGISTER_COLUMNS
            .iter()
            .map(|colonne| i18n.t(&format!("register.column.{colonne}")))
            .collect(),
    );
    options
}

/// `klima-<parcelle>-<aaaammjj-hhmm>.csv`, sans rien qui gêne un système de
/// fichiers : un nom de commune peut porter une apostrophe ou une barre.
fn nom_de_fichier(parcelle: &str, at: i64) -> String {
    let propre: String = parcelle
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    let propre = propre.trim_matches('-').replace("--", "-");
    let date = klima_core::calendar::civil_from_ms(at);
    format!(
        "klima-{propre}-{:04}{:02}{:02}-{:02}{:02}.csv",
        date.year, date.month, date.day, date.hour, date.minute
    )
}

/// Fait enregistrer un texte comme fichier, sans serveur.
///
/// Toute étape peut manquer selon le navigateur ; aucune ne mérite de faire
/// tomber la page, et l'échec se voit tout seul — le fichier n'arrive pas.
fn enregistrer(contenu: &str, nom: &str) {
    let tableau = js_sys::Array::new();
    tableau.push(&wasm_bindgen::JsValue::from_str(contenu));

    let proprietes = BlobPropertyBag::new();
    proprietes.set_type("text/csv;charset=utf-8");

    let Ok(blob) = Blob::new_with_str_sequence_and_options(&tableau, &proprietes) else { return };
    let Ok(url) = Url::create_object_url_with_blob(&blob) else { return };

    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if let Ok(ancre) = document.create_element("a") {
            if let Ok(ancre) = ancre.dyn_into::<HtmlAnchorElement>() {
                ancre.set_href(&url);
                ancre.set_download(nom);
                ancre.click();
            }
        }
    }
    // L'objet est rendu tout de suite : le navigateur a déjà saisi le blob,
    // et le laisser traîner le garderait en mémoire jusqu'au rechargement.
    let _ = Url::revoke_object_url(&url);
}
