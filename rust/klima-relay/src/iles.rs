//! Les îles dynamiques que le relais tient à l'heure.
//!
//! Quand l'iPhone ouvre une activité météo, il donne au relais le jeton
//! qu'Apple lui a remis pour elle, et la maille de la parcelle. Le relais
//! regarde chaque minute ce que l'île devrait afficher ; quand ça change — une
//! heure qui commence, une prévision qui se renouvelle —, il pousse.
//!
//! **Ce que le relais garde.** Le jeton, la maille, et l'empreinte de ce qu'il
//! a poussé en dernier. Pas de nom de parcelle, pas de compte, pas d'adresse :
//! rien qui dise qui suit quoi. En mémoire seulement — un redémarrage oublie
//! tout, et l'iPhone se réinscrit à sa prochaine ouverture ; l'île, entre-temps,
//! bascule encore seule à l'heure pile, comme sans relais.
//!
//! **Le calcul est celui de l'iPhone.** Même heure en cours, même heure
//! suivante, même date de péremption (`klima_core::horizon`, miroir de
//! `Horizon.swift`), et le JSON est exactement celui que `ContentState`
//! décode : les clés de `WeatherActivityAttributes.swift`, les dates en
//! secondes depuis le 1er janvier 2001.

use std::collections::HashMap;
use std::sync::Mutex;

use klima_api::open_meteo::Forecast;
use klima_core::meteo::HourlySample;
use klima_core::grid::cell_for;
use klima_core::horizon;
use serde_json::{Value, json};

/// Au-delà, on refuse les inscriptions : de quoi tenir largement, sans laisser
/// la mémoire du relais à la merci de qui inscrirait des jetons en boucle.
pub const CAPACITE: usize = 5_000;

/// iOS ferme une activité au bout de huit heures, et la laisse quatre heures
/// de plus sur l'écran verrouillé. Passé douze heures, plus rien ne l'affiche.
pub const DUREE_MS: i64 = 12 * 3_600_000;

/// L'écart entre l'époque Unix et celle d'Apple (1er janvier 2001), en
/// secondes. `JSONDecoder` lit une `Date` comme un nombre de secondes depuis
/// celle-ci : c'est ce qu'attend `ContentState`.
const EPOQUE_APPLE_S: f64 = 978_307_200.0;

const HEURE_MS: i64 = 3_600_000;

#[derive(Debug, Clone, PartialEq)]
struct Ile {
    latitude: f64,
    longitude: f64,
    inscrite: i64,
    /// L'empreinte de l'état poussé en dernier, et l'heure qu'il montrait.
    poussee: Option<(String, i64)>,
}

/// Une île à examiner.
#[derive(Debug, Clone, PartialEq)]
pub struct AExaminer {
    pub jeton: String,
    pub latitude: f64,
    pub longitude: f64,
    pub poussee: Option<(String, i64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refus {
    /// Le jeton n'a pas la forme de ceux d'Apple.
    Jeton,
    /// Le point n'est pas sur Terre.
    Point,
    /// Le registre est plein.
    Plein,
}

#[derive(Debug, Default)]
pub struct Iles {
    carte: Mutex<HashMap<String, Ile>>,
}

impl Iles {
    pub fn new() -> Self {
        Iles::default()
    }

    /// Inscrit une île, ou la réinscrit : un jeton déjà connu repart pour
    /// douze heures, à la maille donnée.
    pub fn inscrire(&self, jeton: &str, latitude: f64, longitude: f64, maintenant: i64) -> Result<(), Refus> {
        let jeton = jeton.trim().to_ascii_lowercase();
        if !(16..=512).contains(&jeton.len()) || !jeton.bytes().all(|o| o.is_ascii_hexdigit()) {
            return Err(Refus::Jeton);
        }
        let sur_terre = latitude.is_finite()
            && longitude.is_finite()
            && (-90.0..=90.0).contains(&latitude)
            && (-180.0..=180.0).contains(&longitude);
        if !sur_terre {
            return Err(Refus::Point);
        }

        // La maille, pas le point : le relais n'a pas à savoir à deux mètres
        // près où se trouve la parcelle, et deux îles d'une même maille
        // partagent ainsi leur prévision.
        let maille = cell_for(latitude, longitude);
        let mut carte = self.carte.lock().map_err(|_| Refus::Plein)?;
        retirer_les_echues(&mut carte, maintenant);
        if carte.len() >= CAPACITE && !carte.contains_key(&jeton) {
            return Err(Refus::Plein);
        }
        carte.insert(
            jeton,
            Ile { latitude: maille.latitude, longitude: maille.longitude, inscrite: maintenant, poussee: None },
        );
        Ok(())
    }

    pub fn retirer(&self, jeton: &str) {
        if let Ok(mut carte) = self.carte.lock() {
            carte.remove(&jeton.trim().to_ascii_lowercase());
        }
    }

    pub fn taille(&self) -> usize {
        self.carte.lock().map_or(0, |c| c.len())
    }

    /// Les îles encore vivantes, après avoir oublié les échues.
    pub fn a_examiner(&self, maintenant: i64) -> Vec<AExaminer> {
        let Ok(mut carte) = self.carte.lock() else { return Vec::new() };
        retirer_les_echues(&mut carte, maintenant);
        carte
            .iter()
            .map(|(jeton, ile)| AExaminer {
                jeton: jeton.clone(),
                latitude: ile.latitude,
                longitude: ile.longitude,
                poussee: ile.poussee.clone(),
            })
            .collect()
    }

    /// Note ce qui vient d'être poussé — si l'île est toujours là.
    pub fn noter(&self, jeton: &str, empreinte: String, heure: i64) {
        if let Ok(mut carte) = self.carte.lock() {
            if let Some(ile) = carte.get_mut(jeton) {
                ile.poussee = Some((empreinte, heure));
            }
        }
    }
}

fn retirer_les_echues(carte: &mut HashMap<String, Ile>, maintenant: i64) {
    carte.retain(|_, ile| maintenant - ile.inscrite < DUREE_MS);
}

/// Ce qu'une île doit afficher à un instant.
#[derive(Debug, Clone, PartialEq)]
pub struct Contenu {
    /// Le `content-state`, aux clés de `ContentState`.
    pub etat: Value,
    /// L'empreinte de ce qui se voit — tout sauf l'horodatage.
    pub empreinte: String,
    /// Le début de l'heure montrée, à l'heure de la parcelle.
    pub heure: i64,
    /// Le moment où l'affichage cesse d'être vrai, en secondes Unix.
    pub perime: i64,
}

/// Calcule ce qu'une île doit afficher à `maintenant` (millisecondes Unix).
///
/// Le relevé, s'il est de l'heure en cours ; sinon l'heure prévue — une
/// prévision servie depuis le cache peut dater de l'heure d'avant, et un relevé
/// de 13 h 45 n'a pas à se présenter comme celui de 14 h 05. Faute de
/// ressenti dans la prévision horaire, il vaut la température : c'est ce que
/// fait `CurrentSample(prevu:)` côté iPhone.
pub fn contenu(prevision: &Forecast, maintenant: i64) -> Option<Contenu> {
    let local = maintenant + prevision.utc_offset_seconds * 1000;
    let heure = horizon::heure_contenant(local, &prevision.hourly)?;
    let jour = horizon::jour_contenant(local, &prevision.daily);
    let suivante = horizon::heure_suivante(local, &prevision.hourly);

    let releve = &prevision.current;
    let (temperature, ressenti, code, jour_ou_nuit, vent) = if releve.time >= heure.time {
        (releve.temperature, releve.apparent_temperature, releve.weather_code, releve.is_day, releve.wind_speed)
    } else {
        (heure.temperature, heure.temperature, heure.weather_code, heure.is_day, heure.wind_speed)
    };

    let date_apple = |local_ms: i64| prevision.instant(local_ms) as f64 / 1000.0 - EPOQUE_APPLE_S;
    let prochaine = suivante.map(|h: &HourlySample| {
        json!({
            "start": date_apple(h.time),
            "temperature": h.temperature,
            "weatherCode": h.weather_code,
            "isDay": h.is_day,
            "precipitationProbability": h.precipitation_probability,
            "windSpeed": h.wind_speed,
        })
    });

    let mut etat = json!({
        "temperature": temperature,
        "apparentTemperature": ressenti,
        "weatherCode": code,
        "isDay": jour_ou_nuit,
        "windSpeed": vent,
        "temperatureMin": jour.map_or(temperature, |j| j.temperature_min),
        "temperatureMax": jour.map_or(temperature, |j| j.temperature_max),
    });
    if let Some(prochaine) = prochaine {
        etat["next"] = prochaine;
    }
    let empreinte = etat.to_string();
    etat["updatedAt"] = json!(maintenant as f64 / 1000.0 - EPOQUE_APPLE_S);

    let bascule = horizon::prochaine_bascule(local, &prevision.hourly)
        .map_or(maintenant + HEURE_MS, |b| prevision.instant(b));
    Some(Contenu { etat, empreinte, heure: heure.time, perime: bascule.div_euclid(1000) })
}

/// Le corps de la notification : une mise à jour, datée, et sa péremption.
///
/// La date compte : Apple ignore une mise à jour plus ancienne que celle
/// affichée, et c'est ce qui empêche une poussée en retard d'écraser ce que
/// l'iPhone vient de mettre lui-même.
pub fn corps(contenu: &Contenu, maintenant: i64) -> String {
    json!({
        "aps": {
            "timestamp": maintenant.div_euclid(1000),
            "event": "update",
            "content-state": contenu.etat,
            "stale-date": contenu.perime,
        }
    })
    .to_string()
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use klima_core::calendar::civil;
    use klima_core::position::Parcelle;

    /// Paris en heure d'été : UTC+2.
    pub(crate) const DECALAGE_S: i64 = 7_200;

    /// 2026-10-04 à `h`:`m`, heure de la parcelle, en millisecondes Unix.
    pub(crate) fn a(h: u32, m: u32) -> i64 {
        civil(2026, 10, 4, h, m) - DECALAGE_S * 1000
    }

    /// Une réponse d'Open-Meteo, relevée à 13 h 45, heures de 13 h à 18 h.
    pub(crate) fn reponse() -> String {
        let heures: Vec<String> = (13..=18).map(|h| format!("2026-10-04T{h:02}:00")).collect();
        json!({
            "timezone": "Europe/Paris",
            "utc_offset_seconds": DECALAGE_S,
            "elevation": 150.0,
            "current": {
                "time": "2026-10-04T13:45",
                "temperature_2m": 21.4,
                "apparent_temperature": 20.1,
                "relative_humidity_2m": 55,
                "weather_code": 2,
                "is_day": 1,
                "wind_speed_10m": 12.0,
                "wind_gusts_10m": 20.0
            },
            "hourly": {
                "time": heures,
                "temperature_2m": [21.0, 22.0, 23.0, 22.5, 20.0, 18.0],
                "weather_code": [2, 3, 61, 61, 3, 1],
                "is_day": [1, 1, 1, 1, 1, 0],
                "precipitation_probability": [10, 20, 70, 60, 30, 5],
                "wind_speed_10m": [12.0, 14.0, 18.0, 16.0, 10.0, 6.0]
            },
            "daily": {
                "time": ["2026-10-04", "2026-10-05"],
                "weather_code": [61, 3],
                "temperature_2m_min": [11.0, 9.0],
                "temperature_2m_max": [23.0, 19.0]
            }
        })
        .to_string()
    }

    fn prevision() -> Forecast {
        klima_api::open_meteo::decode_forecast(Parcelle::new("Essai", 48.45, 1.49), &reponse(), 0)
            .expect("réponse lisible")
    }

    #[test]
    fn dans_lheure_du_releve_lile_montre_le_releve_et_lheure_suivante() {
        let c = contenu(&prevision(), a(13, 50)).unwrap();
        assert_eq!(c.etat["temperature"], 21.4);
        assert_eq!(c.etat["apparentTemperature"], 20.1);
        assert_eq!(c.etat["weatherCode"], 2);
        assert_eq!(c.etat["isDay"], true);
        assert_eq!(c.etat["temperatureMin"], 11.0);
        assert_eq!(c.etat["temperatureMax"], 23.0);
        assert_eq!(c.etat["next"]["temperature"], 22.0);
        assert_eq!(c.etat["next"]["precipitationProbability"], 20.0);
        // Péremption à 14 h pile, heure de la parcelle.
        assert_eq!(c.perime, a(14, 0) / 1000);
    }

    #[test]
    fn lheure_passee_le_releve_cede_la_place_a_lheure_prevue() {
        let c = contenu(&prevision(), a(14, 5)).unwrap();
        assert_eq!(c.etat["temperature"], 22.0);
        assert_eq!(c.etat["apparentTemperature"], 22.0);
        assert_eq!(c.etat["weatherCode"], 3);
        // Et l'heure qui vient est calculée d'avance : la pluie de 15 h.
        assert_eq!(c.etat["next"]["weatherCode"], 61);
        assert_eq!(c.etat["next"]["precipitationProbability"], 70.0);
        assert_eq!(c.perime, a(15, 0) / 1000);
        assert_eq!(c.heure, a(14, 0) + DECALAGE_S * 1000);
    }

    #[test]
    fn les_dates_sont_en_secondes_depuis_2001_comme_les_lit_activitykit() {
        let c = contenu(&prevision(), a(14, 5)).unwrap();
        let debut = a(15, 0) as f64 / 1000.0 - 978_307_200.0;
        assert_eq!(c.etat["next"]["start"], debut);
        assert_eq!(c.etat["updatedAt"], a(14, 5) as f64 / 1000.0 - 978_307_200.0);
    }

    #[test]
    fn lempreinte_ignore_lhorodatage_mais_pas_le_changement_dheure() {
        let p = prevision();
        assert_eq!(contenu(&p, a(14, 5)).unwrap().empreinte, contenu(&p, a(14, 50)).unwrap().empreinte);
        assert_ne!(contenu(&p, a(14, 50)).unwrap().empreinte, contenu(&p, a(15, 1)).unwrap().empreinte);
    }

    #[test]
    fn en_fin_de_serie_il_ny_a_plus_dheure_suivante_puis_plus_rien() {
        let p = prevision();
        let derniere = contenu(&p, a(18, 10)).unwrap();
        assert!(derniere.etat.get("next").is_none());
        assert!(contenu(&p, a(19, 10)).is_none());
    }

    /// Les clés que décode `ContentState` — le même JSON que celui de
    /// `IlesRelaisTests.swift`. Une clé renommée d'un côté casse l'autre.
    #[test]
    fn les_cles_sont_celles_de_content_state() {
        let c = contenu(&prevision(), a(14, 5)).unwrap();
        let mut cles: Vec<&str> = c.etat.as_object().unwrap().keys().map(String::as_str).collect();
        cles.sort_unstable();
        assert_eq!(
            cles,
            [
                "apparentTemperature", "isDay", "next", "temperature", "temperatureMax",
                "temperatureMin", "updatedAt", "weatherCode", "windSpeed"
            ]
        );
        let mut suivante: Vec<&str> = c.etat["next"].as_object().unwrap().keys().map(String::as_str).collect();
        suivante.sort_unstable();
        assert_eq!(
            suivante,
            ["isDay", "precipitationProbability", "start", "temperature", "weatherCode", "windSpeed"]
        );
    }

    #[test]
    fn le_corps_est_une_mise_a_jour_datee_qui_perime_a_lheure_suivante() {
        let c = contenu(&prevision(), a(14, 5)).unwrap();
        let lu: Value = serde_json::from_str(&corps(&c, a(14, 5))).unwrap();
        assert_eq!(lu["aps"]["event"], "update");
        assert_eq!(lu["aps"]["timestamp"], a(14, 5) / 1000);
        assert_eq!(lu["aps"]["stale-date"], a(15, 0) / 1000);
        assert_eq!(lu["aps"]["content-state"]["next"]["weatherCode"], 61);
    }

    const JETON: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f90";

    #[test]
    fn le_registre_garde_la_maille_pas_le_point() {
        let iles = Iles::new();
        iles.inscrire(JETON, 48.4567, 1.4891, 0).unwrap();
        let vue = &iles.a_examiner(0)[0];
        let maille = cell_for(48.4567, 1.4891);
        assert_eq!((vue.latitude, vue.longitude), (maille.latitude, maille.longitude));
        assert_ne!(vue.latitude, 48.4567);
    }

    #[test]
    fn le_registre_refuse_ce_qui_nest_pas_un_jeton_ou_un_point() {
        let iles = Iles::new();
        assert_eq!(iles.inscrire("pas-hexa", 48.0, 1.0, 0), Err(Refus::Jeton));
        assert_eq!(iles.inscrire("abcd", 48.0, 1.0, 0), Err(Refus::Jeton));
        assert_eq!(iles.inscrire(JETON, 91.0, 1.0, 0), Err(Refus::Point));
        assert_eq!(iles.inscrire(JETON, f64::NAN, 1.0, 0), Err(Refus::Point));
        assert_eq!(iles.taille(), 0);
    }

    #[test]
    fn une_ile_soublie_au_bout_de_douze_heures_ou_quand_on_la_retire() {
        let iles = Iles::new();
        iles.inscrire(JETON, 48.0, 1.0, 0).unwrap();
        assert_eq!(iles.a_examiner(DUREE_MS - 1).len(), 1);
        assert!(iles.a_examiner(DUREE_MS).is_empty());

        iles.inscrire(JETON, 48.0, 1.0, 0).unwrap();
        iles.retirer(&JETON.to_uppercase());
        assert_eq!(iles.taille(), 0);
    }

    #[test]
    fn une_reinscription_repart_de_zero() {
        let iles = Iles::new();
        iles.inscrire(JETON, 48.0, 1.0, 0).unwrap();
        iles.noter(JETON, "vu".to_owned(), 1);
        iles.inscrire(JETON, 48.0, 1.0, 10).unwrap();
        assert_eq!(iles.a_examiner(10)[0].poussee, None);
        assert_eq!(iles.taille(), 1);
    }

    #[test]
    fn plein_le_registre_refuse_les_nouveaux_mais_pas_les_connus() {
        let iles = Iles::new();
        for i in 0..CAPACITE {
            iles.inscrire(&format!("{i:032x}"), 48.0, 1.0, 0).unwrap();
        }
        assert_eq!(iles.inscrire(&format!("{:032x}", CAPACITE + 1), 48.0, 1.0, 0), Err(Refus::Plein));
        assert_eq!(iles.inscrire(&format!("{:032x}", 0), 48.0, 1.0, 0), Ok(()));
    }
}
