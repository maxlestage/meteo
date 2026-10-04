//! Ce que le déploiement accorde comme palier.
//!
//! Un abonnement se vend par l'App Store, et StoreKit en tient le registre.
//! Mais pendant l'essai, personne n'achète : les testeurs doivent voir le
//! palier payant sans passer par la boutique, et l'auteur doit pouvoir le
//! rendre à n'importe quel moment sans republier une version.
//!
//! D'où cette variable, et sa place : **sur le serveur, pas dans le binaire**.
//! Une valeur glissée dans une application distribuée est une valeur publiée,
//! qu'on ne peut plus retirer qu'en publiant une nouvelle version ; ici, elle
//! s'enlève en une commande et le palier redescend au prochain démarrage.
//!
//! `KLIMA_PRO` dit laquelle des trois situations on est :
//!
//! | Valeur            | Ce que le relais répond                        |
//! | ----------------- | ---------------------------------------------- |
//! | absente, vide, `0`, `non` | rien : StoreKit décide seul            |
//! | `tous`, `1`, `oui`        | le palier payant, à qui demande        |
//! | n'importe quoi d'autre    | le palier payant, à qui présente ce code |
//!
//! La troisième forme existe parce que la seconde est franche : un relais
//! public qui accorde le palier à qui demande l'accorde à **tout le monde**.
//! C'est exactement ce qu'on veut tant que les seuls clients sont les testeurs
//! qu'on a invités ; passé là, un code vaut mieux.

/// Ce que le déploiement accorde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Accord {
    /// Rien. StoreKit décide seul, comme en production.
    Aucun,
    /// À qui demande — la phase d'essai.
    Tous,
    /// À qui présente ce code.
    SurCode(String),
}

/// Ce que `/health` peut dire sans rien révéler.
impl Accord {
    pub fn etiquette(&self) -> &'static str {
        match self {
            Accord::Aucun => "aucun",
            Accord::Tous => "tous",
            Accord::SurCode(_) => "sur code",
        }
    }

    /// Lit la variable d'environnement.
    pub fn depuis(valeur: Option<&str>) -> Accord {
        let valeur = valeur.unwrap_or("").trim();
        match valeur.to_lowercase().as_str() {
            "" | "0" | "non" | "no" | "false" => Accord::Aucun,
            "tous" | "1" | "oui" | "yes" | "true" | "all" => Accord::Tous,
            _ => Accord::SurCode(valeur.to_owned()),
        }
    }

    /// Le palier accordé à cette demande.
    pub fn accorde(&self, code: Option<&str>) -> bool {
        match self {
            Accord::Aucun => false,
            Accord::Tous => true,
            Accord::SurCode(attendu) => code.is_some_and(|donne| egal(attendu, donne)),
        }
    }
}

/// Comparaison à durée constante.
///
/// Un `==` ordinaire s'arrête au premier octet qui diffère : en mesurant le
/// temps de réponse, on devine le code lettre par lettre. Le coût de s'en
/// prémunir est de quelques nanosecondes.
fn egal(attendu: &str, donne: &str) -> bool {
    let (a, b) = (attendu.as_bytes(), donne.as_bytes());
    let mut difference = a.len() ^ b.len();
    for index in 0..a.len().max(b.len()) {
        let octet_a = a.get(index).copied().unwrap_or(0);
        let octet_b = b.get(index).copied().unwrap_or(0);
        difference |= usize::from(octet_a ^ octet_b);
    }
    difference == 0
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rien_de_configure_storekit_decide_seul() {
        for valeur in [None, Some(""), Some("  "), Some("0"), Some("non"), Some("false")] {
            assert_eq!(Accord::depuis(valeur), Accord::Aucun, "{valeur:?}");
            assert!(!Accord::depuis(valeur).accorde(None));
            assert!(!Accord::depuis(valeur).accorde(Some("n'importe quoi")));
        }
    }

    #[test]
    fn tous_accorde_a_qui_demande() {
        for valeur in ["tous", "TOUS", "1", "oui", "true", "all"] {
            let accord = Accord::depuis(Some(valeur));
            assert_eq!(accord, Accord::Tous, "{valeur}");
            assert!(accord.accorde(None));
        }
    }

    #[test]
    fn un_code_naccorde_quau_code() {
        let accord = Accord::depuis(Some("sillon-2026-dUx7"));

        assert!(accord.accorde(Some("sillon-2026-dUx7")));
        assert!(!accord.accorde(Some("sillon-2026-dUx8")));
        assert!(!accord.accorde(Some("sillon")));
        assert!(!accord.accorde(Some("")));
        // Demander sans rien présenter n'ouvre pas davantage.
        assert!(!accord.accorde(None));
    }

    #[test]
    fn letiquette_ne_dit_jamais_le_code() {
        let accord = Accord::depuis(Some("sillon-2026-dUx7"));
        assert_eq!(accord.etiquette(), "sur code");
        assert!(!accord.etiquette().contains("sillon"));
    }

    #[test]
    fn la_comparaison_ne_sarrete_pas_au_premier_octet() {
        // On ne mesure pas le temps dans un test — on vérifie seulement que la
        // fonction juge sur la totalité, longueurs comprises.
        assert!(egal("abcd", "abcd"));
        assert!(!egal("abcd", "abce"));
        assert!(!egal("abcd", "abcde"));
        assert!(!egal("abcde", "abcd"));
        assert!(egal("", ""));
    }
}
