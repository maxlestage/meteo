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

const FR: [(&str, &str); 58] = [
    ("app.loading", "Chargement des données agronomiques…"),
    ("app.retry", "Réessayer"),
    ("app.refresh", "Actualiser"),
    ("app.error", "Impossible de charger la prévision agricole."),
    ("app.updatedAt", "mise à jour {time}"),
    ("app.source", "Données Open-Meteo — modèle agricole : humidité et température du sol, ET0 FAO-56, déficit de pression de vapeur"),
    ("app.home", "Retour à la présentation de Klima"),
    ("pro.lead", "Sur iPhone et Apple Watch, Klima Pro ajoute :"),
    ("pro.free", "Ici et dans l’application, {parcelles} parcelle et {jours} jours de prévision restent gratuits, sans compte ni publicité."),
    ("pro.grant.title", "Accès de test"),
    ("pro.grant.hint", "Si votre adresse fait partie des invitées, saisissez-la : le relais la reconnaîtra."),
    ("pro.grant.field", "Adresse électronique"),
    ("pro.grant.check", "Vérifier"),
    ("pro.grant.checking", "Vérification…"),
    ("pro.grant.refused", "Cette adresse n’ouvre rien pour le moment. Elle est gardée : si l’invitation arrive, le palier suivra."),
    ("pro.grant.active", "Accès de test reconnu par le relais."),
    ("pro.grant.web", "Ici, cela ne change rien de plus : le recoupement est déjà à l’œuvre, et les trois autres fonctions vivent dans l’application iPhone."),
    ("pro.grant.forget", "Retirer mon adresse"),
    ("search.placeholder", "{parcelle} — changer de parcelle"),
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
    ("spray.title", "Fenêtre de traitement"),
    ("spray.none", "Aucune fenêtre sur 7 jours"),
    ("spray.score", "Score {score}/100 sur la plage"),
    ("spray.mainBlocker", "Blocage principal : {blocker}"),
    ("spray.unsuitable", "Conditions défavorables"),
    ("spray.now", "Maintenant"),
    ("spray.plus12", "+12 h"),
    ("spray.plus24", "+24 h"),
    ("tile.soil", "Humidité du sol"),
    ("tile.soil.caption", "{moisture} vol. · {temperature} à 6 cm. {state}"),
    ("tile.water", "Bilan hydrique"),
    ("tile.water.irrigation", "Irrigation conseillée : {amount} sur 7 jours."),
    ("tile.water.caption", "Pluie {rain}, ET0 {et0} sur 7 jours."),
    ("tile.wind", "Vent"),
    ("tile.wind.caption", "Rafales {gusts}. Limite de pulvérisation : {limit}."),
    ("tile.frost", "Risque de gel"),
    ("tile.frost.caption", "Mini {temperature} cette nuit{hoarFrost}."),
    ("tile.disease", "Pression maladie"),
    ("tile.disease.caption", "{hours} h d’humectation du feuillage sur 24 h."),
    ("tile.gdd", "Degrés-jours"),
    ("tile.gdd.caption", "Cumul sur 7 jours, base {base}."),
    ("tile.sunrise", "Lever"),
    ("tile.sunrise.caption", "Coucher à {time}."),
    ("tile.sowing", "Semis"),
    ("tile.sowing.yes", "Possible"),
    ("tile.sowing.no", "Déconseillé"),
    ("tile.sowing.caption", "Sol à {temperature} à 6 cm ; il faut 8 °C et un sol ressuyé."),
    ("language.label", "Langue"),
];

const EN: [(&str, &str); 58] = [
    ("app.loading", "Loading agronomic data…"),
    ("app.retry", "Try again"),
    ("app.refresh", "Refresh"),
    ("app.error", "Could not load the agricultural forecast."),
    ("app.updatedAt", "updated {time}"),
    ("app.source", "Open-Meteo data — agricultural model: soil moisture and temperature, FAO-56 ET0, vapour pressure deficit"),
    ("app.home", "Back to the Klima showcase"),
    ("pro.lead", "On iPhone and Apple Watch, Klima Pro adds:"),
    ("pro.free", "Here and in the app, {parcelles} field and {jours} days of forecast stay free, with no account and no adverts."),
    ("pro.grant.title", "Test access"),
    ("pro.grant.hint", "If your address is on the invite list, enter it: the relay will recognise it."),
    ("pro.grant.field", "Email address"),
    ("pro.grant.check", "Check"),
    ("pro.grant.checking", "Checking…"),
    ("pro.grant.refused", "This address opens nothing for now. It is kept: if the invitation arrives, the tier will follow."),
    ("pro.grant.active", "Test access recognised by the relay."),
    ("pro.grant.web", "It changes nothing further here: cross-checking is already at work, and the other three features live in the iPhone app."),
    ("pro.grant.forget", "Remove my address"),
    ("search.placeholder", "{parcelle} — change field"),
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
    ("spray.title", "Spraying window"),
    ("spray.none", "No window in the next 7 days"),
    ("spray.score", "Score {score}/100 over the window"),
    ("spray.mainBlocker", "Main obstacle: {blocker}"),
    ("spray.unsuitable", "Conditions unsuitable"),
    ("spray.now", "Now"),
    ("spray.plus12", "+12 h"),
    ("spray.plus24", "+24 h"),
    ("tile.soil", "Soil moisture"),
    ("tile.soil.caption", "{moisture} vol. · {temperature} at 6 cm. {state}"),
    ("tile.water", "Water balance"),
    ("tile.water.irrigation", "Irrigation advised: {amount} over 7 days."),
    ("tile.water.caption", "Rain {rain}, ET0 {et0} over 7 days."),
    ("tile.wind", "Wind"),
    ("tile.wind.caption", "Gusts {gusts}. Spraying limit: {limit}."),
    ("tile.frost", "Frost risk"),
    ("tile.frost.caption", "Low of {temperature} tonight{hoarFrost}."),
    ("tile.disease", "Disease pressure"),
    ("tile.disease.caption", "{hours} h of leaf wetness over 24 h."),
    ("tile.gdd", "Growing degree days"),
    ("tile.gdd.caption", "Cumulated over 7 days, base {base}."),
    ("tile.sunrise", "Sunrise"),
    ("tile.sunrise.caption", "Sunset at {time}."),
    ("tile.sowing", "Drilling"),
    ("tile.sowing.yes", "Possible"),
    ("tile.sowing.no", "Not advised"),
    ("tile.sowing.caption", "Soil at {temperature} at 6 cm; it needs 8 °C and drained soil."),
    ("language.label", "Language"),
];

const ES: [(&str, &str); 58] = [
    ("app.loading", "Cargando los datos agronómicos…"),
    ("app.retry", "Reintentar"),
    ("app.refresh", "Actualizar"),
    ("app.error", "No se ha podido cargar la previsión agrícola."),
    ("app.updatedAt", "actualizado a las {time}"),
    ("app.source", "Datos de Open-Meteo — modelo agrícola: humedad y temperatura del suelo, ET0 FAO-56, déficit de presión de vapor"),
    ("app.home", "Volver a la presentación de Klima"),
    ("pro.lead", "En iPhone y Apple Watch, Klima Pro añade:"),
    ("pro.free", "Aquí y en la aplicación, {parcelles} parcela y {jours} días de previsión siguen siendo gratis, sin cuenta ni publicidad."),
    ("pro.grant.title", "Acceso de prueba"),
    ("pro.grant.hint", "Si su dirección está entre las invitadas, introdúzcala: el relé la reconocerá."),
    ("pro.grant.field", "Dirección de correo"),
    ("pro.grant.check", "Comprobar"),
    ("pro.grant.checking", "Comprobando…"),
    ("pro.grant.refused", "Esta dirección no abre nada por ahora. Queda guardada: si llega la invitación, el plan la seguirá."),
    ("pro.grant.active", "Acceso de prueba reconocido por el relé."),
    ("pro.grant.web", "Aquí no cambia nada más: el contraste ya está en marcha, y las otras tres funciones viven en la aplicación de iPhone."),
    ("pro.grant.forget", "Quitar mi dirección"),
    ("search.placeholder", "{parcelle} — cambiar de parcela"),
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
    ("spray.title", "Ventana de tratamiento"),
    ("spray.none", "Ninguna ventana en 7 días"),
    ("spray.score", "Puntuación {score}/100 en la franja"),
    ("spray.mainBlocker", "Principal impedimento: {blocker}"),
    ("spray.unsuitable", "Condiciones desfavorables"),
    ("spray.now", "Ahora"),
    ("spray.plus12", "+12 h"),
    ("spray.plus24", "+24 h"),
    ("tile.soil", "Humedad del suelo"),
    ("tile.soil.caption", "{moisture} vol. · {temperature} a 6 cm. {state}"),
    ("tile.water", "Balance hídrico"),
    ("tile.water.irrigation", "Riego aconsejado: {amount} en 7 días."),
    ("tile.water.caption", "Lluvia {rain}, ET0 {et0} en 7 días."),
    ("tile.wind", "Viento"),
    ("tile.wind.caption", "Rachas {gusts}. Límite de pulverización: {limit}."),
    ("tile.frost", "Riesgo de helada"),
    ("tile.frost.caption", "Mínima de {temperature} esta noche{hoarFrost}."),
    ("tile.disease", "Presión de enfermedad"),
    ("tile.disease.caption", "{hours} h de humectación foliar en 24 h."),
    ("tile.gdd", "Grados-día"),
    ("tile.gdd.caption", "Acumulado en 7 días, base {base}."),
    ("tile.sunrise", "Amanecer"),
    ("tile.sunrise.caption", "Anochecer a las {time}."),
    ("tile.sowing", "Siembra"),
    ("tile.sowing.yes", "Posible"),
    ("tile.sowing.no", "Desaconsejada"),
    ("tile.sowing.caption", "Suelo a {temperature} a 6 cm; hacen falta 8 °C y suelo oreado."),
    ("language.label", "Idioma"),
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
