//! Demander au relais quel palier il accorde.
//!
//! Miroir Swift : `ios/Kliima/Services/PlanGrant.swift`. Même mécanique des
//! deux côtés — une adresse présentée, un verdict rendu — et la même prudence :
//! tout ce qui n'est pas un « oui » franc vaut le palier libre.
//!
//! L'appel lui-même n'est pas ici : ce module construit l'adresse et lit la
//! réponse, deux choses pures, donc vérifiables sans réseau. C'est là que se
//! jouent l'échappement et le nom des paramètres, et c'est précisément ce qui
//! s'est révélé piégeux côté iPhone.

use klima_core::plan::Plan;

/// L'adresse à interroger.
///
/// L'échappement n'est pas un détail. Une query se lit en form-urlencoded, où
/// `+` vaut une espace : `max+ferme@ferme.fr` recopié tel quel arriverait au
/// relais avec un trou au milieu et ne correspondrait à aucune invitation. Le
/// sérialiseur de `url` encode `+` en `%2B`, et c'est pour cela qu'on passe par
/// lui plutôt que par une concaténation.
pub fn plan_url(relais: &str, courriel: Option<&str>) -> String {
    let base = relais.trim_end_matches('/');
    let courriel = courriel.map(str::trim).filter(|c| !c.is_empty());

    match courriel {
        None => format!("{base}/v1/plan"),
        Some(courriel) => {
            let query = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("courriel", courriel)
                .finish();
            format!("{base}/v1/plan?{query}")
        }
    }
}

/// Lit la réponse du relais : `{"plan":"pro"}` ou `{"plan":"libre"}`.
///
/// Un relais en panne, une réponse tronquée, une page d'erreur : aucune de ces
/// choses n'ouvre un palier payant. Le doute vaut toujours « libre ».
pub fn decode_plan(corps: &str) -> Plan {
    let lu: Option<String> = serde_json::from_str::<serde_json::Value>(corps)
        .ok()
        .and_then(|valeur| valeur.get("plan")?.as_str().map(str::to_owned));

    match lu.as_deref() {
        Some(code) if code == Plan::Pro.code() => Plan::Pro,
        _ => Plan::Libre,
    }
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sans_adresse_il_ny_a_pas_de_query() {
        assert_eq!(plan_url("https://exemple.test", None), "https://exemple.test/v1/plan");
        // Une saisie vide ou blanche ne vaut pas une adresse.
        assert_eq!(plan_url("https://exemple.test", Some("   ")), "https://exemple.test/v1/plan");
    }

    #[test]
    fn la_barre_finale_du_relais_ne_se_double_pas() {
        assert_eq!(plan_url("https://exemple.test/", None), "https://exemple.test/v1/plan");
    }

    #[test]
    fn ladresse_voyage_sous_courriel() {
        assert_eq!(
            plan_url("https://exemple.test", Some("max@ferme.fr")),
            "https://exemple.test/v1/plan?courriel=max%40ferme.fr"
        );
    }

    #[test]
    fn une_adresse_a_etiquette_est_encodee() {
        // Le piège : « + » vaut une espace dans une query. Sans encodage,
        // l'adresse arriverait au relais avec un trou au milieu.
        let url = plan_url("https://exemple.test", Some("max+ferme@ferme.fr"));
        assert_eq!(url, "https://exemple.test/v1/plan?courriel=max%2Bferme%40ferme.fr");
        assert!(!url.contains('+'), "{url}");
    }

    #[test]
    fn une_saisie_ne_peut_pas_ajouter_un_parametre() {
        let url = plan_url("https://exemple.test", Some("a&code=x@ferme.fr"));
        assert!(!url.contains("&code="), "{url}");
        assert!(url.contains("%26"), "{url}");
    }

    #[test]
    fn les_blancs_autour_tombent_avant_lenvoi() {
        assert_eq!(
            plan_url("https://exemple.test", Some("  max@ferme.fr\n")),
            "https://exemple.test/v1/plan?courriel=max%40ferme.fr"
        );
    }

    #[test]
    fn un_oui_franc_accorde_le_palier() {
        assert_eq!(decode_plan(r#"{"plan":"pro"}"#), Plan::Pro);
    }

    #[test]
    fn tout_le_reste_vaut_libre() {
        for corps in [
            r#"{"plan":"libre"}"#,
            r#"{"plan":"Pro"}"#,
            r#"{"plan":""}"#,
            r#"{"palier":"pro"}"#,
            r#"{"plan":1}"#,
            "{}",
            "pro",
            "<html>502 Bad Gateway</html>",
            "",
        ] {
            assert_eq!(decode_plan(corps), Plan::Libre, "{corps}");
        }
    }
}
