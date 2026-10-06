//! La prévision recoupée : tous les fournisseurs, une seule météo.
//!
//! Miroir Swift : `ios/Kliima/Models/Fusion.swift`, avec les mêmes cas de
//! test. Toute règle ajoutée d'un côté se porte de l'autre.
//!
//! Jusqu'ici, la prévision affichée venait d'un seul modèle — le « meilleur »
//! qu'Open-Meteo choisit pour le point —, et les autres sources ne servaient
//! qu'à dire si elles étaient d'accord. Ici, elles **font** la prévision :
//! pour chaque heure, chaque source vote, et Klima retient
//!
//! - la **médiane** pour ce qui se mesure — température, ressenti, pluie,
//!   vent, rafales. Une médiane ignore le modèle qui s'égare : un écart de six
//!   degrés chez un seul ne déplace rien, alors qu'il tirerait une moyenne ;
//! - pour le **risque de pluie**, la part des sources qui annoncent de la
//!   pluie, mêlée à moitié au risque que les modèles publient eux-mêmes. Neuf
//!   modèles indépendants dont six mouillent, c'est une chance sur deux et
//!   demie plus parlante qu'un pourcentage tiré d'un seul ;
//! - pour le **temps qu'il fait**, la majorité : si la plupart des sources
//!   mouillent, le code le plus cité parmi elles ; sinon, le plus cité parmi
//!   les sèches. À égalité, le plus marqué — mieux vaut annoncer l'averse que
//!   la taire.
//!
//! Ce qu'une seule source fournit — humidité, point de rosée, UV, jour ou
//! nuit, lever et coucher — reste celui de la prévision de base. Une heure où
//! aucune source n'a rien dit garde ses valeurs de base : on ne remplace pas
//! une prévision par un silence.

use std::collections::HashMap;

use crate::meteo::{CurrentSample, DailySample, HourlySample};

/// Les seuils, au même endroit que leur raison.
pub mod seuils {
    /// Une source mouille une heure à partir de ce cumul (mm) : le seuil de
    /// la pluie de la ville.
    pub const PLUIE_MM: f64 = crate::ville::seuils::PLUIE_MM;
    /// Part du risque publié par les modèles dans le risque recoupé ; le reste
    /// vient de la part des sources qui mouillent.
    pub const POIDS_RISQUE_PUBLIE: f64 = 0.5;
    /// Les codes météo à partir desquels il tombe quelque chose.
    pub const CODE_MOUILLE: u16 = 51;
}

use seuils::*;

/// Ce qu'une source annonce pour une heure. Tout est facultatif : un modèle
/// sans rafales n'en a pas de nulles, il n'en a pas.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HeureSource {
    /// Millisecondes à l'heure de la ville — la même horloge que les séries.
    pub time: i64,
    pub temperature: Option<f64>,
    pub ressenti: Option<f64>,
    pub precipitation: Option<f64>,
    pub probabilite: Option<f64>,
    pub vent: Option<f64>,
    pub rafales: Option<f64>,
    pub code: Option<u16>,
}

/// Ce qu'une source annonce pour une journée.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JourSource {
    pub date: i64,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub cumul: Option<f64>,
    pub probabilite: Option<f64>,
    pub rafales: Option<f64>,
    pub code: Option<u16>,
}

/// Une source et sa série.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SerieSource {
    pub source_id: String,
    pub heures: Vec<HeureSource>,
    pub jours: Vec<JourSource>,
}

/// La médiane ; pour un nombre pair de valeurs, la moyenne des deux du
/// milieu. `None` sans valeur.
pub fn mediane(valeurs: &[f64]) -> Option<f64> {
    let mut v: Vec<f64> = valeurs.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let milieu = v.len() / 2;
    Some(if v.len() % 2 == 0 { (v[milieu - 1] + v[milieu]) / 2.0 } else { v[milieu] })
}

/// Le risque recoupé (%) : la part des sources qui mouillent, mêlée au risque
/// publié quand il y en a un. `None` si aucune source n'a dit sa pluie.
pub fn risque(cumuls: &[f64], publies: &[f64]) -> Option<f64> {
    if cumuls.is_empty() {
        return None;
    }
    let part = cumuls.iter().filter(|c| **c >= PLUIE_MM).count() as f64 / cumuls.len() as f64 * 100.0;
    let melange = match mediane(publies) {
        Some(publie) => POIDS_RISQUE_PUBLIE * publie + (1.0 - POIDS_RISQUE_PUBLIE) * part,
        None => part,
    };
    Some(melange.round())
}

/// Le code retenu : la majorité mouillée ou sèche, puis le code le plus cité
/// de ce camp ; à égalité, le plus marqué. `None` si personne n'a de code.
pub fn code_majoritaire(codes: &[u16], cumuls: &[f64]) -> Option<u16> {
    if codes.is_empty() {
        return None;
    }
    let mouilles = if cumuls.is_empty() {
        codes.iter().filter(|c| **c >= CODE_MOUILLE).count() * 2 >= codes.len()
    } else {
        cumuls.iter().filter(|c| **c >= PLUIE_MM).count() * 2 >= cumuls.len()
    };
    let camp: Vec<u16> = codes.iter().copied().filter(|c| (*c >= CODE_MOUILLE) == mouilles).collect();
    // Les sources mouillent mais aucune n'a de code de pluie (ou l'inverse) :
    // on prend tous les codes plutôt que d'inventer.
    let candidats = if camp.is_empty() { codes.to_vec() } else { camp };
    let mut comptes: HashMap<u16, usize> = HashMap::new();
    for code in &candidats {
        *comptes.entry(*code).or_default() += 1;
    }
    comptes.into_iter().max_by(|(ca, na), (cb, nb)| na.cmp(nb).then(ca.cmp(cb))).map(|(c, _)| c)
}

/// Ce que la fusion a produit.
#[derive(Debug, Clone, PartialEq)]
pub struct Recoupement {
    pub heures: Vec<HourlySample>,
    pub jours: Vec<DailySample>,
    pub courant: CurrentSample,
    /// Les sources qui ont voté, dans l'ordre où on les a reçues.
    pub sources: Vec<String>,
}

/// Recoupe la prévision de base avec toutes les séries reçues.
///
/// `observation` : la température mesurée par une station proche, s'il y en
/// a une. Elle vote pour l'instant présent, et seulement pour lui — une
/// station dit ce qu'il fait, pas ce qui vient.
pub fn recouper(
    base_heures: &[HourlySample],
    base_jours: &[DailySample],
    base_courant: &CurrentSample,
    series: &[SerieSource],
    observation: Option<f64>,
) -> Recoupement {
    let index: Vec<HashMap<i64, &HeureSource>> =
        series.iter().map(|s| s.heures.iter().map(|h| (h.time, h)).collect()).collect();
    let index_jours: Vec<HashMap<i64, &JourSource>> =
        series.iter().map(|s| s.jours.iter().map(|j| (j.date, j)).collect()).collect();

    let heures: Vec<HourlySample> = base_heures
        .iter()
        .map(|base| {
            let voix: Vec<&HeureSource> = index.iter().filter_map(|i| i.get(&base.time).copied()).collect();
            fusionner_heure(base, &voix)
        })
        .collect();

    let jours: Vec<DailySample> = base_jours
        .iter()
        .map(|base| {
            let voix: Vec<&JourSource> = index_jours.iter().filter_map(|i| i.get(&base.date).copied()).collect();
            fusionner_jour(base, &voix)
        })
        .collect();

    let courant = fusionner_courant(base_courant, base_heures.first(), heures.first(), &index, observation);

    let sources = series
        .iter()
        .filter(|s| s.heures.iter().any(|h| h.temperature.is_some()))
        .map(|s| s.source_id.clone())
        .collect();

    Recoupement { heures, jours, courant, sources }
}

fn valeurs<T>(voix: &[&T], champ: impl Fn(&T) -> Option<f64>) -> Vec<f64> {
    voix.iter().filter_map(|v| champ(v)).filter(|x| x.is_finite()).collect()
}

fn fusionner_heure(base: &HourlySample, voix: &[&HeureSource]) -> HourlySample {
    if voix.is_empty() {
        return base.clone();
    }
    let temperature = mediane(&valeurs(voix, |v| v.temperature)).unwrap_or(base.temperature);
    // Sans ressenti publié, celui de base suit la température recoupée.
    let ressenti = mediane(&valeurs(voix, |v| v.ressenti))
        .unwrap_or(base.apparent_temperature + (temperature - base.temperature));
    let cumuls = valeurs(voix, |v| v.precipitation);
    let publies = valeurs(voix, |v| v.probabilite);
    let codes: Vec<u16> = voix.iter().filter_map(|v| v.code).collect();

    HourlySample {
        temperature,
        apparent_temperature: ressenti,
        precipitation: mediane(&cumuls).unwrap_or(base.precipitation),
        precipitation_probability: risque(&cumuls, &publies).unwrap_or(base.precipitation_probability),
        wind_speed: mediane(&valeurs(voix, |v| v.vent)).unwrap_or(base.wind_speed),
        wind_gusts: mediane(&valeurs(voix, |v| v.rafales)).unwrap_or(base.wind_gusts),
        weather_code: code_majoritaire(&codes, &cumuls).unwrap_or(base.weather_code),
        ..base.clone()
    }
}

fn fusionner_jour(base: &DailySample, voix: &[&JourSource]) -> DailySample {
    if voix.is_empty() {
        return base.clone();
    }
    let cumuls = valeurs(voix, |v| v.cumul);
    let publies = valeurs(voix, |v| v.probabilite);
    let codes: Vec<u16> = voix.iter().filter_map(|v| v.code).collect();
    DailySample {
        temperature_min: mediane(&valeurs(voix, |v| v.minimum)).unwrap_or(base.temperature_min),
        temperature_max: mediane(&valeurs(voix, |v| v.maximum)).unwrap_or(base.temperature_max),
        precipitation_sum: mediane(&cumuls).unwrap_or(base.precipitation_sum),
        precipitation_probability_max: risque(&cumuls, &publies).unwrap_or(base.precipitation_probability_max),
        wind_gusts_max: mediane(&valeurs(voix, |v| v.rafales)).unwrap_or(base.wind_gusts_max),
        weather_code: code_majoritaire(&codes, &cumuls).unwrap_or(base.weather_code),
        ..base.clone()
    }
}

/// L'instant présent : la base décalée d'autant que l'heure en cours l'a été
/// par la fusion — puis, s'il y a une station, la médiane de ce chiffre et de
/// la mesure. Le temps qu'il fait suit l'heure recoupée.
fn fusionner_courant(
    base: &CurrentSample,
    base_heure: Option<&HourlySample>,
    heure: Option<&HourlySample>,
    index: &[HashMap<i64, &HeureSource>],
    observation: Option<f64>,
) -> CurrentSample {
    let (Some(base_heure), Some(heure)) = (base_heure, heure) else {
        return base.clone();
    };
    let a_vote = index.iter().any(|i| i.get(&heure.time).is_some_and(|h| h.temperature.is_some()));
    if !a_vote && observation.is_none() {
        return base.clone();
    }
    let decalage = heure.temperature - base_heure.temperature;
    let prevue = base.temperature + decalage;
    let temperature = match observation.filter(|o| o.is_finite()) {
        Some(mesure) => mediane(&[prevue, mesure]).unwrap_or(prevue),
        None => prevue,
    };
    let vent = base.wind_speed + (heure.wind_speed - base_heure.wind_speed);
    let rafales = base.wind_gusts + (heure.wind_gusts - base_heure.wind_gusts);
    CurrentSample {
        temperature,
        apparent_temperature: base.apparent_temperature + (temperature - base.temperature),
        weather_code: if a_vote { heure.weather_code } else { base.weather_code },
        wind_speed: vent.max(0.0),
        wind_gusts: rafales.max(vent.max(0.0)),
        ..base.clone()
    }
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    const MIDI: i64 = 1_778_587_200_000;
    const HEURE: i64 = 3_600_000;

    fn base(decalage: i64) -> HourlySample {
        HourlySample {
            time: MIDI + decalage * HEURE,
            weather_code: 1,
            is_day: true,
            precipitation_probability: 10.0,
            temperature: 18.0,
            apparent_temperature: 17.0,
            relative_humidity: 60.0,
            dew_point: 10.0,
            precipitation: 0.0,
            wind_speed: 10.0,
            wind_gusts: 20.0,
            uv_index: 4.0,
        }
    }

    fn voix(decalage: i64, temperature: f64, pluie: f64, code: u16) -> HeureSource {
        HeureSource {
            time: MIDI + decalage * HEURE,
            temperature: Some(temperature),
            precipitation: Some(pluie),
            code: Some(code),
            vent: Some(12.0),
            ..Default::default()
        }
    }

    fn serie(id: &str, heures: Vec<HeureSource>) -> SerieSource {
        SerieSource { source_id: id.to_owned(), heures, jours: Vec::new() }
    }

    fn courant() -> CurrentSample {
        CurrentSample {
            time: MIDI + 20 * 60_000,
            temperature: 18.4,
            apparent_temperature: 17.4,
            weather_code: 1,
            is_day: true,
            relative_humidity: 60.0,
            wind_speed: 10.0,
            wind_gusts: 20.0,
            pressure: 1015.0,
        }
    }

    #[test]
    fn la_mediane_ignore_le_modele_qui_s_egare() {
        assert_eq!(mediane(&[19.0, 19.5, 20.0, 26.0]), Some(19.75));
        assert_eq!(mediane(&[19.0, 19.5, 20.0, 26.0, 12.0]), Some(19.5));
        assert_eq!(mediane(&[]), None);
        assert_eq!(mediane(&[f64::NAN, 3.0]), Some(3.0));
    }

    #[test]
    fn le_risque_mele_la_part_qui_mouille_et_le_risque_publie() {
        // Six sources sur neuf mouillent : 66,7 % ; publié 40 % → 53 %.
        let cumuls = [0.4, 0.2, 0.0, 1.1, 0.0, 0.3, 0.0, 0.5, 0.2];
        assert_eq!(risque(&cumuls, &[40.0]), Some(53.0));
        // Sans risque publié, la part seule.
        assert_eq!(risque(&cumuls, &[]), Some(67.0));
        assert_eq!(risque(&[], &[40.0]), None);
    }

    #[test]
    fn le_temps_retenu_est_celui_de_la_majorite() {
        // Trois sèches contre deux mouillées : le sec le plus cité.
        assert_eq!(code_majoritaire(&[2, 2, 3, 61, 63], &[0.0, 0.0, 0.0, 0.4, 1.0]), Some(2));
        // Trois mouillées : la pluie la plus citée.
        assert_eq!(code_majoritaire(&[3, 61, 61, 80], &[0.0, 0.3, 0.2, 0.6]), Some(61));
        // À égalité, le plus marqué.
        assert_eq!(code_majoritaire(&[61, 80, 3], &[0.3, 0.6, 0.0]), Some(80));
        // Mouillées sans code de pluie : on garde tous les codes.
        assert_eq!(code_majoritaire(&[3, 3], &[0.4, 0.5]), Some(3));
        assert_eq!(code_majoritaire(&[], &[0.4]), None);
    }

    #[test]
    fn chaque_heure_prend_la_mediane_des_sources() {
        let base_heures = vec![base(0), base(1)];
        let series = vec![
            serie("a", vec![voix(0, 19.0, 0.0, 2), voix(1, 20.0, 0.6, 61)]),
            serie("b", vec![voix(0, 20.0, 0.0, 3), voix(1, 21.0, 0.4, 61)]),
            serie("c", vec![voix(0, 26.0, 0.0, 2), voix(1, 20.5, 0.0, 3)]),
        ];
        let r = recouper(&base_heures, &[], &courant(), &series, None);

        let h0 = &r.heures[0];
        assert_eq!(h0.temperature, 20.0, "le 26 de c ne tire rien");
        assert_eq!(h0.apparent_temperature, 19.0, "sans ressenti publié, la base suit l'écart");
        assert_eq!(h0.weather_code, 2);
        assert_eq!(h0.precipitation_probability, 0.0);
        assert_eq!(h0.wind_speed, 12.0);
        assert_eq!(h0.wind_gusts, 20.0, "personne n'a donné de rafales : la base");
        assert_eq!(h0.uv_index, 4.0, "l'UV reste celui de la base");

        let h1 = &r.heures[1];
        assert_eq!(h1.temperature, 20.5);
        assert_eq!(h1.precipitation, 0.4);
        assert_eq!(h1.precipitation_probability, 67.0, "deux sources sur trois mouillent");
        assert_eq!(h1.weather_code, 61);
        assert_eq!(r.sources, ["a", "b", "c"]);
    }

    #[test]
    fn une_heure_sans_voix_garde_la_base() {
        let base_heures = vec![base(0), base(1)];
        let series = vec![serie("a", vec![voix(0, 19.0, 0.0, 2)])];
        let r = recouper(&base_heures, &[], &courant(), &series, None);
        assert_eq!(r.heures[1], base(1));
    }

    #[test]
    fn une_source_sans_temperature_ne_vote_pas() {
        let muette = SerieSource {
            source_id: "muette".into(),
            heures: vec![HeureSource { time: MIDI, ..Default::default() }],
            jours: Vec::new(),
        };
        let r = recouper(&[base(0)], &[], &courant(), &[muette], None);
        assert_eq!(r.heures[0].temperature, 18.0);
        assert!(r.sources.is_empty());
    }

    #[test]
    fn l_instant_present_suit_l_heure_recoupee_et_la_station() {
        let series = vec![
            serie("a", vec![voix(0, 20.0, 0.0, 2)]),
            serie("b", vec![voix(0, 20.0, 0.0, 2)]),
        ];
        // L'heure passe de 18 à 20 : l'instant de 18,4 à 20,4.
        let r = recouper(&[base(0)], &[], &courant(), &series, None);
        assert!((r.courant.temperature - 20.4).abs() < 1e-9);
        assert!((r.courant.apparent_temperature - 19.4).abs() < 1e-9);
        assert_eq!(r.courant.weather_code, 2);
        // Une station mesure 19 : la médiane de 20,4 et 19.
        let r = recouper(&[base(0)], &[], &courant(), &series, Some(19.0));
        assert!((r.courant.temperature - 19.7).abs() < 1e-9);
        // Sans aucune voix, la base telle quelle.
        let r = recouper(&[base(0)], &[], &courant(), &[], None);
        assert_eq!(r.courant, courant());
    }

    #[test]
    fn les_journees_prennent_aussi_la_mediane() {
        let jour = DailySample {
            date: MIDI - 12 * HEURE,
            weather_code: 1,
            temperature_min: 10.0,
            temperature_max: 20.0,
            precipitation_sum: 0.0,
            precipitation_probability_max: 5.0,
            wind_gusts_max: 30.0,
            uv_index_max: 5.0,
            sunrise: Some(1),
            sunset: Some(2),
        };
        let j = |min: f64, max: f64, cumul: f64, code: u16| JourSource {
            date: MIDI - 12 * HEURE,
            minimum: Some(min),
            maximum: Some(max),
            cumul: Some(cumul),
            code: Some(code),
            ..Default::default()
        };
        let series: Vec<SerieSource> = [j(9.0, 21.0, 0.0, 2), j(11.0, 22.0, 2.0, 61), j(10.0, 23.0, 1.0, 61)]
            .into_iter()
            .enumerate()
            .map(|(i, jour)| SerieSource { source_id: i.to_string(), heures: Vec::new(), jours: vec![jour] })
            .collect();
        let r = recouper(&[], std::slice::from_ref(&jour), &courant(), &series, None);
        let d = &r.jours[0];
        assert_eq!((d.temperature_min, d.temperature_max), (10.0, 22.0));
        assert_eq!(d.precipitation_sum, 1.0);
        assert_eq!(d.precipitation_probability_max, 67.0);
        assert_eq!(d.weather_code, 61);
        assert_eq!((d.uv_index_max, d.sunrise), (5.0, Some(1)), "UV, lever et coucher : la base");
    }
}
