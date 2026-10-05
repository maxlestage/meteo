//! Ce que la prévision dit d'une ville, heure par heure et jour par jour.
//!
//! Miroir Swift : `ios/Kliima/Models/AgroSamples.swift` (le nom du fichier
//! est resté celui de l'époque agricole ; son contenu est celui-ci).
//!
//! Les horodatages sont ceux de la ville : des millisecondes à l'heure
//! locale, sans décalage. « 21 h » veut dire 21 h là-bas, quel que soit le
//! fuseau de qui calcule.

/// Une heure de prévision.
#[derive(Debug, Clone, PartialEq)]
pub struct HourlySample {
    /// Millisecondes depuis l'époque, à l'heure de la ville.
    pub time: i64,
    pub weather_code: u16,
    pub is_day: bool,
    /// Probabilité de pluie (%).
    pub precipitation_probability: f64,
    pub temperature: f64,
    /// Température ressentie (°C).
    pub apparent_temperature: f64,
    pub relative_humidity: f64,
    pub dew_point: f64,
    /// Précipitations sur l'heure (mm).
    pub precipitation: f64,
    /// Vent moyen (km/h).
    pub wind_speed: f64,
    /// Rafales (km/h).
    pub wind_gusts: f64,
    /// Indice UV.
    pub uv_index: f64,
}

/// Les conditions de l'instant.
#[derive(Debug, Clone, PartialEq)]
pub struct CurrentSample {
    /// Millisecondes depuis l'époque, à l'heure de la ville.
    pub time: i64,
    pub temperature: f64,
    /// Température ressentie (°C).
    pub apparent_temperature: f64,
    pub weather_code: u16,
    pub is_day: bool,
    pub relative_humidity: f64,
    pub wind_speed: f64,
    pub wind_gusts: f64,
    /// Pression ramenée au niveau de la mer (hPa).
    pub pressure: f64,
}

/// Une journée de prévision.
#[derive(Debug, Clone, PartialEq)]
pub struct DailySample {
    pub date: i64,
    pub weather_code: u16,
    pub temperature_min: f64,
    pub temperature_max: f64,
    pub precipitation_sum: f64,
    pub precipitation_probability_max: f64,
    pub wind_gusts_max: f64,
    /// Indice UV le plus fort de la journée.
    pub uv_index_max: f64,
    pub sunrise: Option<i64>,
    pub sunset: Option<i64>,
}
