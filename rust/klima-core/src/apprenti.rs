//! L'apprenti : chaque modèle noté sur ce qui s'est vraiment passé.
//!
//! Miroir Swift : `ios/Kliima/Models/Apprenti.swift`, avec les mêmes cas de
//! test. Toute règle ajoutée d'un côté se porte de l'autre.
//!
//! Les sources votent heure par heure (`fusion`), mais toutes ne se valent
//! pas partout : un modèle peut avoir un degré de trop, semaine après semaine,
//! sur une ville. L'apprenti le voit. Le relais lui donne, pour les sept
//! derniers jours, ce que chaque modèle avait annoncé **la veille** pour chaque
//! heure (prévisions déjà faites, archivées par Open-Meteo) et ce que
//! l'aéroport le plus proche a relevé à cette heure-là. L'apprenti en tire, par
//! modèle :
//!
//! - son **biais** de température — de combien il se trompe en moyenne, et
//!   dans quel sens ;
//! - son **écart** — de combien il se trompe tout court ;
//! - sa **justesse sur la pluie** — la part des heures où il a dit juste,
//!   pluie ou pas pluie.
//!
//! Et il corrige : avant le vote, la température d'un modèle est décalée de
//! son biais appris. Le vote reste un vote — médiane, part des sources qui
//! mouillent, majorité — mais entre des modèles redressés.
//!
//! Prudence d'apprenti :
//!
//! - rien n'est corrigé sous `seuils::RELEVES_MIN` heures comparées : une
//!   journée ne fait pas un défaut ;
//! - le biais est tiré vers zéro tant que les comparaisons sont peu
//!   nombreuses (`seuils::RETENUE`) ;
//! - la correction ne dépasse jamais `seuils::CORRECTION_MAX` : au-delà,
//!   c'est l'aéroport qui ne ressemble pas à la ville, pas le modèle qui a
//!   tort ;
//! - la pluie n'est pas corrigée : elle se vote, et ce qui tombe se voit
//!   (`ciel`, `radar`). Sa justesse se montre, elle ne pèse pas.
//!
//! Une source que l'apprenti n'a pas pu noter — MET Norway n'archive pas ses
//! prévisions passées — vote telle quelle.

use crate::fusion::SerieSource;

/// Les seuils, au même endroit que leur raison.
pub mod seuils {
    /// Heures comparées en dessous desquelles on ne corrige rien.
    pub const RELEVES_MIN: u32 = 24;
    /// Le biais appris est multiplié par `n / (n + RETENUE)` : à 24 heures, la
    /// moitié ; à une semaine pleine, près de neuf dixièmes.
    pub const RETENUE: f64 = 24.0;
    /// La plus grande correction de température permise (°C).
    pub const CORRECTION_MAX: f64 = 3.0;
    /// Un relevé ne compte pour une heure que s'il est à moins de tant de
    /// secondes d'elle : les bulletins tombent à l'heure ou à la demie.
    pub const TOLERANCE_S: i64 = 600;
    /// Un modèle « mouille » une heure à partir de ce cumul (mm).
    pub const PLUIE_MM: f64 = crate::ville::seuils::PLUIE_MM;
}

use seuils::*;

/// Ce que l'aéroport a relevé. `time` en secondes UTC.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Releve {
    pub time: i64,
    pub temperature: Option<f64>,
    /// Vrai si quelque chose tombait (`ciel::tombe_du_metar`).
    pub mouille: bool,
}

/// Ce qu'un modèle avait annoncé la veille pour une heure. `time` en
/// secondes UTC.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Annonce {
    pub time: i64,
    pub temperature: Option<f64>,
    pub precipitation: Option<f64>,
}

/// La note d'une source.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub source_id: String,
    /// Heures où la température a pu être comparée.
    pub n: u32,
    /// Moyenne de (annoncé − relevé), °C. Positif : le modèle voit trop chaud.
    pub biais: f64,
    /// Moyenne de |annoncé − relevé|, °C.
    pub ecart: f64,
    /// Heures où la pluie a pu être comparée, et celles où il a dit juste.
    pub pluie_n: u32,
    pub pluie_justes: u32,
}

impl Note {
    /// Ce qu'on retranche à la température de ce modèle avant le vote (°C).
    /// Zéro tant qu'il n'y a pas assez de comparaisons.
    pub fn correction(&self) -> f64 {
        if self.n < RELEVES_MIN || !self.biais.is_finite() {
            return 0.0;
        }
        let n = f64::from(self.n);
        let retenu = self.biais * n / (n + RETENUE);
        let borne = retenu.clamp(-CORRECTION_MAX, CORRECTION_MAX);
        (borne * 10.0).round() / 10.0
    }

    /// La part des heures où il a dit juste sur la pluie (0–100), s'il y en a.
    pub fn justesse_pluie(&self) -> Option<f64> {
        (self.pluie_n > 0).then(|| (f64::from(self.pluie_justes) / f64::from(self.pluie_n) * 100.0).round())
    }
}

/// Le relevé qui vaut pour une heure : le plus proche, dans la tolérance.
fn releve_de(releves: &[Releve], heure: i64) -> Option<&Releve> {
    releves
        .iter()
        .filter(|r| (r.time - heure).abs() <= TOLERANCE_S)
        .min_by_key(|r| (r.time - heure).abs())
}

/// Note une source : ce qu'elle avait annoncé, contre ce qui a été relevé.
/// Seules les heures déjà passées comptent — `maintenant` en secondes UTC.
pub fn noter(source_id: &str, annonces: &[Annonce], releves: &[Releve], maintenant: i64) -> Note {
    let (mut n, mut somme, mut absolue) = (0_u32, 0.0, 0.0);
    let (mut pluie_n, mut pluie_justes) = (0_u32, 0_u32);
    for annonce in annonces.iter().filter(|a| a.time <= maintenant) {
        let Some(releve) = releve_de(releves, annonce.time) else { continue };
        if let (Some(prevu), Some(vu)) = (annonce.temperature, releve.temperature) {
            if prevu.is_finite() && vu.is_finite() {
                n += 1;
                somme += prevu - vu;
                absolue += (prevu - vu).abs();
            }
        }
        if let Some(cumul) = annonce.precipitation.filter(|c| c.is_finite()) {
            pluie_n += 1;
            if (cumul >= PLUIE_MM) == releve.mouille {
                pluie_justes += 1;
            }
        }
    }
    let moyenne = |x: f64| if n == 0 { 0.0 } else { (x / f64::from(n) * 100.0).round() / 100.0 };
    Note { source_id: source_id.to_owned(), n, biais: moyenne(somme), ecart: moyenne(absolue), pluie_n, pluie_justes }
}

/// Redresse les séries avant le vote : la température, le ressenti et les
/// bornes du jour de chaque source notée sont décalés de sa correction. Une
/// source sans note, ou trop peu notée, reste telle quelle.
pub fn corriger(series: &mut [SerieSource], notes: &[Note]) {
    for serie in series.iter_mut() {
        let Some(note) = notes.iter().find(|n| n.source_id == serie.source_id) else { continue };
        let c = note.correction();
        if c == 0.0 {
            continue;
        }
        for heure in &mut serie.heures {
            heure.temperature = heure.temperature.map(|t| t - c);
            heure.ressenti = heure.ressenti.map(|t| t - c);
        }
        for jour in &mut serie.jours {
            jour.minimum = jour.minimum.map(|t| t - c);
            jour.maximum = jour.maximum.map(|t| t - c);
        }
    }
}

/// La source la plus juste en température, parmi celles assez notées.
pub fn meilleure(notes: &[Note]) -> Option<&Note> {
    notes.iter().filter(|n| n.n >= RELEVES_MIN).min_by(|a, b| a.ecart.total_cmp(&b.ecart))
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fusion::{HeureSource, JourSource};

    const H: i64 = 3600;

    fn releves(n: i64, temperature: f64) -> Vec<Releve> {
        (0..n).map(|i| Releve { time: i * H, temperature: Some(temperature), mouille: i % 4 == 0 }).collect()
    }

    fn annonces(n: i64, temperature: f64) -> Vec<Annonce> {
        (0..n)
            .map(|i| Annonce { time: i * H, temperature: Some(temperature), precipitation: Some(if i % 4 == 0 { 1.0 } else { 0.0 }) })
            .collect()
    }

    #[test]
    fn un_modele_qui_voit_trop_chaud_a_un_biais_positif() {
        let note = noter("gfs", &annonces(48, 12.0), &releves(48, 10.0), 100 * H);
        assert_eq!(note.n, 48);
        assert_eq!(note.biais, 2.0);
        assert_eq!(note.ecart, 2.0);
        assert_eq!(note.pluie_n, 48);
        assert_eq!(note.pluie_justes, 48);
        assert_eq!(note.justesse_pluie(), Some(100.0));
    }

    #[test]
    fn la_correction_est_retenue_puis_bornee() {
        let note = |n: u32, biais: f64| Note { source_id: "x".into(), n, biais, ecart: biais.abs(), pluie_n: 0, pluie_justes: 0 };
        // Trop peu de comparaisons : rien.
        assert_eq!(note(23, 2.0).correction(), 0.0);
        // À vingt-quatre heures, la moitié du biais.
        assert_eq!(note(24, 2.0).correction(), 1.0);
        // Une semaine : près de neuf dixièmes.
        assert_eq!(note(168, 2.0).correction(), 1.8);
        // Jamais plus de trois degrés.
        assert_eq!(note(168, 9.0).correction(), 3.0);
        assert_eq!(note(168, -9.0).correction(), -3.0);
    }

    #[test]
    fn un_releve_hors_tolerance_ne_compte_pas() {
        let annonces = [Annonce { time: 10 * H, temperature: Some(15.0), precipitation: None }];
        let loin = [Releve { time: 10 * H + 1200, temperature: Some(10.0), mouille: false }];
        assert_eq!(noter("x", &annonces, &loin, 20 * H).n, 0);
        let pres = [
            Releve { time: 10 * H + 1800, temperature: Some(0.0), mouille: false },
            Releve { time: 10 * H - 300, temperature: Some(14.0), mouille: false },
        ];
        let note = noter("x", &annonces, &pres, 20 * H);
        assert_eq!((note.n, note.biais), (1, 1.0));
    }

    #[test]
    fn l_avenir_ne_se_note_pas() {
        let note = noter("x", &annonces(48, 12.0), &releves(48, 10.0), 9 * H);
        assert_eq!(note.n, 10);
    }

    #[test]
    fn la_pluie_ratee_se_compte() {
        let annonces = [
            Annonce { time: 0, temperature: None, precipitation: Some(0.0) },
            Annonce { time: H, temperature: None, precipitation: Some(2.0) },
        ];
        let releves = [
            Releve { time: 0, temperature: None, mouille: true },
            Releve { time: H, temperature: None, mouille: true },
        ];
        let note = noter("x", &annonces, &releves, 10 * H);
        assert_eq!((note.n, note.pluie_n, note.pluie_justes), (0, 2, 1));
        assert_eq!(note.justesse_pluie(), Some(50.0));
    }

    #[test]
    fn corriger_redresse_la_source_notee_et_elle_seule() {
        let serie = |id: &str| SerieSource {
            source_id: id.into(),
            heures: vec![HeureSource { time: 0, temperature: Some(12.0), ressenti: Some(11.0), ..Default::default() }],
            jours: vec![JourSource { date: 0, minimum: Some(8.0), maximum: Some(14.0), ..Default::default() }],
        };
        let mut series = vec![serie("gfs"), serie("icon"), serie("met")];
        let notes = vec![
            Note { source_id: "gfs".into(), n: 168, biais: 2.0, ecart: 2.0, pluie_n: 0, pluie_justes: 0 },
            Note { source_id: "icon".into(), n: 10, biais: 2.0, ecart: 2.0, pluie_n: 0, pluie_justes: 0 },
        ];
        corriger(&mut series, &notes);
        assert_eq!(series[0].heures[0].temperature, Some(10.2));
        assert_eq!(series[0].heures[0].ressenti, Some(9.2));
        assert_eq!(series[0].jours[0].minimum, Some(6.2));
        assert_eq!(series[0].jours[0].maximum, Some(12.2));
        assert_eq!(series[1].heures[0].temperature, Some(12.0));
        assert_eq!(series[2].heures[0].temperature, Some(12.0));
    }

    #[test]
    fn la_meilleure_est_la_plus_juste_parmi_les_assez_notees() {
        let note = |id: &str, n: u32, ecart: f64| Note { source_id: id.into(), n, biais: 0.0, ecart, pluie_n: 0, pluie_justes: 0 };
        let notes = vec![note("a", 100, 1.4), note("b", 10, 0.2), note("c", 100, 0.9)];
        assert_eq!(meilleure(&notes).map(|n| n.source_id.as_str()), Some("c"));
        assert_eq!(meilleure(&[note("b", 10, 0.2)]), None);
    }
}
