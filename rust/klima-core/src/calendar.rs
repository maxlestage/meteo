//! Le calendrier, réduit à ce dont Klima a besoin.
//!
//! Le cœur n'a aucune dépendance — pas même une bibliothèque de dates. Ce
//! n'est pas de l'ascétisme : les horodatages sont ceux de la parcelle, et une
//! bibliothèque de dates passe son temps à proposer des fuseaux dont on ne
//! veut pas ici. Reste à savoir découper un horodatage en année, mois, jour et
//! heure, ce qui tient en trente lignes.
//!
//! L'algorithme de conversion est celui de Howard Hinnant (`civil_from_days`) :
//! il place l'origine de l'ère au 1er mars, ce qui met le 29 février en fin de
//! cycle et fait disparaître le cas particulier des années bissextiles.

const DAY_MS: i64 = 86_400_000;
const HOUR_MS: i64 = 3_600_000;
const MINUTE_MS: i64 = 60_000;

/// Un instant découpé, dans le fuseau de la parcelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilDateTime {
    pub year: i32,
    /// 1 à 12.
    pub month: u32,
    /// 1 à 31.
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
}

/// Le minuit de la journée qui contient cet instant.
///
/// `div_euclid` et non une division entière : pour un horodatage antérieur à
/// 1970, la seconde tronquerait vers zéro et placerait minuit après l'instant.
pub fn at_midnight(ms: i64) -> i64 {
    ms.div_euclid(DAY_MS) * DAY_MS
}

/// L'heure locale d'un horodatage de parcelle, de 0 à 23.
pub fn hour_of(ms: i64) -> i64 {
    ms.rem_euclid(DAY_MS) / HOUR_MS
}

/// Le nombre de jours écoulés depuis l'époque, négatif avant 1970.
pub fn days_since_epoch(ms: i64) -> i64 {
    ms.div_euclid(DAY_MS)
}

/// Découpe un horodatage.
pub fn civil_from_ms(ms: i64) -> CivilDateTime {
    let days = days_since_epoch(ms);
    let in_day = ms.rem_euclid(DAY_MS);
    let (year, month, day) = civil_from_days(days);

    CivilDateTime {
        year,
        month,
        day,
        hour: (in_day / HOUR_MS) as u32,
        minute: (in_day % HOUR_MS / MINUTE_MS) as u32,
    }
}

/// Année, mois et jour d'un nombre de jours depuis l'époque.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    // On décale l'origine au 1er mars 0000, où le 29 février tombe en dernier.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097; // 0 à 146 096
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153; // 0 = mars, 11 = février
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    // Janvier et février appartiennent à l'année civile suivante.
    let year = if month <= 2 { year + 1 } else { year };

    (year as i32, month as u32, day as u32)
}

/// L'horodatage d'une date civile, à l'heure et à la minute données.
///
/// Réciproque de `civil_from_ms`, surtout utile aux tests : elle leur permet
/// d'écrire une date plutôt qu'un nombre de millisecondes.
pub fn ms_from_civil(date: CivilDateTime) -> i64 {
    days_from_civil(date.year, date.month, date.day) * DAY_MS
        + i64::from(date.hour) * HOUR_MS
        + i64::from(date.minute) * MINUTE_MS
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = i64::from(year) - if month <= 2 { 1 } else { 0 };
    let era = y.div_euclid(400);
    let year_of_era = y - era * 400;
    let shifted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era =
        year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

/// Raccourci de lecture : `civil(2026, 4, 15, 9, 0)`.
pub fn civil(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> i64 {
    ms_from_civil(CivilDateTime { year, month, day, hour, minute })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lepoque_est_le_premier_janvier_1970() {
        assert_eq!(civil_from_ms(0), CivilDateTime {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0
        });
        assert_eq!(civil(1970, 1, 1, 0, 0), 0);
    }

    #[test]
    fn decoupe_une_date_avec_son_heure() {
        let date = civil_from_ms(civil(2026, 4, 15, 9, 37));
        assert_eq!(date, CivilDateTime {
            year: 2026,
            month: 4,
            day: 15,
            hour: 9,
            minute: 37
        });
    }

    #[test]
    fn le_vingt_neuf_fevrier_existe_les_annees_bissextiles() {
        let bissextile = civil_from_ms(civil(2024, 2, 29, 12, 0));
        assert_eq!((bissextile.month, bissextile.day), (2, 29));

        // 2100 n'est pas bissextile : le 29 février y tombe le 1er mars.
        let fausse = civil_from_ms(civil(2100, 2, 29, 0, 0));
        assert_eq!((fausse.month, fausse.day), (3, 1));
    }

    #[test]
    fn avant_lepoque_minuit_reste_avant_linstant() {
        let ms = civil(1962, 7, 4, 3, 15);
        assert!(ms < 0);
        assert!(at_midnight(ms) <= ms);
        assert_eq!(civil_from_ms(at_midnight(ms)), CivilDateTime {
            year: 1962,
            month: 7,
            day: 4,
            hour: 0,
            minute: 0
        });
    }

    #[test]
    fn laller_et_retour_tient_sur_quatre_cents_ans() {
        // Un cycle grégorien complet, de jour en jour : si l'algorithme se
        // décale d'un jour quelque part, c'est ici que ça se voit.
        let mut ms = civil(1800, 1, 1, 6, 30);
        let fin = civil(2200, 1, 1, 6, 30);
        while ms < fin {
            assert_eq!(ms_from_civil(civil_from_ms(ms)), ms);
            ms += DAY_MS;
        }
    }

    #[test]
    fn minuit_est_le_meme_pour_toutes_les_heures_de_la_journee() {
        let jour = civil(2026, 4, 15, 0, 0);
        for heure in 0..24 {
            assert_eq!(at_midnight(jour + heure * HOUR_MS), jour);
        }
    }
}
