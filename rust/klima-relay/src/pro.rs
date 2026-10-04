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
//! `KLIMA_PRO` dit laquelle des quatre situations on est :
//!
//! | Valeur                     | Ce que le relais répond                      |
//! | -------------------------- | -------------------------------------------- |
//! | absente, vide, `0`, `non`  | rien : StoreKit décide seul                  |
//! | `tous`, `1`, `oui`         | le palier payant, à qui demande              |
//! | une valeur contenant `@`   | le palier payant, aux adresses énumérées     |
//! | n'importe quoi d'autre     | le palier payant, à qui présente ce code     |
//!
//! La deuxième forme est franche : un relais public qui accorde le palier à qui
//! demande l'accorde à **tout le monde**. C'est exactement ce qu'on veut tant
//! que le relais n'est connu que des testeurs qu'on a invités.
//!
//! La troisième est celle qui vaut pour un essai nominatif, et c'est la seule
//! qui n'oblige à rien embarquer : une adresse électronique n'est pas un
//! secret, le testeur la connaît déjà, elle ne voyage pas dans le binaire, et
//! elle se retire une par une sans toucher aux autres.
//!
//! **Ce qu'une liste d'adresses n'est pas.** Ce n'est pas une preuve
//! d'identité : personne ne vérifie que celui qui présente une adresse la
//! relève. Quiconque connaît une adresse invitée obtient le palier. Pour un
//! essai fermé, c'est le bon compromis — c'est plus étroit que `tous` et plus
//! révocable qu'un code partagé. Ce n'est pas de quoi vendre un abonnement :
//! ça, c'est le travail de StoreKit, et il reste entier.

/// Ce que le déploiement accorde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Accord {
    /// Rien. StoreKit décide seul, comme en production.
    Aucun,
    /// À qui demande — la phase d'essai, relais confidentiel.
    Tous,
    /// À qui présente ce code.
    SurCode(String),
    /// À qui présente une de ces adresses, déjà normalisées.
    SurListe(Vec<String>),
}

impl Accord {
    /// Ce que `/health` peut dire sans rien révéler.
    ///
    /// Le nombre d'adresses en fait partie : il ne nomme personne, et c'est le
    /// seul moyen de voir du dehors qu'une virgule oubliée a réduit la liste à
    /// une seule entrée. Sans lui, un `KLIMA_PRO` mal écrit répond « libre »
    /// sans qu'on sache si c'est l'adresse ou la variable qui est en cause.
    pub fn etiquette(&self) -> String {
        match self {
            Accord::Aucun => "aucun".to_owned(),
            Accord::Tous => "tous".to_owned(),
            Accord::SurCode(_) => "sur code".to_owned(),
            Accord::SurListe(liste) => format!("sur liste ({})", liste.len()),
        }
    }

    /// Lit la variable d'environnement.
    pub fn depuis(valeur: Option<&str>) -> Accord {
        let valeur = valeur.unwrap_or("").trim();
        match valeur.to_lowercase().as_str() {
            "" | "0" | "non" | "no" | "false" => Accord::Aucun,
            "tous" | "1" | "oui" | "yes" | "true" | "all" => Accord::Tous,
            _ if valeur.contains('@') => {
                // Séparateurs admis : virgule, point-virgule, espace, retour à
                // la ligne. On accepte les quatre parce qu'une liste d'adresses
                // se colle depuis un courriel, un tableur ou une note, et que
                // chacun la sépare à sa façon.
                let liste: Vec<String> = valeur
                    .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
                    .map(normaliser)
                    // Une entrée sans arobase est une faute de frappe, pas une
                    // adresse. La garder reviendrait à ouvrir le palier payant
                    // à qui taperait ce mot-là : on la laisse tomber, et le
                    // compte affiché par `/health` rend la perte visible.
                    .filter(|adresse| adresse.contains('@'))
                    .collect();
                if liste.is_empty() { Accord::Aucun } else { Accord::SurListe(liste) }
            }
            _ => Accord::SurCode(valeur.to_owned()),
        }
    }

    /// Le palier accordé à cette demande.
    ///
    /// Les deux formes de preuve sont séparées et ne se remplacent pas : une
    /// liste d'adresses ignore un code, un code ignore une adresse. Mélanger
    /// les deux donnerait deux portes là où on en voulait une.
    pub fn accorde(&self, code: Option<&str>, courriel: Option<&str>) -> bool {
        match self {
            Accord::Aucun => false,
            Accord::Tous => true,
            Accord::SurCode(attendu) => code.is_some_and(|donne| egal(attendu, donne)),
            Accord::SurListe(liste) => courriel.is_some_and(|donne| {
                let donne = normaliser(donne);
                // Pas de sortie au premier accord : la liste est parcourue en
                // entier. Elle n'est pas secrète, mais la boucle est si courte
                // que s'en priver ne gagne rien.
                liste.iter().fold(false, |trouve, attendu| trouve | egal(attendu, &donne))
            }),
        }
    }
}

/// Une adresse comparable : sans blancs autour, en minuscules.
///
/// La partie gauche d'une adresse est sensible à la casse selon la norme, et
/// chez aucun fournisseur réel. On aligne donc sur l'usage : `Max@Ferme.fr` et
/// `max@ferme.fr` sont la même personne, et refuser la seconde ferait passer un
/// clavier à majuscules pour une panne.
fn normaliser(valeur: &str) -> String {
    valeur.trim().to_lowercase()
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
            assert!(!Accord::depuis(valeur).accorde(None, None));
            assert!(!Accord::depuis(valeur).accorde(Some("n'importe quoi"), None));
            assert!(!Accord::depuis(valeur).accorde(None, Some("max@ferme.fr")));
        }
    }

    #[test]
    fn tous_accorde_a_qui_demande() {
        for valeur in ["tous", "TOUS", "1", "oui", "true", "all"] {
            let accord = Accord::depuis(Some(valeur));
            assert_eq!(accord, Accord::Tous, "{valeur}");
            assert!(accord.accorde(None, None));
        }
    }

    #[test]
    fn un_code_naccorde_quau_code() {
        let accord = Accord::depuis(Some("sillon-2026-dUx7"));

        assert!(accord.accorde(Some("sillon-2026-dUx7"), None));
        assert!(!accord.accorde(Some("sillon-2026-dUx8"), None));
        assert!(!accord.accorde(Some("sillon"), None));
        assert!(!accord.accorde(Some(""), None));
        // Demander sans rien présenter n'ouvre pas davantage.
        assert!(!accord.accorde(None, None));
        // Et une adresse ne tient pas lieu de code.
        assert!(!accord.accorde(None, Some("sillon-2026-dUx7")));
    }

    /* ---- la liste d'adresses ---- */

    #[test]
    fn une_adresse_suffit_a_faire_une_liste() {
        let accord = Accord::depuis(Some("max@ferme.fr"));
        assert_eq!(accord, Accord::SurListe(vec!["max@ferme.fr".to_owned()]));
        assert!(accord.accorde(None, Some("max@ferme.fr")));
        assert!(!accord.accorde(None, Some("autre@ferme.fr")));
        assert!(!accord.accorde(None, None));
        // Un code ne tient pas lieu d'adresse.
        assert!(!accord.accorde(Some("max@ferme.fr"), None));
    }

    #[test]
    fn les_quatre_separateurs_donnent_la_meme_liste() {
        let attendue = vec![
            "max@ferme.fr".to_owned(),
            "ana@vina.es".to_owned(),
            "jo@farm.uk".to_owned(),
        ];
        for valeur in [
            "max@ferme.fr,ana@vina.es,jo@farm.uk",
            "max@ferme.fr, ana@vina.es, jo@farm.uk",
            "max@ferme.fr; ana@vina.es; jo@farm.uk",
            "max@ferme.fr ana@vina.es jo@farm.uk",
            "max@ferme.fr\nana@vina.es\njo@farm.uk",
            "  max@ferme.fr ,\n ana@vina.es ;\tjo@farm.uk  ",
        ] {
            assert_eq!(Accord::depuis(Some(valeur)), Accord::SurListe(attendue.clone()), "{valeur}");
        }
    }

    #[test]
    fn la_casse_ne_fait_pas_perdre_son_palier() {
        let accord = Accord::depuis(Some("Max@Ferme.FR"));
        assert_eq!(accord, Accord::SurListe(vec!["max@ferme.fr".to_owned()]));
        for presentee in ["max@ferme.fr", "MAX@FERME.FR", "Max@Ferme.fr", "  max@ferme.fr  "] {
            assert!(accord.accorde(None, Some(presentee)), "{presentee}");
        }
    }

    #[test]
    fn une_entree_sans_arobase_est_une_faute_de_frappe_et_tombe() {
        // Sans ce filtre, taper « oups » ouvrirait le palier payant.
        let accord = Accord::depuis(Some("max@ferme.fr, oups, ana@vina.es"));
        assert_eq!(
            accord,
            Accord::SurListe(vec!["max@ferme.fr".to_owned(), "ana@vina.es".to_owned()])
        );
        assert!(!accord.accorde(None, Some("oups")));
        assert!(accord.accorde(None, Some("ana@vina.es")));
    }

    #[test]
    fn une_valeur_qui_na_que_des_fautes_naccorde_rien() {
        // « @ » tout seul contient une arobase mais ne nomme personne ; on ne
        // veut surtout pas d'un palier qui s'ouvre à une chaîne vide.
        let accord = Accord::depuis(Some("@"));
        assert_eq!(accord, Accord::SurListe(vec!["@".to_owned()]));
        assert!(!accord.accorde(None, Some("")));
        assert!(!accord.accorde(None, Some("max@ferme.fr")));
    }

    #[test]
    fn letiquette_ne_dit_jamais_le_code_ni_les_adresses() {
        let code = Accord::depuis(Some("sillon-2026-dUx7"));
        assert_eq!(code.etiquette(), "sur code");
        assert!(!code.etiquette().contains("sillon"));

        let liste = Accord::depuis(Some("max@ferme.fr, ana@vina.es"));
        assert_eq!(liste.etiquette(), "sur liste (2)");
        assert!(!liste.etiquette().contains('@'));
        assert!(!liste.etiquette().contains("max"));

        assert_eq!(Accord::Aucun.etiquette(), "aucun");
        assert_eq!(Accord::Tous.etiquette(), "tous");
    }

    #[test]
    fn letiquette_compte_ce_qui_a_survecu_au_filtre() {
        // C'est tout l'intérêt du compte : voir du dehors qu'une entrée est
        // tombée, sans nommer celles qui restent.
        assert_eq!(Accord::depuis(Some("max@ferme.fr, oups")).etiquette(), "sur liste (1)");
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
