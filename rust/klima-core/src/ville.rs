//! La ville : ce qu'on regarde avant de sortir.
//!
//! Miroir Swift : `ios/Kliima/Models/Ville.swift`, avec les mêmes cas de test.
//! Toute règle ajoutée d'un côté se porte de l'autre.
//!
//! Trois questions, dans l'ordre où on se les pose sur le pas de la porte :
//! va-t-il pleuvoir, et quand ; que faut-il emporter ; le soleil tape-t-il.
//! Le domaine répond par des états et des motifs, jamais par des phrases :
//! l'interface les dit dans la langue de la personne.
//!
//! Les séries commencent à l'heure en cours — c'est ainsi que le décodage les
//! rend —, la première heure est donc « maintenant ».

use crate::meteo::HourlySample;

/// Les seuils, au même endroit que leur raison.
pub mod seuils {
    /// Une heure est pluvieuse à partir de ce cumul (mm) : en deçà, c'est de
    /// la bruine qu'on ne sent pas sous un auvent.
    pub const PLUIE_MM: f64 = 0.1;
    /// … ou à partir de ce risque (%). Une chance sur deux, c'est le moment
    /// où l'on prend le parapluie « au cas où ».
    pub const PLUIE_PROBABILITE: f64 = 50.0;
    /// On regarde la pluie sur douze heures : la journée qui reste, ou la
    /// soirée et la nuit.
    pub const HORIZON_PLUIE: usize = 12;
    /// Ressenti sous lequel on conseille un manteau (°C).
    pub const MANTEAU_RESSENTI: f64 = 10.0;
    /// Indice UV à partir duquel on conseille des lunettes de soleil.
    pub const UV_LUNETTES: f64 = 3.0;
    /// … et de la crème solaire.
    pub const UV_CREME: f64 = 6.0;
    /// Température à partir de laquelle on conseille de boire (°C).
    pub const CHALEUR_CONSEIL: f64 = 30.0;
    /// Température sous laquelle routes et trottoirs peuvent geler (°C).
    pub const GEL: f64 = 0.0;
    /// Rafales à partir desquelles un parapluie se retourne (km/h).
    pub const RAFALES_CONSEIL: f64 = 50.0;
}

use seuils::*;

const HEURE_MS: i64 = 3_600_000;

/// Vrai si l'heure est pluvieuse : assez d'eau, ou assez de risque.
pub fn pluvieuse(heure: &HourlySample) -> bool {
    heure.precipitation >= PLUIE_MM || heure.precipitation_probability >= PLUIE_PROBABILITE
}

/* ---------------------------------------------------------------- */
/* La pluie                                                          */
/* ---------------------------------------------------------------- */

/// Ce que la pluie fera dans les heures qui viennent.
#[derive(Debug, Clone, PartialEq)]
pub enum Pluie {
    /// Rien de prévu sur `heures` heures.
    Aucune { heures: usize },
    /// Il pleut maintenant. `fin` : le début de la première heure sèche,
    /// absente si la pluie dure au-delà de l'horizon.
    EnCours { fin: Option<i64> },
    /// La pluie arrive. `probabilite` : le risque le plus fort de l'épisode ;
    /// `cumul` : ce qu'il versera, en millimètres.
    Prevue { debut: i64, probabilite: f64, cumul: f64 },
}

impl Pluie {
    /// Nom stable, pour les journaux et les tests des deux côtés.
    pub fn code(&self) -> &'static str {
        match self {
            Pluie::Aucune { .. } => "aucune",
            Pluie::EnCours { .. } => "enCours",
            Pluie::Prevue { .. } => "prevue",
        }
    }
}

/// La pluie des douze prochaines heures.
pub fn prochaine_pluie(heures: &[HourlySample]) -> Pluie {
    let fenetre = &heures[..heures.len().min(HORIZON_PLUIE)];
    let Some(premiere) = fenetre.first() else {
        return Pluie::Aucune { heures: 0 };
    };

    if pluvieuse(premiere) {
        let fin = fenetre.iter().find(|h| !pluvieuse(h)).map(|h| h.time);
        return Pluie::EnCours { fin };
    }

    let Some(debut) = fenetre.iter().position(pluvieuse) else {
        return Pluie::Aucune { heures: fenetre.len() };
    };
    let episode: Vec<&HourlySample> = fenetre[debut..].iter().take_while(|h| pluvieuse(h)).collect();
    Pluie::Prevue {
        debut: fenetre[debut].time,
        probabilite: episode.iter().map(|h| h.precipitation_probability).fold(0.0, f64::max),
        cumul: arrondi(episode.iter().map(|h| h.precipitation).sum(), 1),
    }
}

/* ---------------------------------------------------------------- */
/* Ce qu'il faut emporter                                            */
/* ---------------------------------------------------------------- */

/// Un conseil pour sortir. L'ordre de l'énumération est celui de l'affichage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Conseil {
    Parapluie,
    Manteau,
    CremeSolaire,
    Lunettes,
    Hydratation,
    Gel,
    Vent,
}

impl Conseil {
    pub fn code(self) -> &'static str {
        match self {
            Conseil::Parapluie => "parapluie",
            Conseil::Manteau => "manteau",
            Conseil::CremeSolaire => "cremeSolaire",
            Conseil::Lunettes => "lunettes",
            Conseil::Hydratation => "hydratation",
            Conseil::Gel => "gel",
            Conseil::Vent => "vent",
        }
    }

    /// Clé de catalogue.
    pub fn key(self) -> String {
        format!("advice.{}", self.code())
    }
}

/// Ce qu'il faut emporter pour les douze prochaines heures.
///
/// Une liste vide est une bonne nouvelle, pas un oubli : l'interface le dit.
pub fn conseils(heures: &[HourlySample]) -> Vec<Conseil> {
    let fenetre = &heures[..heures.len().min(HORIZON_PLUIE)];
    if fenetre.is_empty() {
        return Vec::new();
    }
    let max = |f: fn(&HourlySample) -> f64| fenetre.iter().map(f).fold(f64::NEG_INFINITY, f64::max);
    let min = |f: fn(&HourlySample) -> f64| fenetre.iter().map(f).fold(f64::INFINITY, f64::min);

    let mut liste = Vec::new();
    if fenetre.iter().any(pluvieuse) {
        liste.push(Conseil::Parapluie);
    }
    if min(|h| h.apparent_temperature) <= MANTEAU_RESSENTI {
        liste.push(Conseil::Manteau);
    }
    let uv = max(|h| h.uv_index);
    if uv >= UV_CREME {
        liste.push(Conseil::CremeSolaire);
    } else if uv >= UV_LUNETTES {
        liste.push(Conseil::Lunettes);
    }
    if max(|h| h.temperature) >= CHALEUR_CONSEIL {
        liste.push(Conseil::Hydratation);
    }
    if min(|h| h.temperature) <= GEL {
        liste.push(Conseil::Gel);
    }
    if max(|h| h.wind_gusts) >= RAFALES_CONSEIL {
        liste.push(Conseil::Vent);
    }
    liste
}

/* ---------------------------------------------------------------- */
/* Le soleil                                                         */
/* ---------------------------------------------------------------- */

/// L'échelle de l'Organisation mondiale de la santé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NiveauUv {
    Faible,
    Modere,
    Eleve,
    TresEleve,
    Extreme,
}

impl NiveauUv {
    pub fn code(self) -> &'static str {
        match self {
            NiveauUv::Faible => "faible",
            NiveauUv::Modere => "modere",
            NiveauUv::Eleve => "eleve",
            NiveauUv::TresEleve => "tresEleve",
            NiveauUv::Extreme => "extreme",
        }
    }

    pub fn key(self) -> String {
        format!("uv.{}", self.code())
    }
}

/// Le niveau d'un indice UV. L'indice s'arrondit avant de se classer : 2,6 se
/// lit « 3 » sur tous les bulletins, et doit donc dire « modéré ».
pub fn niveau_uv(indice: f64) -> NiveauUv {
    match arrondi(indice, 0) {
        i if i < 3.0 => NiveauUv::Faible,
        i if i < 6.0 => NiveauUv::Modere,
        i if i < 8.0 => NiveauUv::Eleve,
        i if i < 11.0 => NiveauUv::TresEleve,
        _ => NiveauUv::Extreme,
    }
}

/// L'heure de la série qui contient `instant` — à défaut, la première.
pub fn heure_de(instant: i64, heures: &[HourlySample]) -> Option<&HourlySample> {
    heures
        .iter()
        .rev()
        .find(|h| h.time <= instant && instant < h.time + HEURE_MS)
        .or_else(|| heures.first())
}

/// Arrondi à la manière de JavaScript et de Swift dans ce dépôt :
/// `floor(x + 0,5)`, pour que les trois écritures rendent le même nombre.
fn arrondi(valeur: f64, decimales: u32) -> f64 {
    let facteur = 10f64.powi(decimales as i32);
    (valeur * facteur + 0.5).floor() / facteur
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// 14 h pile, heure de la ville.
    pub(crate) const T0: i64 = 1_790_000_000_000 - 1_790_000_000_000 % HEURE_MS;

    /// Une heure sèche, douce, sans vent ni soleil fort.
    pub(crate) fn heure(decalage: i64) -> HourlySample {
        HourlySample {
            time: T0 + decalage * HEURE_MS,
            weather_code: 3,
            is_day: true,
            precipitation_probability: 10.0,
            temperature: 18.0,
            apparent_temperature: 17.0,
            relative_humidity: 60.0,
            dew_point: 10.0,
            precipitation: 0.0,
            wind_speed: 10.0,
            wind_gusts: 20.0,
            uv_index: 1.0,
        }
    }

    pub(crate) fn serie(n: i64) -> Vec<HourlySample> {
        (0..n).map(heure).collect()
    }

    #[test]
    fn une_heure_est_pluvieuse_par_leau_ou_par_le_risque() {
        let mut h = heure(0);
        assert!(!pluvieuse(&h));
        h.precipitation = 0.1;
        assert!(pluvieuse(&h));
        h.precipitation = 0.0;
        h.precipitation_probability = 50.0;
        assert!(pluvieuse(&h));
        h.precipitation_probability = 49.0;
        assert!(!pluvieuse(&h));
    }

    #[test]
    fn sans_pluie_on_dit_sur_combien_dheures() {
        assert_eq!(prochaine_pluie(&serie(24)), Pluie::Aucune { heures: 12 });
        assert_eq!(prochaine_pluie(&serie(5)), Pluie::Aucune { heures: 5 });
        assert_eq!(prochaine_pluie(&[]), Pluie::Aucune { heures: 0 });
    }

    #[test]
    fn la_pluie_qui_vient_dit_son_heure_son_risque_et_son_cumul() {
        let mut s = serie(24);
        s[3].precipitation = 0.4;
        s[3].precipitation_probability = 60.0;
        s[4].precipitation = 1.2;
        s[4].precipitation_probability = 80.0;
        s[6].precipitation = 3.0; // un second épisode, qui ne compte pas
        assert_eq!(
            prochaine_pluie(&s),
            Pluie::Prevue { debut: T0 + 3 * HEURE_MS, probabilite: 80.0, cumul: 1.6 }
        );
    }

    #[test]
    fn la_pluie_au_dela_de_douze_heures_ne_se_dit_pas() {
        let mut s = serie(24);
        s[12].precipitation = 2.0;
        assert_eq!(prochaine_pluie(&s), Pluie::Aucune { heures: 12 });
    }

    #[test]
    fn il_pleut_et_cela_cesse_a_la_premiere_heure_seche() {
        let mut s = serie(24);
        s[0].precipitation = 0.8;
        s[1].precipitation_probability = 70.0;
        assert_eq!(prochaine_pluie(&s), Pluie::EnCours { fin: Some(T0 + 2 * HEURE_MS) });

        for h in s.iter_mut().take(12) {
            h.precipitation = 1.0;
        }
        assert_eq!(prochaine_pluie(&s), Pluie::EnCours { fin: None });
    }

    #[test]
    fn une_belle_journee_douce_ne_demande_rien() {
        assert!(conseils(&serie(24)).is_empty());
        assert!(conseils(&[]).is_empty());
    }

    #[test]
    fn chaque_conseil_a_sa_raison() {
        let mut s = serie(24);
        s[2].precipitation_probability = 55.0;
        s[5].apparent_temperature = 9.0;
        s[1].uv_index = 6.2;
        s[3].temperature = 31.0;
        s[8].temperature = -1.0;
        s[9].wind_gusts = 55.0;
        assert_eq!(
            conseils(&s),
            vec![
                Conseil::Parapluie,
                Conseil::Manteau,
                Conseil::CremeSolaire,
                Conseil::Hydratation,
                Conseil::Gel,
                Conseil::Vent
            ]
        );
    }

    #[test]
    fn les_lunettes_avant_la_creme_et_jamais_les_deux() {
        let mut s = serie(24);
        s[1].uv_index = 3.0;
        assert_eq!(conseils(&s), vec![Conseil::Lunettes]);
        s[1].uv_index = 6.0;
        assert_eq!(conseils(&s), vec![Conseil::CremeSolaire]);
    }

    #[test]
    fn ce_qui_arrive_apres_douze_heures_ne_change_pas_les_conseils() {
        let mut s = serie(24);
        s[12].temperature = -5.0;
        s[13].precipitation = 4.0;
        assert!(conseils(&s).is_empty());
    }

    #[test]
    fn lechelle_uv_de_loms_arrondit_avant_de_classer() {
        assert_eq!(niveau_uv(0.0), NiveauUv::Faible);
        assert_eq!(niveau_uv(2.4), NiveauUv::Faible);
        assert_eq!(niveau_uv(2.5), NiveauUv::Modere);
        assert_eq!(niveau_uv(5.0), NiveauUv::Modere);
        assert_eq!(niveau_uv(6.0), NiveauUv::Eleve);
        assert_eq!(niveau_uv(8.0), NiveauUv::TresEleve);
        assert_eq!(niveau_uv(10.4), NiveauUv::TresEleve);
        assert_eq!(niveau_uv(11.0), NiveauUv::Extreme);
    }

    #[test]
    fn lheure_de_maintenant_se_trouve_dans_la_serie() {
        let s = serie(4);
        assert_eq!(heure_de(T0 + 2 * HEURE_MS + 600_000, &s).unwrap().time, T0 + 2 * HEURE_MS);
        assert_eq!(heure_de(T0 - 1, &s).unwrap().time, T0);
        assert!(heure_de(0, &[]).is_none());
    }

    #[test]
    fn les_codes_sont_ceux_des_catalogues() {
        assert_eq!(Conseil::CremeSolaire.key(), "advice.cremeSolaire");
        assert_eq!(NiveauUv::TresEleve.key(), "uv.tresEleve");
        assert_eq!(Pluie::EnCours { fin: None }.code(), "enCours");
    }
}
