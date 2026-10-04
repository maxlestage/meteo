//! Traduction des codes météo WMO renvoyés par Open-Meteo.
//!
//! Le code renvoie une clé, pas un libellé : le texte affiché dépend de la
//! langue et vit dans les catalogues. La même table existe en TypeScript
//! en Swift (`ios/Kliima/Models/WeatherCondition.swift`).

/// Famille de pictogramme, déclinée jour / nuit à l'affichage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionIcon {
    Clear,
    Partly,
    Cloudy,
    Fog,
    Drizzle,
    Rain,
    Showers,
    Snow,
    Thunder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherCondition {
    /// Clé de catalogue, par exemple « wmo.drizzle ».
    pub label_key: &'static str,
    pub icon: ConditionIcon,
}

use ConditionIcon::*;

/// Un code inconnu retombe sur « couvert » : afficher un ciel plausible vaut
/// mieux que ne rien afficher, et Open-Meteo peut ajouter des codes.
const UNKNOWN: WeatherCondition =
    WeatherCondition { label_key: "wmo.overcast", icon: Cloudy };

/// Clé de libellé et pictogramme d'un code WMO.
pub fn weather_condition(code: u16) -> WeatherCondition {
    let (label_key, icon) = match code {
        0 => ("wmo.clearSky", Clear),
        1 => ("wmo.mainlyClear", Partly),
        2 => ("wmo.partlyCloudy", Partly),
        3 => ("wmo.overcast", Cloudy),
        45 => ("wmo.fog", Fog),
        48 => ("wmo.rimeFog", Fog),
        51 => ("wmo.lightDrizzle", Drizzle),
        53 => ("wmo.drizzle", Drizzle),
        55 => ("wmo.denseDrizzle", Drizzle),
        56 => ("wmo.freezingDrizzle", Drizzle),
        57 => ("wmo.denseFreezingDrizzle", Drizzle),
        61 => ("wmo.slightRain", Rain),
        63 => ("wmo.rain", Rain),
        65 => ("wmo.heavyRain", Rain),
        66 => ("wmo.freezingRain", Rain),
        67 => ("wmo.heavyFreezingRain", Rain),
        71 => ("wmo.slightSnow", Snow),
        73 => ("wmo.snow", Snow),
        75 => ("wmo.heavySnow", Snow),
        77 => ("wmo.snowGrains", Snow),
        80 => ("wmo.showers", Showers),
        81 => ("wmo.moderateShowers", Showers),
        82 => ("wmo.violentShowers", Showers),
        85 => ("wmo.snowShowers", Snow),
        86 => ("wmo.heavySnowShowers", Snow),
        95 => ("wmo.thunderstorm", Thunder),
        96 => ("wmo.thunderstormHail", Thunder),
        99 => ("wmo.thunderstormHeavyHail", Thunder),
        _ => return UNKNOWN,
    };

    WeatherCondition { label_key, icon }
}

/// Les codes documentés, pour que les tests de catalogue sachent quoi vérifier.
pub const DOCUMENTED_CODES: [u16; 28] = [
    0, 1, 2, 3, 45, 48, 51, 53, 55, 56, 57, 61, 63, 65, 66, 67, 71, 73, 75, 77, 80, 81, 82, 85,
    86, 95, 96, 99,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ciel_degage() {
        assert_eq!(
            weather_condition(0),
            WeatherCondition { label_key: "wmo.clearSky", icon: Clear }
        );
    }

    #[test]
    fn bruine_comme_sur_la_cote_basque() {
        assert_eq!(weather_condition(53).label_key, "wmo.drizzle");
        assert_eq!(weather_condition(53).icon, Drizzle);
    }

    #[test]
    fn les_trois_intensites_de_pluie_partagent_le_meme_pictogramme() {
        let icones: Vec<_> = [61, 63, 65].iter().map(|c| weather_condition(*c).icon).collect();
        assert_eq!(icones, vec![Rain, Rain, Rain]);
    }

    #[test]
    fn orage_avec_grele() {
        assert_eq!(weather_condition(96).icon, Thunder);
    }

    #[test]
    fn code_inconnu_repli_sans_planter() {
        assert_eq!(
            weather_condition(42),
            WeatherCondition { label_key: "wmo.overcast", icon: Cloudy }
        );
    }

    /// Chaque code documenté a une clé qui lui est propre : une faute de frappe
    /// dans la table ferait pointer deux codes sur le même libellé.
    #[test]
    fn chaque_code_documente_a_sa_propre_cle() {
        let mut vues = std::collections::HashMap::new();
        for code in DOCUMENTED_CODES {
            let cle = weather_condition(code).label_key;
            assert!(cle.starts_with("wmo."), "code {code} : clé « {cle} »");
            // « snow » et « snowShowers » sont deux codes distincts, mais 73 et
            // 85 ne doivent pas partager la même clé.
            if let Some(autre) = vues.insert(cle, code) {
                panic!("codes {autre} et {code} partagent la clé « {cle} »");
            }
        }
        assert_eq!(vues.len(), DOCUMENTED_CODES.len());
    }
}
