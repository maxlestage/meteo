//! Le ciel observé : ce qui tombe **vraiment**, d'après l'aéroport le plus
//! proche.
//!
//! Miroir Swift : `ios/Kliima/Models/Ciel.swift`, avec les mêmes cas de test.
//! Toute règle ajoutée d'un côté se porte de l'autre.
//!
//! Toutes les autres sources de Klima sont des prévisions : des modèles qui
//! calculent le ciel. Une averse qu'aucun n'a vue venir n'existe pour aucun —
//! et quelqu'un, sous la pluie, lisait « pas une goutte ». Les aéroports, eux,
//! regardent : chaque demi-heure ou chaque heure, leur bulletin (METAR) dit le
//! temps présent — `-RA` pour une pluie faible, `+SHRA` pour une forte averse,
//! `TS` pour un orage. L'Aviation Weather Center de la NOAA les publie tous,
//! dans le domaine public.
//!
//! On retient le bulletin le plus proche, s'il est assez près pour parler de la
//! ville et assez récent pour parler de maintenant. La règle est **à sens
//! unique** : une pluie observée s'ajoute à la prévision, un ciel sec observé
//! n'en retire rien — un aéroport à quinze kilomètres qui ne voit rien tomber
//! ne prouve pas qu'il ne pleut pas en ville. Ce qui est vu tomber, en
//! revanche, tombe.
//!
//! Les horodatages sont ceux de la ville : des millisecondes à l'heure locale.

use crate::veille::{Intensite, Nature, Precipitation};

/// Les seuils, au même endroit que leur raison.
pub mod seuils {
    /// Au-delà, l'aéroport parle d'un autre ciel (km).
    pub const RAYON_KM: f64 = 30.0;
    /// Au-delà, le bulletin parle d'un autre moment (ms). Les METAR partent
    /// toutes les demi-heures ou toutes les heures : une heure et quart laisse
    /// passer un bulletin horaire en retard, pas un bulletin périmé.
    pub const AGE_MAX_MS: i64 = 75 * 60_000;
    /// Un bulletin daté d'un peu après « maintenant » : les horloges ne sont
    /// jamais tout à fait d'accord.
    pub const AVANCE_MAX_MS: i64 = 10 * 60_000;
    /// Ce qu'un quart observé mouillé compte, selon l'intensité (mm) : un
    /// débit faible, modéré et fort au sens du guetteur.
    pub const QUART_FAIBLE_MM: f64 = 0.2;
    pub const QUART_MODERE_MM: f64 = 1.0;
    pub const QUART_FORT_MM: f64 = 2.5;
}

use seuils::*;

/// Un bulletin d'aéroport, déjà lu.
#[derive(Debug, Clone, PartialEq)]
pub struct Metar {
    /// Indicatif OACI, `LFPG`.
    pub station: String,
    /// Nom lisible, tel que le bulletin le donne, débarrassé de son suffixe.
    pub nom: String,
    pub latitude: f64,
    pub longitude: f64,
    /// Heure de l'observation, à l'heure de la ville (ms).
    pub time: i64,
    /// Le temps présent, brut : `-RA BR`, `+TSRA`… Vide si rien à signaler.
    pub temps_present: String,
}

/// Le ciel observé près de la ville.
#[derive(Debug, Clone, PartialEq)]
pub struct CielObserve {
    pub station: String,
    pub nom: String,
    /// Distance à la ville, arrondie au kilomètre.
    pub distance_km: f64,
    /// Heure de l'observation, à l'heure de la ville (ms).
    pub time: i64,
    /// Ce qui tombe, en code météo de l'OMM ; `None` si rien ne tombe.
    pub tombe: Option<u16>,
}

impl CielObserve {
    /// Ce qui tombe, pour le dire : nature et intensité.
    pub fn precipitation(&self) -> Option<Precipitation> {
        self.tombe.map(precipitation_du_code)
    }
}

/// Distance à vol d'oiseau (km), sur une Terre ronde.
pub fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const RAYON_TERRE_KM: f64 = 6371.0;
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let dp = (lat2 - lat1).to_radians();
    let dl = (lon2 - lon1).to_radians();
    let a = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    2.0 * RAYON_TERRE_KM * a.sqrt().asin()
}

/// Le cadre où chercher les aéroports, autour d'un point : sud, ouest, nord,
/// est, en degrés, arrondis au centième. Un peu plus large que le rayon, pour
/// qu'un point déplacé à sa maille par le relais trouve encore les mêmes.
pub fn cadre(latitude: f64, longitude: f64) -> (f64, f64, f64, f64) {
    let marge_km = RAYON_KM + 10.0;
    let dlat = marge_km / 111.32;
    let dlon = (marge_km / (111.32 * latitude.to_radians().cos().max(0.05))).min(180.0);
    let c = |x: f64| (x * 100.0).round() / 100.0;
    (
        c((latitude - dlat).max(-90.0)),
        c((longitude - dlon).max(-180.0)),
        c((latitude + dlat).min(90.0)),
        c((longitude + dlon).min(180.0)),
    )
}

/// Le code météo de l'OMM d'un temps présent METAR, `None` si rien ne tombe.
///
/// Chaque groupe se lit à part ; on garde le plus marqué. Ce qui est « au
/// voisinage » (`VC`) ou « récent » (`RE`) ne tombe pas sur la ville
/// maintenant, et ne compte pas. Brume, brouillard, poussière : rien ne tombe.
pub fn code_du_metar(temps_present: &str) -> Option<u16> {
    temps_present.split_whitespace().filter_map(code_du_groupe).max()
}

fn code_du_groupe(groupe: &str) -> Option<u16> {
    if groupe.starts_with("VC") || groupe.starts_with("RE") {
        return None;
    }
    let (force, reste) = match groupe.as_bytes().first() {
        Some(b'-') => (0, &groupe[1..]),
        Some(b'+') => (2, &groupe[1..]),
        _ => (1, groupe),
    };
    // Chasse-neige, sable soulevé : ça vole, ça ne tombe pas.
    if reste.starts_with("BL") || reste.starts_with("DR") {
        return None;
    }
    let a = |motif: &str| reste.contains(motif);
    let selon = |codes: [u16; 3]| codes[force];

    if a("TS") {
        return Some(if a("GR") || a("GS") { if force == 2 { 99 } else { 96 } } else { 95 });
    }
    if a("SH") {
        if a("SN") {
            return Some(if force == 2 { 86 } else { 85 });
        }
        if a("RA") || a("GR") || a("GS") || a("UP") {
            return Some(selon([80, 81, 82]));
        }
    }
    if a("FZ") {
        if a("RA") || a("UP") {
            return Some(if force == 2 { 67 } else { 66 });
        }
        if a("DZ") {
            return Some(if force == 2 { 57 } else { 56 });
        }
    }
    if a("SN") {
        return Some(selon([71, 73, 75]));
    }
    if a("SG") || a("PL") || a("GR") || a("GS") || a("IC") {
        return Some(77);
    }
    if a("RA") || a("UP") {
        return Some(selon([61, 63, 65]));
    }
    if a("DZ") {
        return Some(selon([51, 53, 55]));
    }
    None
}

/// Nature et intensité d'un code météo mouillé.
pub fn precipitation_du_code(code: u16) -> Precipitation {
    let nature = match code {
        95 | 96 | 99 => Nature::Orage,
        71..=77 | 85 | 86 => Nature::Neige,
        _ => Nature::Pluie,
    };
    let intensite = match code {
        53 | 63 | 73 | 81 => Intensite::Moderee,
        55 | 57 | 65 | 67 | 75 | 82 | 86 | 99 => Intensite::Forte,
        _ => Intensite::Faible,
    };
    Precipitation { nature, intensite }
}

/// Ce qu'un quart observé mouillé compte (mm), selon le code.
pub fn quart_observe_mm(code: u16) -> f64 {
    match precipitation_du_code(code).intensite {
        Intensite::Faible => QUART_FAIBLE_MM,
        Intensite::Moderee => QUART_MODERE_MM,
        Intensite::Forte => QUART_FORT_MM,
    }
}

/// Le bulletin qui parle de la ville : le plus proche dans le rayon, parmi
/// ceux qui parlent de maintenant. `None` s'il n'y en a pas.
pub fn plus_proche(metars: &[Metar], latitude: f64, longitude: f64, maintenant: i64) -> Option<CielObserve> {
    metars
        .iter()
        .filter(|m| m.time <= maintenant + AVANCE_MAX_MS && maintenant - m.time <= AGE_MAX_MS)
        .map(|m| (distance_km(latitude, longitude, m.latitude, m.longitude), m))
        .filter(|(d, _)| *d <= RAYON_KM)
        // À distance égale, le plus récent.
        .min_by(|(da, a), (db, b)| da.total_cmp(db).then(b.time.cmp(&a.time)))
        .map(|(d, m)| CielObserve {
            station: m.station.clone(),
            nom: m.nom.clone(),
            distance_km: d.round(),
            time: m.time,
            tombe: code_du_metar(&m.temps_present),
        })
}

/// Le nom lisible d'un aéroport, d'après celui du bulletin :
/// `Paris/Le Bourge Arpt, ID, FR` → `Paris/Le Bourge`.
pub fn nom_lisible(brut: &str) -> String {
    let premier = brut.split(',').next().unwrap_or("").trim();
    let mut nom = premier;
    for suffixe in [" Intl Arpt", " Arpt", " Airport", " Aprt", " Intl", " AB", " Afb"] {
        if let Some(court) = nom.strip_suffix(suffixe) {
            nom = court.trim_end();
            break;
        }
    }
    nom.to_owned()
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    const MAINTENANT: i64 = 1_791_302_400_000;
    const MINUTE: i64 = 60_000;

    fn metar(station: &str, latitude: f64, longitude: f64, age_min: i64, temps: &str) -> Metar {
        Metar {
            station: station.to_owned(),
            nom: station.to_owned(),
            latitude,
            longitude,
            time: MAINTENANT - age_min * MINUTE,
            temps_present: temps.to_owned(),
        }
    }

    #[test]
    fn le_temps_present_devient_un_code_de_l_omm() {
        assert_eq!(code_du_metar("-RA"), Some(61));
        assert_eq!(code_du_metar("RA BR"), Some(63));
        assert_eq!(code_du_metar("+RA"), Some(65));
        assert_eq!(code_du_metar("-DZ"), Some(51));
        assert_eq!(code_du_metar("-SHRA"), Some(80));
        assert_eq!(code_du_metar("+SHRA"), Some(82));
        assert_eq!(code_du_metar("SHSN"), Some(85));
        assert_eq!(code_du_metar("-SN"), Some(71));
        assert_eq!(code_du_metar("FZRA"), Some(66));
        assert_eq!(code_du_metar("TSRA"), Some(95));
        assert_eq!(code_du_metar("+TSGR"), Some(99));
        assert_eq!(code_du_metar("TS"), Some(95));
        assert_eq!(code_du_metar("-RASN"), Some(71), "la neige l'emporte sur la pluie");
    }

    #[test]
    fn ce_qui_ne_tombe_pas_sur_la_ville_ne_compte_pas() {
        assert_eq!(code_du_metar(""), None);
        assert_eq!(code_du_metar("BR"), None);
        assert_eq!(code_du_metar("FG"), None);
        assert_eq!(code_du_metar("VCSH"), None, "une averse au voisinage");
        assert_eq!(code_du_metar("RERA"), None, "une pluie récente, finie");
        assert_eq!(code_du_metar("BLSN"), None, "de la neige soulevée par le vent");
        assert_eq!(code_du_metar("-DZ -RA"), Some(61), "le plus marqué des groupes");
    }

    #[test]
    fn nature_et_intensite_d_un_code() {
        let p = precipitation_du_code(82);
        assert_eq!((p.nature, p.intensite), (Nature::Pluie, Intensite::Forte));
        let p = precipitation_du_code(73);
        assert_eq!((p.nature, p.intensite), (Nature::Neige, Intensite::Moderee));
        assert_eq!(precipitation_du_code(95).nature, Nature::Orage);
        assert_eq!(quart_observe_mm(61), QUART_FAIBLE_MM);
        assert_eq!(quart_observe_mm(65), QUART_FORT_MM);
    }

    #[test]
    fn le_bulletin_retenu_est_le_plus_proche_et_assez_recent() {
        // Brest : Guipavas à 9 km, Lanvéoc à 13 km.
        let (lat, lon) = (48.39, -4.4861);
        let bulletins = vec![
            metar("LFRL", 48.279, -4.439, 10, "-RA"),
            metar("LFRB", 48.444, -4.412, 10, ""),
            metar("LFRJ", 48.527, -4.138, 10, "+RA"),
        ];
        let ciel = plus_proche(&bulletins, lat, lon, MAINTENANT).unwrap();
        assert_eq!(ciel.station, "LFRB");
        assert_eq!(ciel.distance_km, 8.0);
        assert_eq!(ciel.tombe, None);

        // Guipavas date de deux heures : Lanvéoc parle à sa place.
        let vieux = vec![metar("LFRB", 48.444, -4.412, 120, ""), metar("LFRL", 48.279, -4.439, 20, "-RA")];
        let ciel = plus_proche(&vieux, lat, lon, MAINTENANT).unwrap();
        assert_eq!((ciel.station.as_str(), ciel.tombe), ("LFRL", Some(61)));
        assert_eq!(ciel.precipitation().unwrap().key(), "veille.kind.pluie.faible");
    }

    #[test]
    fn trop_loin_ou_trop_vieux_le_ciel_n_est_pas_observe() {
        let loin = vec![metar("LFRJ", 48.527, -4.138, 10, "-RA")];
        assert!(plus_proche(&loin, 48.0, -3.0, MAINTENANT).is_none());
        let vieux = vec![metar("LFRB", 48.444, -4.412, 80, "-RA")];
        assert!(plus_proche(&vieux, 48.39, -4.4861, MAINTENANT).is_none());
        let futur = vec![metar("LFRB", 48.444, -4.412, -20, "-RA")];
        assert!(plus_proche(&futur, 48.39, -4.4861, MAINTENANT).is_none());
    }

    #[test]
    fn un_nom_d_aeroport_lisible() {
        assert_eq!(nom_lisible("Paris/Le Bourge Arpt, ID, FR"), "Paris/Le Bourge");
        assert_eq!(nom_lisible("Brest/Guipavas Intl Arpt, BRE, FR"), "Brest/Guipavas");
        assert_eq!(nom_lisible("Villacoublay, ID, FR"), "Villacoublay");
    }

    #[test]
    fn le_cadre_couvre_le_rayon_a_toutes_les_latitudes() {
        assert_eq!(cadre(48.39, -4.4861), (48.03, -5.03, 48.75, -3.94));
        let (s, o, n, e) = cadre(60.0, 10.0);
        assert!(distance_km(60.0, 10.0, 60.0, e) > RAYON_KM && distance_km(60.0, 10.0, 60.0, o) > RAYON_KM);
        assert!(distance_km(60.0, 10.0, s, 10.0) > RAYON_KM && distance_km(60.0, 10.0, n, 10.0) > RAYON_KM);
        assert_eq!(cadre(89.9, 0.0).2, 90.0);
    }

    #[test]
    fn la_distance_a_vol_d_oiseau() {
        // Paris – Lyon : 392 km.
        let d = distance_km(48.8566, 2.3522, 45.764, 4.8357);
        assert!((d - 392.0).abs() < 2.0, "{d}");
    }
}
