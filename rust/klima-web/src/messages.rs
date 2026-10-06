//! Textes propres à l'application web.
//!
//! fait, états du sol, verdicts, paliers — vivent dans `klima_core::messages`
//! et ne sont pas recopiés ici : le traducteur lit les deux catalogues, celui
//! de l'application d'abord.

use std::sync::LazyLock;

use klima_core::i18n::{Catalog, MessageSet};

pub static WEB_MESSAGES: LazyLock<MessageSet> = LazyLock::new(|| MessageSet {
    fr: catalog(&FR),
    en: catalog(&EN),
    es: catalog(&ES),
});

fn catalog(entries: &[(&'static str, &'static str)]) -> Catalog {
    entries.iter().copied().collect()
}

const FR: [(&str, &str); 52] = [
    ("app.retry", "Réessayer"),
    ("app.refresh", "Actualiser"),
    ("app.updatedAt", "mise à jour {time}"),
    ("app.home", "Retour à la présentation de Klima"),
    ("pro.lead", "Sur iPhone et Apple Watch, Klima Pro ajoute :"),
    ("pro.grant.title", "Accès de test"),
    ("pro.grant.hint", "Si votre adresse fait partie des invitées, saisissez-la : le relais la reconnaîtra."),
    ("pro.grant.field", "Adresse électronique"),
    ("pro.grant.check", "Vérifier"),
    ("pro.grant.checking", "Vérification…"),
    ("pro.grant.refused", "Cette adresse n’ouvre rien pour le moment. Elle est gardée : si l’invitation arrive, le palier suivra."),
    ("pro.grant.active", "Accès de test reconnu par le relais."),
    ("pro.grant.forget", "Retirer mon adresse"),
    ("search.label", "Rechercher une commune"),
    ("search.locate", "Me localiser"),
    ("search.locating", "Localisation…"),
    ("search.unsupported", "Géolocalisation indisponible sur ce navigateur."),
    ("search.denied", "Position refusée. Recherchez la commune à la main."),
    ("hourly.title", "Conditions météo"),
    ("hourly.now", "Maint."),
    ("hourly.unfold", "Voir les 24 heures"),
    ("hourly.fold", "Replier"),
    ("daily.title", "Prévision sur 7 jours"),
    ("daily.today", "Auj."),
    ("tile.wind", "Vent"),
    ("tile.sunrise", "Lever"),
    ("tile.sunrise.caption", "Coucher à {time}."),
    ("language.label", "Langue"),
    ("app.loading", "Chargement de la météo…"),
    ("app.error", "Impossible de charger la prévision."),
    ("app.source", "Données Open-Meteo — modèles Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC et JMA"),
    ("pro.free", "Ici et dans l’application, {parcelles} ville et {jours} jours de prévision restent gratuits, sans compte ni publicité."),
    ("pro.grant.web", "Ici, cela ne change rien de plus : le recoupement et l’air sont déjà offerts, et les alertes vivent dans l’application iPhone."),
    ("search.placeholder", "{parcelle} — changer de ville"),
    ("rain.axisNow", "Maint."),
    ("tile.wind.caption", "Rafales {gusts}."),
    ("tile.feelsLike", "Ressenti"),
    ("tile.feelsLike.caption", "Il fait {temperature} au thermomètre."),
    ("tile.humidity", "Humidité"),
    ("tile.humidity.caption", "Point de rosée {dewPoint}."),
    ("tile.uv", "Indice UV"),
    ("tile.uv.caption", "Jusqu’à {max} aujourd’hui ({level})."),
    ("tile.pressure", "Pression"),
    ("tile.pressure.caption", "Ramenée au niveau de la mer."),
    ("tile.rainToday", "Pluie du jour"),
    ("tile.rainToday.caption", "Risque maximal {probability}."),
    ("tile.air", "Qualité de l’air"),
    ("tile.air.caption", "Indice européen {aqi} · particules fines {pm25}."),
    ("tile.pollen", "Pollens"),
    ("tile.pollen.caption", "{grains} grains/m³ — {level}."),
    ("tile.pollen.none", "Aucun"),
    ("tile.pollen.noneCaption", "Rien de notable dans l’air."),
];

const EN: [(&str, &str); 52] = [
    ("app.retry", "Try again"),
    ("app.refresh", "Refresh"),
    ("app.updatedAt", "updated {time}"),
    ("app.home", "Back to the Klima showcase"),
    ("pro.lead", "On iPhone and Apple Watch, Klima Pro adds:"),
    ("pro.grant.title", "Test access"),
    ("pro.grant.hint", "If your address is on the invite list, enter it: the relay will recognise it."),
    ("pro.grant.field", "Email address"),
    ("pro.grant.check", "Check"),
    ("pro.grant.checking", "Checking…"),
    ("pro.grant.refused", "This address opens nothing for now. It is kept: if the invitation arrives, the tier will follow."),
    ("pro.grant.active", "Test access recognised by the relay."),
    ("pro.grant.forget", "Remove my address"),
    ("search.label", "Search for a town"),
    ("search.locate", "Locate me"),
    ("search.locating", "Locating…"),
    ("search.unsupported", "Geolocation is not available in this browser."),
    ("search.denied", "Location denied. Search for the town instead."),
    ("hourly.title", "Conditions"),
    ("hourly.now", "Now"),
    ("hourly.unfold", "Show all 24 hours"),
    ("hourly.fold", "Show less"),
    ("daily.title", "7-day forecast"),
    ("daily.today", "Today"),
    ("tile.wind", "Wind"),
    ("tile.sunrise", "Sunrise"),
    ("tile.sunrise.caption", "Sunset at {time}."),
    ("language.label", "Language"),
    ("app.loading", "Loading the weather…"),
    ("app.error", "Could not load the forecast."),
    ("app.source", "Open-Meteo data — Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC and JMA models"),
    ("pro.free", "Here and in the app, {parcelles} city and {jours} days of forecast stay free, with no account and no ads."),
    ("pro.grant.web", "Nothing more changes here: cross-checking and air quality are already free, and alerts live in the iPhone app."),
    ("search.placeholder", "{parcelle} — change city"),
    ("rain.axisNow", "Now"),
    ("tile.wind.caption", "Gusts {gusts}."),
    ("tile.feelsLike", "Feels like"),
    ("tile.feelsLike.caption", "The thermometer reads {temperature}."),
    ("tile.humidity", "Humidity"),
    ("tile.humidity.caption", "Dew point {dewPoint}."),
    ("tile.uv", "UV index"),
    ("tile.uv.caption", "Up to {max} today ({level})."),
    ("tile.pressure", "Pressure"),
    ("tile.pressure.caption", "At sea level."),
    ("tile.rainToday", "Rain today"),
    ("tile.rainToday.caption", "Highest chance {probability}."),
    ("tile.air", "Air quality"),
    ("tile.air.caption", "European index {aqi} · fine particles {pm25}."),
    ("tile.pollen", "Pollen"),
    ("tile.pollen.caption", "{grains} grains/m³ — {level}."),
    ("tile.pollen.none", "None"),
    ("tile.pollen.noneCaption", "Nothing notable in the air."),
];

const ES: [(&str, &str); 52] = [
    ("app.retry", "Reintentar"),
    ("app.refresh", "Actualizar"),
    ("app.updatedAt", "actualizado a las {time}"),
    ("app.home", "Volver a la presentación de Klima"),
    ("pro.lead", "En iPhone y Apple Watch, Klima Pro añade:"),
    ("pro.grant.title", "Acceso de prueba"),
    ("pro.grant.hint", "Si su dirección está entre las invitadas, introdúzcala: el relé la reconocerá."),
    ("pro.grant.field", "Dirección de correo"),
    ("pro.grant.check", "Comprobar"),
    ("pro.grant.checking", "Comprobando…"),
    ("pro.grant.refused", "Esta dirección no abre nada por ahora. Queda guardada: si llega la invitación, el plan la seguirá."),
    ("pro.grant.active", "Acceso de prueba reconocido por el relé."),
    ("pro.grant.forget", "Quitar mi dirección"),
    ("search.label", "Buscar un municipio"),
    ("search.locate", "Ubicarme"),
    ("search.locating", "Ubicando…"),
    ("search.unsupported", "La geolocalización no está disponible en este navegador."),
    ("search.denied", "Ubicación denegada. Busque el municipio a mano."),
    ("hourly.title", "Condiciones"),
    ("hourly.now", "Ahora"),
    ("hourly.unfold", "Ver las 24 horas"),
    ("hourly.fold", "Plegar"),
    ("daily.title", "Previsión a 7 días"),
    ("daily.today", "Hoy"),
    ("tile.wind", "Viento"),
    ("tile.sunrise", "Amanecer"),
    ("tile.sunrise.caption", "Anochecer a las {time}."),
    ("language.label", "Idioma"),
    ("app.loading", "Cargando el tiempo…"),
    ("app.error", "No se pudo cargar la previsión."),
    ("app.source", "Datos de Open-Meteo — modelos de Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC y JMA"),
    ("pro.free", "Aquí y en la aplicación, {parcelles} ciudad y {jours} días de previsión siguen siendo gratis, sin cuenta ni publicidad."),
    ("pro.grant.web", "Aquí no cambia nada más: el contraste de fuentes y el aire ya son gratuitos, y las alertas viven en la aplicación de iPhone."),
    ("search.placeholder", "{parcelle} — cambiar de ciudad"),
    ("rain.axisNow", "Ahora"),
    ("tile.wind.caption", "Rachas {gusts}."),
    ("tile.feelsLike", "Sensación"),
    ("tile.feelsLike.caption", "El termómetro marca {temperature}."),
    ("tile.humidity", "Humedad"),
    ("tile.humidity.caption", "Punto de rocío {dewPoint}."),
    ("tile.uv", "Índice UV"),
    ("tile.uv.caption", "Hasta {max} hoy ({level})."),
    ("tile.pressure", "Presión"),
    ("tile.pressure.caption", "Reducida al nivel del mar."),
    ("tile.rainToday", "Lluvia de hoy"),
    ("tile.rainToday.caption", "Probabilidad máxima {probability}."),
    ("tile.air", "Calidad del aire"),
    ("tile.air.caption", "Índice europeo {aqi} · partículas finas {pm25}."),
    ("tile.pollen", "Polen"),
    ("tile.pollen.caption", "{grains} granos/m³ — {level}."),
    ("tile.pollen.none", "Ninguno"),
    ("tile.pollen.noneCaption", "Nada destacable en el aire."),
];

#[cfg(test)]
mod tests {
    use super::*;
    use klima_core::i18n::{LANGUAGES, REFERENCE_LANGUAGE};

    #[test]
    fn les_trois_langues_portent_exactement_les_memes_cles() {
        let reference: Vec<&str> =
            WEB_MESSAGES.catalog(REFERENCE_LANGUAGE).keys().copied().collect();
        for language in LANGUAGES {
            let keys: Vec<&str> = WEB_MESSAGES.catalog(language).keys().copied().collect();
            assert_eq!(keys, reference, "{language}");
        }
    }

    #[test]
    fn aucun_texte_vide() {
        for language in LANGUAGES {
            for (key, value) in WEB_MESSAGES.catalog(language) {
                assert!(!value.trim().is_empty(), "{language} · {key}");
            }
        }
    }
}
