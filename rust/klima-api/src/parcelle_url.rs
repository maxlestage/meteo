//! La parcelle consultée, dans l'adresse.
//!
//! Sans cela, choisir une commune ne laisse aucune trace : le bouton retour du
//! navigateur ne défait rien, recharger la page perd le choix, et envoyer
//! l'adresse à quelqu'un lui montre une autre parcelle que la sienne. Trois
//! défauts pour une seule cause — l'état n'était pas dans l'URL.
//!
//! Ce module n'a pas de miroir Swift, et c'est voulu : ce n'est pas une règle
//! de météo mais une convention de navigation propre au web. L'application
//! iOS n'a pas d'adresse à porter.

use url::Url;

use crate::open_meteo::Parcelle;

/// Quatre décimales : environ onze mètres, bien au-delà du besoin.
const PRECISION: usize = 4;

/// Les clés que la parcelle occupe dans l'adresse.
const CLES: [&str; 5] = ["parcelle", "lat", "lon", "admin", "pays"];

/// Écrit la parcelle en paramètres d'adresse, dans l'ordre où on les lit.
pub fn parcelle_to_query(parcelle: &Parcelle) -> String {
    let mut paires: Vec<(&str, String)> = vec![
        ("parcelle", parcelle.name.clone()),
        ("lat", format!("{:.*}", PRECISION, parcelle.latitude)),
        ("lon", format!("{:.*}", PRECISION, parcelle.longitude)),
    ];
    if let Some(admin) = &parcelle.admin {
        paires.push(("admin", admin.clone()));
    }
    if let Some(country) = &parcelle.country {
        paires.push(("pays", country.clone()));
    }

    let mut serialiseur = url::form_urlencoded::Serializer::new(String::new());
    for (cle, valeur) in &paires {
        serialiseur.append_pair(cle, valeur);
    }
    serialiseur.finish()
}

/// Relit une parcelle depuis une chaîne de paramètres.
///
/// Renvoie `None` dès qu'un champ manque ou sort des bornes : une adresse
/// bricolée à la main ne doit pas envoyer l'application chercher la météo d'un
/// point qui n'existe pas.
pub fn parcelle_from_query(query: &str) -> Option<Parcelle> {
    let params: Vec<(String, String)> =
        url::form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
    let lire = |nom: &str| {
        params.iter().find(|(k, _)| k == nom).map(|(_, v)| v.trim()).filter(|v| !v.is_empty())
    };

    let name = lire("parcelle")?.to_owned();
    // Sans ce garde-fou, des coordonnées manquantes vaudraient zéro et l'on
    // irait chercher la météo du golfe de Guinée.
    let latitude: f64 = lire("lat")?.parse().ok()?;
    let longitude: f64 = lire("lon")?.parse().ok()?;

    if !latitude.is_finite() || !longitude.is_finite() {
        return None;
    }
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        return None;
    }

    Some(Parcelle {
        name,
        latitude,
        longitude,
        admin: lire("admin").map(str::to_owned),
        country: lire("pays").map(str::to_owned),
    })
}

/// Vrai si les deux adresses désignent la même parcelle.
pub fn same_parcelle(a: Option<&Parcelle>, b: Option<&Parcelle>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            let arrondi = |v: f64| format!("{:.*}", PRECISION, v);
            a.name == b.name
                && arrondi(a.latitude) == arrondi(b.latitude)
                && arrondi(a.longitude) == arrondi(b.longitude)
        }
        _ => false,
    }
}

/// L'adresse à afficher pour une parcelle, en gardant le reste de l'existante.
///
/// On conserve le fragment : sur la vitrine, il désigne la section où l'on se
/// trouve, et le perdre au moment de choisir une commune ferait sauter la page
/// en haut.
pub fn url_for_parcelle(current: &str, parcelle: &Parcelle) -> String {
    let Ok(url) = Url::parse(current) else {
        return format!("?{}", parcelle_to_query(parcelle));
    };

    // Les autres paramètres survivent ; ceux de la parcelle sont remplacés, pas
    // empilés.
    let mut autres: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(cle, _)| !CLES.contains(&cle.as_ref()))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    let mut serialiseur = url::form_urlencoded::Serializer::new(String::new());
    for (cle, valeur) in autres.drain(..) {
        serialiseur.append_pair(&cle, &valeur);
    }
    let avant = serialiseur.finish();

    let query = if avant.is_empty() {
        parcelle_to_query(parcelle)
    } else {
        format!("{avant}&{}", parcelle_to_query(parcelle))
    };

    format!("{}?{query}{}", url.path(), url.fragment().map_or(String::new(), |f| format!("#{f}")))
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    fn reims() -> Parcelle {
        Parcelle {
            name: "Reims".to_owned(),
            latitude: 49.2628,
            longitude: 4.0347,
            admin: Some("Grand Est".to_owned()),
            country: Some("France".to_owned()),
        }
    }

    fn nue(name: &str, latitude: f64, longitude: f64) -> Parcelle {
        Parcelle { name: name.to_owned(), latitude, longitude, admin: None, country: None }
    }

    #[test]
    fn un_aller_retour_rend_la_meme_parcelle() {
        assert_eq!(parcelle_from_query(&parcelle_to_query(&reims())), Some(reims()));
    }

    #[test]
    fn une_parcelle_sans_region_ni_pays_reste_lisible() {
        let nu = nue("Le Clos", 48.4468, 1.4892);
        assert_eq!(parcelle_from_query(&parcelle_to_query(&nu)), Some(nu));
    }

    #[test]
    fn ladresse_se_lit_a_loeil_nu() {
        assert_eq!(
            parcelle_to_query(&reims()),
            "parcelle=Reims&lat=49.2628&lon=4.0347&admin=Grand+Est&pays=France"
        );
    }

    #[test]
    fn une_adresse_sans_parcelle_ne_renvoie_rien() {
        assert_eq!(parcelle_from_query(""), None);
        assert_eq!(parcelle_from_query("lat=49&lon=4"), None);
    }

    #[test]
    fn des_coordonnees_manquantes_ne_valent_pas_zero() {
        assert_eq!(parcelle_from_query("parcelle=Reims"), None);
    }

    #[test]
    fn une_adresse_bricolee_hors_bornes_est_refusee() {
        for query in
            ["parcelle=X&lat=95&lon=4", "parcelle=X&lat=49&lon=200", "parcelle=X&lat=abc&lon=4"]
        {
            assert_eq!(parcelle_from_query(query), None, "{query}");
        }
    }

    #[test]
    fn un_nom_vide_ne_fait_pas_une_parcelle() {
        assert_eq!(parcelle_from_query("parcelle=%20%20&lat=49&lon=4"), None);
    }

    #[test]
    fn deux_adresses_de_la_meme_parcelle_se_reconnaissent() {
        let presque = Parcelle { latitude: 49.26281, ..reims() };
        let ailleurs = Parcelle { latitude: 49.27, ..reims() };

        assert!(same_parcelle(Some(&reims()), Some(&presque)));
        assert!(!same_parcelle(Some(&reims()), Some(&ailleurs)));
        assert!(same_parcelle(None, None));
        assert!(!same_parcelle(Some(&reims()), None));
    }

    #[test]
    fn changer_de_parcelle_garde_le_fragment_de_la_page() {
        // Sur la vitrine, le fragment dit où l'on est : le perdre ferait
        // sauter la page en haut au moment de choisir une commune.
        assert_eq!(
            url_for_parcelle("https://klima.test/meteo/#aujourdhui", &reims()),
            "/meteo/?parcelle=Reims&lat=49.2628&lon=4.0347&admin=Grand+Est&pays=France#aujourdhui"
        );
    }

    #[test]
    fn changer_de_parcelle_remplace_lancienne_au_lieu_de_sempiler() {
        let premiere = url_for_parcelle("https://klima.test/app/", &reims());
        let seconde = url_for_parcelle(
            &format!("https://klima.test{premiere}"),
            &nue("Chartres", 48.4468, 1.4892),
        );

        assert_eq!(seconde, "/app/?parcelle=Chartres&lat=48.4468&lon=1.4892");
        assert!(!seconde.contains("Reims"));
    }

    #[test]
    fn les_autres_parametres_de_ladresse_survivent() {
        assert!(url_for_parcelle("https://klima.test/app/?debug=1", &reims()).contains("debug=1"));
    }
}
