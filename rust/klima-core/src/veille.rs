//! Le guetteur : la demi-heure en cours et les deux heures qui viennent, au
//! quart d'heure près.
//!
//! Miroir Swift : `ios/Kliima/Models/Veille.swift`, avec les mêmes cas de
//! test. Toute règle ajoutée d'un côté se porte de l'autre.
//!
//! La carte de la pluie répond heure par heure sur douze heures ; le guetteur
//! répond à la question d'après, celle qu'on pose la main sur la poignée :
//! est-ce que ça tombe **maintenant**, et pour combien de temps. Il lit la
//! prévision au quart d'heure d'Open-Meteo (`minutely_15`), la relit tous les
//! quarts d'heure, et dit deux choses : la demi-heure en cours — le quart
//! entamé et le suivant —, puis les deux heures.
//!
//! Comme le reste du domaine, il ne fabrique pas de phrases : il rend des
//! états, des heures et des quantités, que l'interface dit dans la langue de
//! la personne.
//!
//! Les horodatages sont ceux de la ville, comme partout : des millisecondes à
//! l'heure locale, sans décalage.

/// Les seuils, au même endroit que leur raison.
pub mod seuils {
    /// Un quart d'heure, en millisecondes.
    pub const QUART_MS: i64 = 900_000;
    /// La demi-heure en cours : le quart entamé et le suivant.
    pub const IMMEDIAT_QUARTS: usize = 2;
    /// Deux heures : huit quarts.
    pub const HORIZON_QUARTS: usize = 8;
    /// Un quart est mouillé à partir de 0,1 mm : la plus petite quantité que
    /// le modèle rapporte.
    pub const PLUIE_QUART_MM: f64 = 0.1;
    /// Intensités, en millimètres par heure (un quart compte quatre fois) :
    /// faible en deçà de 2,5, modérée jusqu'à 7,6, forte au-delà — les
    /// classes de l'Organisation météorologique mondiale.
    pub const MODEREE_MM_H: f64 = 2.5;
    pub const FORTE_MM_H: f64 = 7.6;
    /// Rafales que le guetteur signale (km/h) : celles qui retournent un
    /// parapluie, le même seuil que le conseil de la ville.
    pub const RAFALES: f64 = crate::ville::seuils::RAFALES_CONSEIL;
}

use seuils::*;

/// Un quart d'heure de prévision.
#[derive(Debug, Clone, PartialEq)]
pub struct QuartSample {
    /// Début du quart, en millisecondes à l'heure de la ville.
    pub time: i64,
    /// Précipitations sur le quart (mm).
    pub precipitation: f64,
    pub weather_code: u16,
    pub temperature: f64,
    /// Température ressentie (°C).
    pub apparent_temperature: f64,
    /// Rafales (km/h).
    pub wind_gusts: f64,
    pub is_day: bool,
}

/// Ce qui tombe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nature {
    Pluie,
    Neige,
    Orage,
}

/// Avec quelle force.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intensite {
    Faible,
    Moderee,
    Forte,
}

impl Intensite {
    pub fn code(self) -> &'static str {
        match self {
            Intensite::Faible => "faible",
            Intensite::Moderee => "moderee",
            Intensite::Forte => "forte",
        }
    }
}

/// Ce qui tombe, et avec quelle force : une seule clé de catalogue, parce
/// que l'accord se fait dans la langue — « forte pluie », « orage ».
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Precipitation {
    pub nature: Nature,
    pub intensite: Intensite,
}

impl Precipitation {
    /// Clé de catalogue : `veille.kind.pluie.faible`, … ; l'orage n'a qu'une
    /// clé, sa force se passe de qualificatif.
    pub fn key(self) -> String {
        match self.nature {
            Nature::Orage => "veille.kind.orage".to_owned(),
            Nature::Pluie => format!("veille.kind.pluie.{}", self.intensite.code()),
            Nature::Neige => format!("veille.kind.neige.{}", self.intensite.code()),
        }
    }
}

/// Vrai si le quart est mouillé.
pub fn mouille(quart: &QuartSample) -> bool {
    quart.precipitation >= PLUIE_QUART_MM
}

/// L'intensité d'un quart, d'après son débit horaire.
pub fn intensite(quart: &QuartSample) -> Intensite {
    let debit = quart.precipitation * 4.0;
    if debit >= FORTE_MM_H {
        Intensite::Forte
    } else if debit >= MODEREE_MM_H {
        Intensite::Moderee
    } else {
        Intensite::Faible
    }
}

/// La nature de ce qui tombe, d'après le code météo du quart.
pub fn nature(quart: &QuartSample) -> Nature {
    match quart.weather_code {
        95 | 96 | 99 => Nature::Orage,
        71..=77 | 85 | 86 => Nature::Neige,
        _ => Nature::Pluie,
    }
}

/// Ce qui tombe sur une suite de quarts : la nature la plus marquante (l'orage
/// l'emporte sur la neige, la neige sur la pluie) et l'intensité la plus forte.
fn precipitation(quarts: &[QuartSample]) -> Precipitation {
    let mouilles = quarts.iter().filter(|q| mouille(q));
    let nature = mouilles
        .clone()
        .map(nature)
        .max_by_key(|n| match n {
            Nature::Pluie => 0,
            Nature::Neige => 1,
            Nature::Orage => 2,
        })
        .unwrap_or(Nature::Pluie);
    let intensite = mouilles.map(intensite).max().unwrap_or(Intensite::Faible);
    Precipitation { nature, intensite }
}

/// La demi-heure en cours.
#[derive(Debug, Clone, PartialEq)]
pub enum Immediat {
    /// Rien ne tombe d'ici une demi-heure.
    Sec,
    /// Ça commence au quart suivant.
    Commence { debut: i64, precipitation: Precipitation },
    /// Ça tombe, et toute la demi-heure.
    Continue { precipitation: Precipitation },
    /// Ça tombe, et ça s'arrête au quart suivant.
    Cesse { fin: i64 },
}

impl Immediat {
    pub fn code(&self) -> &'static str {
        match self {
            Immediat::Sec => "sec",
            Immediat::Commence { .. } => "commence",
            Immediat::Continue { .. } => "continue",
            Immediat::Cesse { .. } => "cesse",
        }
    }
}

/// Les deux heures.
#[derive(Debug, Clone, PartialEq)]
pub enum Suite {
    /// Rien d'ici la fin de la fenêtre.
    Sec,
    /// Une averse qui n'a pas commencé. `fin` : le premier quart sec après
    /// elle, absent si elle dure au-delà de la fenêtre.
    Episode { debut: i64, fin: Option<i64>, precipitation: Precipitation, cumul: f64 },
    /// Ça tombe de bout en bout.
    Persiste { precipitation: Precipitation, cumul: f64 },
    /// Ça tombe maintenant, et ça s'arrête à `fin` ; `reprise` si ça
    /// recommence avant la fin de la fenêtre.
    Accalmie { fin: i64, reprise: Option<i64> },
}

impl Suite {
    pub fn code(&self) -> &'static str {
        match self {
            Suite::Sec => "sec",
            Suite::Episode { .. } => "episode",
            Suite::Persiste { .. } => "persiste",
            Suite::Accalmie { .. } => "accalmie",
        }
    }
}

/// Ce que le guetteur a vu.
#[derive(Debug, Clone, PartialEq)]
pub struct Veille {
    /// Les quarts regardés, à partir de celui qui est entamé.
    pub quarts: Vec<QuartSample>,
    pub immediat: Immediat,
    pub suite: Suite,
    /// La fin de la fenêtre : le début du quart qui suit le dernier.
    pub fin_fenetre: i64,
    /// Les plus fortes rafales, et leur quart, si elles passent le seuil.
    pub rafales: Option<(f64, i64)>,
}

/// Le guetteur, à `maintenant` (heure de la ville).
///
/// La série commence au quart entamé : celui dont le début est passé et la
/// fin pas encore. `None` si la série ne couvre pas au moins la demi-heure —
/// mieux vaut se taire que dire « sec » sur un trou.
pub fn veille(serie: &[QuartSample], maintenant: i64) -> Option<Veille> {
    let debut = serie.iter().position(|q| q.time + QUART_MS > maintenant)?;
    // Le premier quart retenu doit bien être celui en cours : une série qui
    // reprend une heure plus tard n'en tient pas lieu.
    if serie[debut].time > maintenant {
        return None;
    }
    let quarts: Vec<QuartSample> =
        serie[debut..].iter().take(HORIZON_QUARTS).cloned().collect();
    if quarts.len() < IMMEDIAT_QUARTS {
        return None;
    }

    let mouilles: Vec<bool> = quarts.iter().map(mouille).collect();
    let immediat = match (mouilles[0], mouilles[1]) {
        (false, false) => Immediat::Sec,
        (false, true) => Immediat::Commence {
            debut: quarts[1].time,
            precipitation: precipitation(&quarts[1..2]),
        },
        (true, true) => Immediat::Continue { precipitation: precipitation(&quarts[..2]) },
        (true, false) => Immediat::Cesse { fin: quarts[1].time },
    };

    let fin_fenetre = quarts[quarts.len() - 1].time + QUART_MS;
    let suite = match mouilles.iter().position(|m| *m) {
        None => Suite::Sec,
        Some(0) => match mouilles.iter().position(|m| !*m) {
            None => Suite::Persiste {
                precipitation: precipitation(&quarts),
                cumul: cumul(&quarts),
            },
            Some(sec) => Suite::Accalmie {
                fin: quarts[sec].time,
                reprise: mouilles[sec..].iter().position(|m| *m).map(|i| quarts[sec + i].time),
            },
        },
        Some(premier) => {
            let duree = mouilles[premier..].iter().take_while(|m| **m).count();
            let episode = &quarts[premier..premier + duree];
            Suite::Episode {
                debut: quarts[premier].time,
                fin: quarts.get(premier + duree).map(|q| q.time),
                precipitation: precipitation(episode),
                cumul: cumul(episode),
            }
        }
    };

    // À égalité, le premier quart : « vers 16 h 45 » plutôt que le dernier.
    let rafales = quarts
        .iter()
        .fold(None::<&QuartSample>, |plus, q| match plus {
            Some(p) if p.wind_gusts >= q.wind_gusts => Some(p),
            _ => Some(q),
        })
        .filter(|q| q.wind_gusts >= RAFALES)
        .map(|q| (q.wind_gusts, q.time));

    Some(Veille { quarts, immediat, suite, fin_fenetre, rafales })
}

/// La série, corrigée de ce qu'un aéroport proche voit tomber (`ciel`).
///
/// Les modèles ratent les cellules d'orage : à Bordeaux, Mérignac voyait un
/// `+TSRA` pendant que tous annonçaient zéro. Ce qu'on voit tomber mouille
/// donc les quarts secs, à la force observée, aussi longtemps que le bulletin
/// le permet (`CielObserve::tombe_jusqu_a`) : le quart en cours toujours, la
/// demi-heure jusqu'au bulletin suivant, deux heures si les prévisionnistes
/// ne prévoient pas de changement. Au-delà, la prévision reprend la parole —
/// et la fin n'est jamais inventée plus tôt que le bulletin. Un ciel sec
/// observé ne retire rien.
pub fn observer(serie: &[QuartSample], maintenant: i64, ciel: Option<&crate::ciel::CielObserve>) -> Vec<QuartSample> {
    let mut serie = serie.to_vec();
    let Some(ciel) = ciel else { return serie };
    let (Some(tombe), Some(jusqu_a)) = (ciel.tombe, ciel.tombe_jusqu_a()) else {
        return serie;
    };
    if tombe.code < crate::fusion::seuils::CODE_MOUILLE {
        return serie;
    }
    // Ce qu'on voit couvre au moins la demi-heure en cours — on relit à
    // chaque quart, et un bulletin part toutes les demi-heures —, sauf si
    // les prévisionnistes en annoncent la fin.
    let fin_annoncee = matches!(ciel.tendance, crate::ciel::Tendance::Changement { passager: false, sec: true, .. });
    let demi_heure = maintenant.div_euclid(QUART_MS) * QUART_MS + IMMEDIAT_QUARTS as i64 * QUART_MS;
    let jusqu_a = if fin_annoncee { jusqu_a } else { jusqu_a.max(demi_heure) };
    for quart in serie.iter_mut() {
        let fin = quart.time + QUART_MS;
        let en_cours = quart.time <= maintenant && maintenant < fin;
        let couvert = fin > maintenant && quart.time < jusqu_a;
        if (en_cours || couvert) && !mouille(quart) {
            quart.precipitation = crate::ciel::quart_observe_mm(tombe.intensite);
            quart.weather_code = tombe.code;
        }
    }
    serie
}

/// Vrai quand l'aéroport voit tomber ce que la prévision du quart en cours ne
/// voit pas : les modèles ont raté la cellule, et la fin qu'ils donneraient
/// n'en est pas une. L'interface le dit plutôt que d'inventer une heure.
pub fn aveugle(serie: &[QuartSample], maintenant: i64, ciel: Option<&crate::ciel::CielObserve>) -> bool {
    let vu = ciel.and_then(|c| c.tombe).is_some_and(|t| t.code >= crate::fusion::seuils::CODE_MOUILLE);
    vu && serie
        .iter()
        .find(|q| q.time <= maintenant && maintenant < q.time + QUART_MS)
        .is_some_and(|q| !mouille(q))
}

/// Quand relire : une minute après le début du quart qui suit la lecture —
/// le temps que la série du nouveau quart soit publiée. Les deux horloges
/// (lecture et retour) sont les mêmes, quelles qu'elles soient.
pub fn prochaine_lecture(lu_a: i64) -> i64 {
    (lu_a.div_euclid(QUART_MS) + 1) * QUART_MS + 60_000
}

/// Ce que versent des quarts, arrondi au dixième de millimètre.
fn cumul(quarts: &[QuartSample]) -> f64 {
    (quarts.iter().map(|q| q.precipitation).sum::<f64>() * 10.0).round() / 10.0
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    /// 12 mai 2026, 16 h 00, heure de la ville.
    const SEIZE_H: i64 = 1_778_601_600_000;

    fn serie(pluie: &[f64]) -> Vec<QuartSample> {
        pluie
            .iter()
            .enumerate()
            .map(|(i, mm)| QuartSample {
                time: SEIZE_H + i as i64 * QUART_MS,
                precipitation: *mm,
                weather_code: if *mm > 0.0 { 61 } else { 3 },
                temperature: 16.0 - i as f64 * 0.25,
                apparent_temperature: 15.0,
                wind_gusts: 20.0,
                is_day: true,
            })
            .collect()
    }

    /// Il est 16 h 07 : le quart entamé est celui de 16 h.
    const MAINTENANT: i64 = SEIZE_H + 7 * 60_000;

    #[test]
    fn une_pluie_observee_mouille_les_quarts_que_le_bulletin_couvre() {
        use crate::ciel::{CielObserve, Tendance, Tombe};
        // Le bulletin de 16 h, lu à 16 h 07 : la pluie modérée vaut jusqu'au
        // suivant, à 16 h 30 — le quart en cours et le suivant.
        let vu = |tombe: Option<Tombe>, tendance: Tendance| CielObserve {
            station: "LFBD".into(),
            nom: "Bordeaux/Merignac".into(),
            distance_km: 8.0,
            time: SEIZE_H,
            tombe,
            tendance,
        };
        let pluie = Some(Tombe { code: 63, intensite: Intensite::Moderee });
        let sec = serie(&[0.0; 10]);

        let lu = observer(&sec, MAINTENANT, Some(&vu(pluie, Tendance::Inconnue)));
        assert_eq!((lu[0].precipitation, lu[0].weather_code), (1.0, 63));
        assert_eq!(lu[1].precipitation, 1.0);
        assert_eq!(lu[2], sec[2], "au-delà du bulletin, la prévision");
        let v = veille(&lu, MAINTENANT).unwrap();
        assert_eq!(
            v.immediat,
            Immediat::Continue { precipitation: Precipitation { nature: Nature::Pluie, intensite: Intensite::Moderee } }
        );

        // NOSIG : rien ne changera d'ici deux heures, la pluie dure.
        let lu = observer(&sec, MAINTENANT, Some(&vu(pluie, Tendance::Stable)));
        assert!(lu[..8].iter().all(mouille));
        assert!(!mouille(&lu[8]));

        // Un orage fort compte pour une forte pluie.
        let orage = Some(Tombe { code: 95, intensite: Intensite::Forte });
        let lu = observer(&sec, MAINTENANT, Some(&vu(orage, Tendance::Inconnue)));
        assert_eq!(intensite(&lu[0]), Intensite::Forte);
        assert_eq!(nature(&lu[0]), Nature::Orage);

        // Un bulletin de 15 h 45 lu à 16 h 07 couvre encore la demi-heure.
        let ancien = CielObserve { time: SEIZE_H - QUART_MS, ..vu(pluie, Tendance::Inconnue) };
        let lu = observer(&sec, MAINTENANT, Some(&ancien));
        assert!(mouille(&lu[0]) && mouille(&lu[1]) && !mouille(&lu[2]));
        assert!(aveugle(&sec, MAINTENANT, Some(&ancien)), "les modèles n'ont rien vu");
        assert!(!aveugle(&serie(&[0.6]), MAINTENANT, Some(&ancien)));
        assert!(!aveugle(&sec, MAINTENANT, None));

        // BECMG NSW : la fin est annoncée, le quart en cours seulement.
        let fin = Tendance::Changement { passager: false, tombe: None, sec: true };
        let lu = observer(&sec, MAINTENANT, Some(&vu(pluie, fin)));
        assert!(mouille(&lu[0]) && !mouille(&lu[1]));

        // Un quart déjà mouillé garde sa prévision ; rien d'observé, rien ne change.
        let mouillee = serie(&[0.6, 0.6]);
        assert_eq!(observer(&mouillee, MAINTENANT, Some(&vu(orage, Tendance::Inconnue)))[0], mouillee[0]);
        assert_eq!(observer(&sec, MAINTENANT, None), sec);
        assert_eq!(observer(&sec, MAINTENANT, Some(&vu(None, Tendance::Stable))), sec);
        let brouillard = Some(Tombe { code: 45, intensite: Intensite::Moderee });
        assert_eq!(observer(&sec, MAINTENANT, Some(&vu(brouillard, Tendance::Stable))), sec);
    }

    #[test]
    fn rien_ne_tombe() {
        let v = veille(&serie(&[0.0; 10]), MAINTENANT).unwrap();
        assert_eq!(v.immediat, Immediat::Sec);
        assert_eq!(v.suite, Suite::Sec);
        assert_eq!(v.quarts.len(), HORIZON_QUARTS);
        assert_eq!(v.quarts[0].time, SEIZE_H);
        assert_eq!(v.fin_fenetre, SEIZE_H + 8 * QUART_MS);
        assert_eq!(v.rafales, None);
    }

    #[test]
    fn la_serie_repart_du_quart_entame() {
        // À 16 h 20, le quart entamé est celui de 16 h 15.
        let v = veille(&serie(&[0.0; 10]), SEIZE_H + 20 * 60_000).unwrap();
        assert_eq!(v.quarts[0].time, SEIZE_H + QUART_MS);
    }

    #[test]
    fn ca_commence_au_quart_suivant() {
        let v = veille(&serie(&[0.0, 0.3, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0]), MAINTENANT).unwrap();
        assert_eq!(
            v.immediat,
            Immediat::Commence {
                debut: SEIZE_H + QUART_MS,
                precipitation: Precipitation { nature: Nature::Pluie, intensite: Intensite::Faible },
            }
        );
        assert_eq!(
            v.suite,
            Suite::Episode {
                debut: SEIZE_H + QUART_MS,
                fin: Some(SEIZE_H + 3 * QUART_MS),
                precipitation: Precipitation { nature: Nature::Pluie, intensite: Intensite::Faible },
                cumul: 0.8,
            }
        );
    }

    #[test]
    fn ca_continue_et_l_intensite_est_la_plus_forte() {
        // 0,8 mm en un quart : 3,2 mm/h, modérée.
        let v = veille(&serie(&[0.2, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]), MAINTENANT).unwrap();
        assert_eq!(
            v.immediat,
            Immediat::Continue {
                precipitation: Precipitation { nature: Nature::Pluie, intensite: Intensite::Moderee },
            }
        );
        assert_eq!(v.suite, Suite::Accalmie { fin: SEIZE_H + 2 * QUART_MS, reprise: None });
    }

    #[test]
    fn ca_cesse_puis_reprend() {
        let v = veille(&serie(&[0.4, 0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0]), MAINTENANT).unwrap();
        assert_eq!(v.immediat, Immediat::Cesse { fin: SEIZE_H + QUART_MS });
        assert_eq!(
            v.suite,
            Suite::Accalmie { fin: SEIZE_H + QUART_MS, reprise: Some(SEIZE_H + 4 * QUART_MS) }
        );
    }

    #[test]
    fn une_pluie_qui_ne_s_arrete_pas() {
        let v = veille(&serie(&[2.0; 8]), MAINTENANT).unwrap();
        assert_eq!(
            v.suite,
            Suite::Persiste {
                precipitation: Precipitation { nature: Nature::Pluie, intensite: Intensite::Forte },
                cumul: 16.0,
            }
        );
    }

    #[test]
    fn une_averse_qui_deborde_de_la_fenetre() {
        let v = veille(&serie(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.3, 0.3, 0.3]), MAINTENANT).unwrap();
        assert_eq!(v.immediat, Immediat::Sec);
        match v.suite {
            Suite::Episode { debut, fin, cumul, .. } => {
                assert_eq!(debut, SEIZE_H + 6 * QUART_MS);
                assert_eq!(fin, None, "le quart sec suivant est hors de la fenêtre");
                assert_eq!(cumul, 0.6);
            }
            autre => panic!("{autre:?}"),
        }
    }

    #[test]
    fn un_dixieme_ne_mouille_pas_en_dessous() {
        let v = veille(&serie(&[0.09, 0.09, 0.0, 0.0]), MAINTENANT).unwrap();
        assert_eq!(v.immediat, Immediat::Sec);
        assert_eq!(v.quarts.len(), 4, "une série courte se lit telle quelle");
        assert_eq!(v.fin_fenetre, SEIZE_H + 4 * QUART_MS);
    }

    #[test]
    fn les_bornes_d_intensite() {
        let quart = |mm: f64| QuartSample { precipitation: mm, ..serie(&[0.0])[0].clone() };
        assert_eq!(intensite(&quart(0.6)), Intensite::Faible); // 2,4 mm/h
        assert_eq!(intensite(&quart(0.625)), Intensite::Moderee); // 2,5 mm/h
        assert_eq!(intensite(&quart(1.9)), Intensite::Forte); // 7,6 mm/h
    }

    #[test]
    fn la_neige_et_l_orage_se_disent() {
        let mut quarts = serie(&[0.3, 0.3, 0.3, 0.0]);
        quarts[0].weather_code = 73;
        quarts[1].weather_code = 73;
        let v = veille(&quarts, MAINTENANT).unwrap();
        assert_eq!(
            v.immediat,
            Immediat::Continue {
                precipitation: Precipitation { nature: Nature::Neige, intensite: Intensite::Faible },
            }
        );
        // Un quart d'orage suffit à faire de l'épisode un orage.
        quarts[2].weather_code = 95;
        match veille(&quarts, MAINTENANT).unwrap().suite {
            Suite::Accalmie { .. } => {}
            autre => panic!("{autre:?}"),
        }
        assert_eq!(precipitation(&quarts[..3]).nature, Nature::Orage);
        assert_eq!(precipitation(&quarts[..3]).key(), "veille.kind.orage");
        assert_eq!(
            Precipitation { nature: Nature::Neige, intensite: Intensite::Forte }.key(),
            "veille.kind.neige.forte"
        );
    }

    #[test]
    fn les_rafales_qui_retournent_un_parapluie() {
        let mut quarts = serie(&[0.0; 8]);
        quarts[3].wind_gusts = 62.0;
        quarts[5].wind_gusts = 55.0;
        quarts[6].wind_gusts = 62.0;
        let v = veille(&quarts, MAINTENANT).unwrap();
        // À égalité, le premier quart.
        assert_eq!(v.rafales, Some((62.0, SEIZE_H + 3 * QUART_MS)));
        // Sous le seuil, rien à signaler.
        let calme = serie(&[0.0; 8]);
        assert_eq!(veille(&calme, MAINTENANT).unwrap().rafales, None);
    }

    #[test]
    fn on_relit_une_minute_apres_le_quart_suivant() {
        assert_eq!(prochaine_lecture(MAINTENANT), SEIZE_H + QUART_MS + 60_000);
        // Lu pile au début d'un quart : on attend le suivant.
        assert_eq!(prochaine_lecture(SEIZE_H), SEIZE_H + QUART_MS + 60_000);
        assert_eq!(prochaine_lecture(SEIZE_H + QUART_MS - 1), SEIZE_H + QUART_MS + 60_000);
    }

    #[test]
    fn un_trou_dans_la_serie_fait_taire_le_guetteur() {
        // La série commence dans une heure : pas de quart en cours.
        let tard: Vec<_> = serie(&[0.0; 8])
            .into_iter()
            .map(|q| QuartSample { time: q.time + 3_600_000, ..q })
            .collect();
        assert_eq!(veille(&tard, MAINTENANT), None);
        // La série est finie : rien après maintenant.
        assert_eq!(veille(&serie(&[0.0; 8]), SEIZE_H + 3 * 3_600_000), None);
        // Un seul quart : pas de quoi couvrir la demi-heure.
        assert_eq!(veille(&serie(&[0.0]), MAINTENANT), None);
    }
}
