//! L'air de la ville : sa qualité, et les pollens.
//!
//! Miroir Swift : `ios/Kliima/Models/Air.swift`, avec les mêmes cas de test.
//!
//! La source est le service de qualité de l'air d'Open-Meteo, qui redistribue
//! les prévisions européennes de Copernicus (CAMS). L'indice est l'indice
//! européen de l'Agence européenne pour l'environnement — le même que celui
//! des panneaux lumineux des grandes villes —, et ses six classes sont les
//! siennes. Les pollens ne sont prévus qu'en Europe : ailleurs, la liste est
//! vide, et l'interface n'en parle pas.

/// La mention que la licence de Copernicus impose d'afficher, dès qu'une
/// mesure d'air est montrée.
pub const ATTRIBUTION: &str =
    "Qualité de l’air et pollens : Copernicus Atmosphere Monitoring Service, par Open-Meteo (CC BY 4.0)";

/// Ce que l'air contient à l'heure en cours.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AirSample {
    /// Indice européen de qualité de l'air (0 et plus).
    pub european_aqi: Option<f64>,
    /// Particules fines (µg/m³).
    pub pm2_5: Option<f64>,
    pub pm10: Option<f64>,
    pub nitrogen_dioxide: Option<f64>,
    pub ozone: Option<f64>,
    /// Grains par mètre cube, espèce par espèce. Vide hors d'Europe.
    pub pollens: Vec<(Pollen, f64)>,
}

/// Les six classes de l'indice européen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualiteAir {
    Bonne,
    Correcte,
    Moyenne,
    Mediocre,
    TresMediocre,
    ExtremementMediocre,
}

impl QualiteAir {
    pub fn code(self) -> &'static str {
        match self {
            QualiteAir::Bonne => "bonne",
            QualiteAir::Correcte => "correcte",
            QualiteAir::Moyenne => "moyenne",
            QualiteAir::Mediocre => "mediocre",
            QualiteAir::TresMediocre => "tresMediocre",
            QualiteAir::ExtremementMediocre => "extremementMediocre",
        }
    }

    pub fn key(self) -> String {
        format!("air.{}", self.code())
    }
}

/// La classe d'un indice européen : bornes de 20 en 20, jusqu'à 100.
pub fn qualite_air(indice: f64) -> QualiteAir {
    match indice {
        i if i <= 20.0 => QualiteAir::Bonne,
        i if i <= 40.0 => QualiteAir::Correcte,
        i if i <= 60.0 => QualiteAir::Moyenne,
        i if i <= 80.0 => QualiteAir::Mediocre,
        i if i <= 100.0 => QualiteAir::TresMediocre,
        _ => QualiteAir::ExtremementMediocre,
    }
}

/// Les espèces que prévoit Copernicus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pollen {
    Aulne,
    Bouleau,
    Graminees,
    Armoise,
    Olivier,
    Ambroisie,
}

pub const POLLENS: [Pollen; 6] =
    [Pollen::Aulne, Pollen::Bouleau, Pollen::Graminees, Pollen::Armoise, Pollen::Olivier, Pollen::Ambroisie];

impl Pollen {
    pub fn code(self) -> &'static str {
        match self {
            Pollen::Aulne => "aulne",
            Pollen::Bouleau => "bouleau",
            Pollen::Graminees => "graminees",
            Pollen::Armoise => "armoise",
            Pollen::Olivier => "olivier",
            Pollen::Ambroisie => "ambroisie",
        }
    }

    /// Le nom de la variable chez Open-Meteo.
    pub fn variable(self) -> &'static str {
        match self {
            Pollen::Aulne => "alder_pollen",
            Pollen::Bouleau => "birch_pollen",
            Pollen::Graminees => "grass_pollen",
            Pollen::Armoise => "mugwort_pollen",
            Pollen::Olivier => "olive_pollen",
            Pollen::Ambroisie => "ragweed_pollen",
        }
    }

    pub fn key(self) -> String {
        format!("pollen.{}", self.code())
    }
}

/// L'intensité d'un pollen.
///
/// Une seule échelle pour toutes les espèces, et c'est une simplification
/// assumée : les réseaux de surveillance en ont une par espèce, et l'ambroisie
/// gêne bien plus tôt que les graminées. Elle suffit à dire « rien », « un
/// peu », « beaucoup » ; pour un traitement, l'allergologue a les siennes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NiveauPollen {
    Faible,
    Modere,
    Eleve,
    TresEleve,
}

impl NiveauPollen {
    pub fn code(self) -> &'static str {
        match self {
            NiveauPollen::Faible => "faible",
            NiveauPollen::Modere => "modere",
            NiveauPollen::Eleve => "eleve",
            NiveauPollen::TresEleve => "tresEleve",
        }
    }

    pub fn key(self) -> String {
        format!("pollenLevel.{}", self.code())
    }
}

/// Moins d'un grain par mètre cube : rien à signaler.
pub const POLLEN_PRESENT: f64 = 1.0;

pub fn niveau_pollen(grains: f64) -> NiveauPollen {
    match grains {
        g if g < 10.0 => NiveauPollen::Faible,
        g if g < 50.0 => NiveauPollen::Modere,
        g if g < 200.0 => NiveauPollen::Eleve,
        _ => NiveauPollen::TresEleve,
    }
}

/// Le pollen le plus présent, s'il y en a un qui vaille d'être dit.
pub fn pollen_dominant(air: &AirSample) -> Option<(Pollen, f64, NiveauPollen)> {
    air.pollens
        .iter()
        .copied()
        .filter(|(_, grains)| grains.is_finite() && *grains >= POLLEN_PRESENT)
        // À égalité, le premier de la liste : un ordre stable des deux côtés.
        .fold(None, |meilleur: Option<(Pollen, f64)>, (p, g)| match meilleur {
            Some((_, m)) if m >= g => meilleur,
            _ => Some((p, g)),
        })
        .map(|(p, g)| (p, g, niveau_pollen(g)))
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_six_classes_de_lindice_europeen() {
        assert_eq!(qualite_air(0.0), QualiteAir::Bonne);
        assert_eq!(qualite_air(20.0), QualiteAir::Bonne);
        assert_eq!(qualite_air(20.5), QualiteAir::Correcte);
        assert_eq!(qualite_air(40.0), QualiteAir::Correcte);
        assert_eq!(qualite_air(55.0), QualiteAir::Moyenne);
        assert_eq!(qualite_air(80.0), QualiteAir::Mediocre);
        assert_eq!(qualite_air(100.0), QualiteAir::TresMediocre);
        assert_eq!(qualite_air(140.0), QualiteAir::ExtremementMediocre);
    }

    #[test]
    fn les_niveaux_de_pollen() {
        assert_eq!(niveau_pollen(0.0), NiveauPollen::Faible);
        assert_eq!(niveau_pollen(9.9), NiveauPollen::Faible);
        assert_eq!(niveau_pollen(10.0), NiveauPollen::Modere);
        assert_eq!(niveau_pollen(50.0), NiveauPollen::Eleve);
        assert_eq!(niveau_pollen(200.0), NiveauPollen::TresEleve);
    }

    #[test]
    fn le_pollen_dominant_est_le_plus_present() {
        let air = AirSample {
            pollens: vec![(Pollen::Bouleau, 12.0), (Pollen::Graminees, 64.0), (Pollen::Ambroisie, 3.0)],
            ..AirSample::default()
        };
        assert_eq!(pollen_dominant(&air), Some((Pollen::Graminees, 64.0, NiveauPollen::Eleve)));
    }

    #[test]
    fn a_egalite_le_premier_de_la_liste() {
        let air = AirSample {
            pollens: vec![(Pollen::Aulne, 20.0), (Pollen::Bouleau, 20.0)],
            ..AirSample::default()
        };
        assert_eq!(pollen_dominant(&air).map(|(p, _, _)| p), Some(Pollen::Aulne));
    }

    #[test]
    fn sous_un_grain_il_ny_a_rien_a_dire() {
        let air = AirSample { pollens: vec![(Pollen::Bouleau, 0.4)], ..AirSample::default() };
        assert_eq!(pollen_dominant(&air), None);
        assert_eq!(pollen_dominant(&AirSample::default()), None);
    }

    #[test]
    fn les_codes_sont_ceux_des_catalogues() {
        assert_eq!(QualiteAir::ExtremementMediocre.key(), "air.extremementMediocre");
        assert_eq!(Pollen::Graminees.key(), "pollen.graminees");
        assert_eq!(Pollen::Ambroisie.variable(), "ragweed_pollen");
        assert_eq!(NiveauPollen::TresEleve.key(), "pollenLevel.tresEleve");
    }
}
