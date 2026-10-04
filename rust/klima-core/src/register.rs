//! Le registre : les conditions à l'heure d'un traitement.
//!
//! Miroir Swift : `ios/Kliima/Models/Register.swift`.
//!
//! Tenir un registre des traitements phytosanitaires est une obligation. Klima
//! **n'est pas ce registre** et ne prétend pas l'être : il fournit la partie
//! pénible à reconstituer après coup — les conditions météo relevées à l'heure
//! de l'application. Le reste (produit, dose, culture, opérateur) appartient à
//! l'exploitant, et le module se contente de le recopier s'il le fournit.
//!
//! Ce que la réglementation regarde à l'heure d'une application : le vent, ses
//! rafales, la température et l'hygrométrie. On y ajoute la pluie de l'heure et
//! le verdict de Klima, qui n'a aucune valeur réglementaire mais dit ce que
//! l'outil aurait conseillé.

use crate::agro::{HourlySample, SprayVerdict, evaluate_spray_hour};
use crate::calendar::civil_from_ms;

const HOUR_MS: i64 = 3_600_000;

#[derive(Debug, Clone, PartialEq)]
pub struct TreatmentRecord {
    /// Heure de l'application, dans le fuseau de la parcelle.
    pub at: i64,
    pub parcelle: String,
    /// Renseigné par l'exploitant : Klima ne l'invente pas.
    pub product: Option<String>,
    pub temperature: f64,
    pub relative_humidity: f64,
    pub wind_speed: f64,
    pub wind_gusts: f64,
    pub precipitation: f64,
    /// Ce que Klima aurait conseillé — indicatif, sans valeur réglementaire.
    pub verdict: SprayVerdict,
    pub score: i32,
}

/// Relève les conditions de l'heure qui contient `at`.
///
/// Renvoie `None` si la série ne couvre pas ce moment : mieux vaut une ligne
/// absente qu'une ligne inventée dans un document qu'on pourra vous opposer.
pub fn record_at(
    hours: &[HourlySample],
    at: i64,
    parcelle: &str,
    product: Option<&str>,
) -> Option<TreatmentRecord> {
    let slot = at.div_euclid(HOUR_MS);
    let index = hours.iter().position(|hour| hour.time.div_euclid(HOUR_MS) == slot)?;

    let hour = &hours[index];
    let window = evaluate_spray_hour(hours, index).ok()?;

    Some(TreatmentRecord {
        at: hour.time,
        parcelle: parcelle.to_owned(),
        product: product.map(str::to_owned),
        temperature: hour.temperature,
        relative_humidity: hour.relative_humidity,
        wind_speed: hour.wind_speed,
        wind_gusts: hour.wind_gusts,
        precipitation: hour.precipitation,
        verdict: window.verdict,
        score: window.score,
    })
}

/// Les colonnes du document, dans l'ordre. Les en-têtes sont des clés.
pub const REGISTER_COLUMNS: [&str; 11] = [
    "date",
    "heure",
    "parcelle",
    "produit",
    "temperature",
    "humidite",
    "vent",
    "rafales",
    "pluie",
    "verdict",
    "score",
];

#[derive(Debug, Clone)]
pub struct CsvOptions {
    /// Séparateur de colonnes. Le point-virgule par défaut : Excel en langue
    /// française lit un fichier à virgules comme une seule colonne.
    pub delimiter: char,
    /// Séparateur décimal. La virgule par défaut, pour la même raison.
    pub decimal: char,
    /// En-têtes déjà traduits, dans l'ordre de `REGISTER_COLUMNS`.
    pub headers: Option<Vec<String>>,
}

impl Default for CsvOptions {
    fn default() -> Self {
        CsvOptions { delimiter: ';', decimal: ',', headers: None }
    }
}

impl CsvOptions {
    /// Les réglages d'un tableur anglais : la virgule sépare les colonnes, le
    /// point les décimales.
    pub fn anglais() -> Self {
        CsvOptions { delimiter: ',', decimal: '.', headers: None }
    }
}

fn escape(value: &str, delimiter: char) -> String {
    // Un guillemet se double, et tout champ qui contient un séparateur, un
    // guillemet ou un saut de ligne se met entre guillemets. Sans ça, une
    // parcelle nommée « Le Clos ; bas » casserait la colonne suivante.
    let needs_quotes =
        value.contains(delimiter) || value.contains('"') || value.contains(['\r', '\n']);
    let escaped = value.replace('"', "\"\"");
    if needs_quotes { format!("\"{escaped}\"") } else { escaped }
}

/// Un nombre à `digits` décimales, avec le séparateur demandé.
///
/// L'arrondi est celui du reste du cœur — `floor(x + 0.5)` — et non celui que
/// Rust applique à l'affichage, qui départage les demis vers le nombre pair :
/// 0,25 s'écrirait « 0,2 » là où le TypeScript et Swift écrivent « 0,3 ».
fn number(value: f64, digits: u32, decimal: char) -> String {
    let factor = 10f64.powi(digits as i32);
    let rounded = (value * factor + 0.5).floor() / factor;
    format!("{rounded:.*}", digits as usize).replace('.', &decimal.to_string())
}

fn two(value: u32) -> String {
    format!("{value:02}")
}

/// Met les relevés en CSV.
///
/// Le fichier commence par une marque d'ordre des octets : sans elle, Excel
/// lit l'UTF-8 comme du Latin-1 et « évapotranspiration » perd ses accents.
pub fn to_csv(records: &[TreatmentRecord], options: &CsvOptions) -> String {
    let delimiter = options.delimiter;
    let decimal = options.decimal;

    let headers: Vec<String> = match &options.headers {
        Some(headers) => headers.clone(),
        None => REGISTER_COLUMNS.iter().map(|column| (*column).to_owned()).collect(),
    };

    let mut lines = vec![
        headers
            .iter()
            .map(|header| escape(header, delimiter))
            .collect::<Vec<_>>()
            .join(&delimiter.to_string()),
    ];

    for record in records {
        let date = civil_from_ms(record.at);
        let fields = [
            format!("{}-{}-{}", date.year, two(date.month), two(date.day)),
            format!("{}:{}", two(date.hour), two(date.minute)),
            escape(&record.parcelle, delimiter),
            escape(record.product.as_deref().unwrap_or(""), delimiter),
            number(record.temperature, 1, decimal),
            number(record.relative_humidity, 0, decimal),
            number(record.wind_speed, 1, decimal),
            number(record.wind_gusts, 1, decimal),
            number(record.precipitation, 1, decimal),
            record.verdict.code().to_owned(),
            record.score.to_string(),
        ];
        lines.push(fields.join(&delimiter.to_string()));
    }

    format!("\u{feff}{}\r\n", lines.join("\r\n"))
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::civil;

    /// Le 15 avril 2026, 8 h à la parcelle.
    fn start() -> i64 {
        civil(2026, 4, 15, 8, 0)
    }

    fn hour(offset: i64) -> HourlySample {
        HourlySample {
            time: start() + offset * HOUR_MS,
            weather_code: 3,
            is_day: true,
            precipitation_probability: 10.0,
            temperature: 17.4,
            relative_humidity: 62.0,
            dew_point: 9.0,
            precipitation: 0.0,
            wind_speed: 11.2,
            wind_gusts: 18.5,
            soil_temperature_6cm: 12.0,
            soil_moisture_3to9cm: 0.22,
            et0: 0.1,
            vapour_pressure_deficit: 0.6,
        }
    }

    fn series() -> Vec<HourlySample> {
        vec![hour(0), hour(1), hour(2), hour(3)]
    }

    fn releve() -> TreatmentRecord {
        record_at(&series(), civil(2026, 4, 15, 9, 0), "Le Clos", Some("Cuivre")).unwrap()
    }

    fn ligne(csv: &str, index: usize) -> String {
        csv.trim_start_matches('\u{feff}').split("\r\n").nth(index).unwrap().to_owned()
    }

    /* ---- le relevé ---- */

    #[test]
    fn prend_les_conditions_de_lheure_qui_contient_le_moment_donne() {
        let record =
            record_at(&series(), civil(2026, 4, 15, 9, 37), "Le Clos", None).unwrap();

        assert_eq!(record.at, civil(2026, 4, 15, 9, 0));
        assert_eq!(record.parcelle, "Le Clos");
        assert_eq!(record.wind_speed, 11.2);
        assert_eq!(record.relative_humidity, 62.0);
    }

    #[test]
    fn porte_le_verdict_de_klima_a_titre_indicatif() {
        let record = record_at(&series(), civil(2026, 4, 15, 9, 0), "Le Clos", None).unwrap();
        assert!(
            [SprayVerdict::Favorable, SprayVerdict::Acceptable, SprayVerdict::Defavorable]
                .contains(&record.verdict)
        );
        assert!(record.score >= 0);
    }

    #[test]
    fn le_produit_vient_de_lexploitant_jamais_de_klima() {
        let sans = record_at(&series(), civil(2026, 4, 15, 9, 0), "Le Clos", None).unwrap();
        assert_eq!(sans.product, None);

        let avec =
            record_at(&series(), civil(2026, 4, 15, 9, 0), "Le Clos", Some("Cuivre")).unwrap();
        assert_eq!(avec.product.as_deref(), Some("Cuivre"));
    }

    #[test]
    fn hors_de_la_serie_pas_de_ligne_plutot_quune_ligne_inventee() {
        // Un document qu'on pourra vous opposer ne se remplit pas au jugé.
        assert_eq!(record_at(&series(), civil(2026, 4, 20, 9, 0), "Le Clos", None), None);
    }

    /* ---- l'export ---- */

    #[test]
    fn separe_par_point_virgule_et_decime_a_la_virgule() {
        // Excel en français lit un fichier à virgules comme une seule colonne.
        let csv = to_csv(&[releve()], &CsvOptions::default());
        let line = ligne(&csv, 1);

        assert!(line.contains(';'));
        assert!(line.contains("11,2"));
        assert!(!line.contains("11.2"));
    }

    #[test]
    fn commence_par_la_marque_dordre_des_octets() {
        // Sans elle, Excel lit l'UTF-8 comme du Latin-1.
        assert!(to_csv(&[releve()], &CsvOptions::default()).starts_with('\u{feff}'));
    }

    #[test]
    fn la_date_et_lheure_sont_celles_de_la_parcelle() {
        let csv = to_csv(&[releve()], &CsvOptions::default());
        assert!(ligne(&csv, 1).starts_with("2026-04-15;09:00;"));
    }

    #[test]
    fn une_parcelle_qui_contient_le_separateur_ne_casse_pas_la_colonne_suivante() {
        let mut piege = releve();
        piege.parcelle = "Le Clos ; bas".to_owned();
        let line = ligne(&to_csv(&[piege], &CsvOptions::default()), 1);

        assert!(line.contains("\"Le Clos ; bas\""));
        // Onze colonnes malgré le point-virgule dans le nom.
        assert!(line.split(';').count() > REGISTER_COLUMNS.len());
        assert_eq!(line.matches('"').count(), 2);
    }

    #[test]
    fn un_guillemet_dans_un_nom_se_double() {
        let mut piege = releve();
        piege.parcelle = "Le \"Clos\"".to_owned();
        assert!(
            to_csv(&[piege], &CsvOptions::default()).contains("\"Le \"\"Clos\"\"\"")
        );
    }

    #[test]
    fn len_tete_suit_lordre_des_colonnes_et_se_traduit() {
        let headers = [
            "Date",
            "Heure",
            "Parcelle",
            "Produit",
            "Température",
            "Humidité",
            "Vent",
            "Rafales",
            "Pluie",
            "Avis",
            "Score",
        ]
        .map(str::to_owned)
        .to_vec();
        let csv = to_csv(
            &[releve()],
            &CsvOptions { headers: Some(headers), ..CsvOptions::default() },
        );

        assert_eq!(
            ligne(&csv, 0),
            "Date;Heure;Parcelle;Produit;Température;Humidité;Vent;Rafales;Pluie;Avis;Score"
        );
    }

    #[test]
    fn chaque_ligne_a_autant_de_champs_que_de_colonnes() {
        let csv = to_csv(&[releve()], &CsvOptions::default());
        assert_eq!(ligne(&csv, 1).split(';').count(), REGISTER_COLUMNS.len());
    }

    #[test]
    fn un_export_vide_garde_son_en_tete() {
        let csv = to_csv(&[], &CsvOptions::default());
        assert_eq!(csv.split("\r\n").filter(|line| !line.is_empty()).count(), 1);
    }

    #[test]
    fn on_peut_demander_le_point_et_la_virgule_pour_un_tableur_anglais() {
        let csv = to_csv(&[releve()], &CsvOptions::anglais());
        let line = ligne(&csv, 1);

        assert!(line.contains("11.2"));
        assert_eq!(line.split(',').count(), REGISTER_COLUMNS.len());
    }

    #[test]
    fn le_fichier_est_octet_pour_octet_celui_du_typescript() {
        // Octet pour octet ce que produisait le TypeScript sur le même
        // relevé : personne n'héritera de deux registres différents selon
        // l'application qui l'a exporté.
        assert_eq!(
            to_csv(&[releve()], &CsvOptions::default()),
            concat!(
                "\u{feff}date;heure;parcelle;produit;temperature;humidite;vent;rafales;",
                "pluie;verdict;score\r\n",
                "2026-04-15;09:00;Le Clos;Cuivre;17,4;62;11,2;18,5;0,0;favorable;100\r\n",
            )
        );
    }

    #[test]
    fn un_demi_sarrondit_comme_dans_les_deux_autres_langues() {
        // 0,25 mm de pluie s'écrit « 0,3 » en TypeScript et en Swift ;
        // l'affichage de Rust écrirait « 0,2 ».
        assert_eq!(number(0.25, 1, ','), "0,3");
        assert_eq!(number(-0.25, 1, ','), "-0,2");
        assert_eq!(number(61.5, 0, ','), "62");
    }
}
