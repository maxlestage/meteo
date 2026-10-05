//! Catalogue des libellés partagés.
//!
//! Ce que le domaine décrit par des clés — temps qu'il fait, pluie à venir,
//! conseils, UV, qualité de l'air, pollens, alertes, paliers — dans les trois
//! langues de Klima. Les textes propres à chaque interface vivent chez elle.
//!
//! Les tables sont écrites à plat et montées une seule fois : un catalogue ne
//! change pas en cours d'exécution, et les trois langues portent exactement
//! les mêmes clés — un test le vérifie, parce qu'une clé oubliée dans une
//! langue ne se voit qu'une fois le texte affiché à quelqu'un.

use std::sync::LazyLock;

use crate::i18n::{Catalog, MessageSet};

/// Les libellés partagés, dans les trois langues.
pub static SHARED_MESSAGES: LazyLock<MessageSet> = LazyLock::new(|| MessageSet {
    fr: catalog(&FR),
    en: catalog(&EN),
    es: catalog(&ES),
});

fn catalog(entries: &[(&'static str, &'static str)]) -> Catalog {
    entries.iter().copied().collect()
}

const FR: [(&str, &str); 125] = [
    ("wmo.clearSky", "Ciel dégagé"),
    ("wmo.mainlyClear", "Peu nuageux"),
    ("wmo.partlyCloudy", "Partiellement nuageux"),
    ("wmo.overcast", "Couvert"),
    ("wmo.fog", "Brouillard"),
    ("wmo.rimeFog", "Brouillard givrant"),
    ("wmo.lightDrizzle", "Bruine légère"),
    ("wmo.drizzle", "Bruine"),
    ("wmo.denseDrizzle", "Bruine dense"),
    ("wmo.freezingDrizzle", "Bruine verglaçante"),
    ("wmo.denseFreezingDrizzle", "Bruine verglaçante dense"),
    ("wmo.slightRain", "Pluie faible"),
    ("wmo.rain", "Pluie"),
    ("wmo.heavyRain", "Pluie forte"),
    ("wmo.freezingRain", "Pluie verglaçante"),
    ("wmo.heavyFreezingRain", "Pluie verglaçante forte"),
    ("wmo.slightSnow", "Neige faible"),
    ("wmo.snow", "Neige"),
    ("wmo.heavySnow", "Neige forte"),
    ("wmo.snowGrains", "Grains de neige"),
    ("wmo.showers", "Averses"),
    ("wmo.moderateShowers", "Averses modérées"),
    ("wmo.violentShowers", "Averses violentes"),
    ("wmo.snowShowers", "Averses de neige"),
    ("wmo.heavySnowShowers", "Averses de neige fortes"),
    ("wmo.thunderstorm", "Orage"),
    ("wmo.thunderstormHail", "Orage et grêle"),
    ("wmo.thunderstormHeavyHail", "Orage et forte grêle"),
    ("api.unreachable", "Service météo injoignable. Vérifiez votre connexion."),
    ("api.status", "Le service météo a répondu {status}."),
    ("api.malformed", "Réponse illisible du service météo."),
    ("consensus.title", "Accord des sources"),
    ("consensus.forte", "Fort"),
    ("consensus.moyenne", "Moyen"),
    ("consensus.faible", "Faible"),
    ("consensus.detail", "{count} sources · écart {spread}"),
    ("consensus.rainDisagreement", "désaccord sur la pluie"),
    ("consensus.unavailable", "Comparaison indisponible"),
    ("consensus.median", "Valeur retenue : {value}"),
    ("sources.title", "Ce que dit chaque source"),
    ("sources.dry", "sec"),
    ("plan.libre", "Klima"),
    ("plan.pro", "Klima Pro"),
    ("plan.reason.recoupement", "Comparer plusieurs instituts demande l’abonnement."),
    ("plan.reason.alertes", "Être prévenu sans ouvrir l’application demande l’abonnement."),
    ("search.myField", "Ma ville"),
    ("rain.title", "Pluie"),
    ("rain.none", "Pas de pluie d’ici {hours} h"),
    ("rain.now", "Il pleut — accalmie vers {time}"),
    ("rain.nowLasting", "Il pleut, et pour un moment"),
    ("rain.soon", "Pluie vers {time}"),
    ("rain.detail", "{probability} de risque · {amount}"),
    ("advice.title", "À emporter"),
    ("advice.none", "Rien de particulier : profitez-en."),
    ("advice.parapluie", "Un parapluie"),
    ("advice.manteau", "Un manteau"),
    ("advice.cremeSolaire", "De la crème solaire"),
    ("advice.lunettes", "Des lunettes de soleil"),
    ("advice.hydratation", "De l’eau : il va faire chaud"),
    ("advice.gel", "Prudence : trottoirs gelés possibles"),
    ("advice.vent", "Rafales : gare au parapluie"),
    ("uv.faible", "Faible"),
    ("uv.modere", "Modéré"),
    ("uv.eleve", "Élevé"),
    ("uv.tresEleve", "Très élevé"),
    ("uv.extreme", "Extrême"),
    ("air.bonne", "Bonne"),
    ("air.correcte", "Correcte"),
    ("air.moyenne", "Moyenne"),
    ("air.mediocre", "Médiocre"),
    ("air.tresMediocre", "Très médiocre"),
    ("air.extremementMediocre", "Extrêmement médiocre"),
    ("pollen.aulne", "Aulne"),
    ("pollen.bouleau", "Bouleau"),
    ("pollen.graminees", "Graminées"),
    ("pollen.armoise", "Armoise"),
    ("pollen.olivier", "Olivier"),
    ("pollen.ambroisie", "Ambroisie"),
    ("pollenLevel.faible", "Faible"),
    ("pollenLevel.modere", "Modéré"),
    ("pollenLevel.eleve", "Élevé"),
    ("pollenLevel.tresEleve", "Très élevé"),
    ("alert.pluie.title", "Pluie imminente"),
    ("alert.pluie.body", "Elle arrive d’ici peu : {probability} % de risque."),
    ("alert.orage.title", "Orage en approche"),
    ("alert.orage.body", "Un orage est prévu dans les prochaines heures."),
    ("alert.gel.title", "Gel annoncé"),
    ("alert.gel.body", "Jusqu’à {temperature} °C : trottoirs et pare-brise gelés."),
    ("alert.chaleur.title", "Forte chaleur"),
    ("alert.chaleur.body", "Jusqu’à {temperature} °C : buvez, cherchez l’ombre."),
    ("alert.vent.title", "Vent violent"),
    ("alert.vent.body", "Rafales jusqu’à {gusts} km/h."),
    ("plan.reason.air", "La qualité de l’air et les pollens demandent l’abonnement."),
    ("plan.feature.recoupement", "Six sources recoupées, et leur niveau d’accord"),
    ("plan.feature.alertes", "Prévenu de la pluie, de l’orage, du gel et de la chaleur"),
    ("plan.feature.air", "Qualité de l’air et pollens"),
    ("veille.title", "Le guetteur"),
    ("veille.subtitle", "Il relit le ciel tous les quarts d’heure."),
    ("veille.now", "Les 30 minutes en cours"),
    ("veille.next", "Les 2 heures à venir"),
    ("veille.now.dry", "Pas une goutte d’ici une demi-heure."),
    ("veille.now.starts", "À partir de {time} : {kind}."),
    ("veille.now.continues", "Ça tombe, et toute la demi-heure : {kind}."),
    ("veille.now.stops", "Ça tombe encore, mais ça s’arrête vers {time}."),
    ("veille.next.dry", "Sec jusqu’à {end} au moins."),
    ("veille.next.episode", "Entre {start} et {end} : {kind}, {amount}."),
    ("veille.next.episodeOpen", "À partir de {start} : {kind}, et encore après {end}."),
    ("veille.next.persists", "Ça ne s’arrête pas avant {end} : {kind}, {amount} en tout."),
    ("veille.next.lull", "Fin vers {time}, puis sec jusqu’à {end}."),
    ("veille.next.lullReturn", "Fin vers {time}, mais ça reprend vers {back}."),
    ("veille.gusts", "Rafales jusqu’à {gusts} vers {time}."),
    ("veille.temperature", "{temp} à {time}, ressenti {feels}."),
    ("veille.checked", "Relu à {time} · prochaine lecture à {next}"),
    ("veille.unavailable", "Pas de prévision au quart d’heure pour cette ville en ce moment."),
    ("veille.loading", "Le guetteur regarde le ciel…"),
    ("veille.chart", "Précipitations au quart d’heure, sur deux heures"),
    ("veille.kind.pluie.faible", "pluie faible"),
    ("veille.kind.pluie.moderee", "pluie modérée"),
    ("veille.kind.pluie.forte", "forte pluie"),
    ("veille.kind.neige.faible", "neige faible"),
    ("veille.kind.neige.moderee", "neige modérée"),
    ("veille.kind.neige.forte", "forte neige"),
    ("veille.kind.orage", "orage"),
    ("plan.reason.villes", "Enregistrer plusieurs villes demande l’abonnement."),
    ("plan.feature.villes", "Plusieurs villes enregistrées, de l’une à l’autre d’un geste"),
];

const EN: [(&str, &str); 125] = [
    ("wmo.clearSky", "Clear sky"),
    ("wmo.mainlyClear", "Mainly clear"),
    ("wmo.partlyCloudy", "Partly cloudy"),
    ("wmo.overcast", "Overcast"),
    ("wmo.fog", "Fog"),
    ("wmo.rimeFog", "Freezing fog"),
    ("wmo.lightDrizzle", "Light drizzle"),
    ("wmo.drizzle", "Drizzle"),
    ("wmo.denseDrizzle", "Heavy drizzle"),
    ("wmo.freezingDrizzle", "Freezing drizzle"),
    ("wmo.denseFreezingDrizzle", "Heavy freezing drizzle"),
    ("wmo.slightRain", "Light rain"),
    ("wmo.rain", "Rain"),
    ("wmo.heavyRain", "Heavy rain"),
    ("wmo.freezingRain", "Freezing rain"),
    ("wmo.heavyFreezingRain", "Heavy freezing rain"),
    ("wmo.slightSnow", "Light snow"),
    ("wmo.snow", "Snow"),
    ("wmo.heavySnow", "Heavy snow"),
    ("wmo.snowGrains", "Snow grains"),
    ("wmo.showers", "Showers"),
    ("wmo.moderateShowers", "Moderate showers"),
    ("wmo.violentShowers", "Violent showers"),
    ("wmo.snowShowers", "Snow showers"),
    ("wmo.heavySnowShowers", "Heavy snow showers"),
    ("wmo.thunderstorm", "Thunderstorm"),
    ("wmo.thunderstormHail", "Thunderstorm with hail"),
    ("wmo.thunderstormHeavyHail", "Thunderstorm with heavy hail"),
    ("api.unreachable", "Weather service unreachable. Check your connection."),
    ("api.status", "The weather service replied {status}."),
    ("api.malformed", "Unreadable response from the weather service."),
    ("consensus.title", "Source agreement"),
    ("consensus.forte", "Strong"),
    ("consensus.moyenne", "Moderate"),
    ("consensus.faible", "Weak"),
    ("consensus.detail", "{count} sources · {spread} apart"),
    ("consensus.rainDisagreement", "they disagree on rain"),
    ("consensus.unavailable", "Comparison unavailable"),
    ("consensus.median", "Value used: {value}"),
    ("sources.title", "What each source says"),
    ("sources.dry", "dry"),
    ("plan.libre", "Klima"),
    ("plan.pro", "Klima Pro"),
    ("plan.reason.recoupement", "Comparing several institutes needs the subscription."),
    ("plan.reason.alertes", "Being warned without opening the app needs the subscription."),
    ("search.myField", "My city"),
    ("rain.title", "Rain"),
    ("rain.none", "No rain for {hours} h"),
    ("rain.now", "Raining — easing around {time}"),
    ("rain.nowLasting", "Raining, and set to last"),
    ("rain.soon", "Rain around {time}"),
    ("rain.detail", "{probability} chance · {amount}"),
    ("advice.title", "Take with you"),
    ("advice.none", "Nothing special: enjoy it."),
    ("advice.parapluie", "An umbrella"),
    ("advice.manteau", "A coat"),
    ("advice.cremeSolaire", "Sunscreen"),
    ("advice.lunettes", "Sunglasses"),
    ("advice.hydratation", "Water: it will be hot"),
    ("advice.gel", "Careful: icy pavements possible"),
    ("advice.vent", "Gusts: mind your umbrella"),
    ("uv.faible", "Low"),
    ("uv.modere", "Moderate"),
    ("uv.eleve", "High"),
    ("uv.tresEleve", "Very high"),
    ("uv.extreme", "Extreme"),
    ("air.bonne", "Good"),
    ("air.correcte", "Fair"),
    ("air.moyenne", "Moderate"),
    ("air.mediocre", "Poor"),
    ("air.tresMediocre", "Very poor"),
    ("air.extremementMediocre", "Extremely poor"),
    ("pollen.aulne", "Alder"),
    ("pollen.bouleau", "Birch"),
    ("pollen.graminees", "Grass"),
    ("pollen.armoise", "Mugwort"),
    ("pollen.olivier", "Olive"),
    ("pollen.ambroisie", "Ragweed"),
    ("pollenLevel.faible", "Low"),
    ("pollenLevel.modere", "Moderate"),
    ("pollenLevel.eleve", "High"),
    ("pollenLevel.tresEleve", "Very high"),
    ("alert.pluie.title", "Rain on its way"),
    ("alert.pluie.body", "Arriving shortly: {probability}% chance."),
    ("alert.orage.title", "Thunderstorm approaching"),
    ("alert.orage.body", "A thunderstorm is expected in the coming hours."),
    ("alert.gel.title", "Frost ahead"),
    ("alert.gel.body", "Down to {temperature} °C: icy pavements and windscreens."),
    ("alert.chaleur.title", "Intense heat"),
    ("alert.chaleur.body", "Up to {temperature} °C: drink, find shade."),
    ("alert.vent.title", "Strong wind"),
    ("alert.vent.body", "Gusts up to {gusts} km/h."),
    ("plan.reason.air", "Air quality and pollen require a subscription."),
    ("plan.feature.recoupement", "Six sources cross-checked, and how far they agree"),
    ("plan.feature.alertes", "Warned of rain, storms, frost and heat"),
    ("plan.feature.air", "Air quality and pollen"),
    ("veille.title", "The lookout"),
    ("veille.subtitle", "It rereads the sky every fifteen minutes."),
    ("veille.now", "The next 30 minutes"),
    ("veille.next", "The next 2 hours"),
    ("veille.now.dry", "Not a drop for the next half hour."),
    ("veille.now.starts", "From {time}: {kind}."),
    ("veille.now.continues", "It’s coming down, and will for the whole half hour: {kind}."),
    ("veille.now.stops", "Still coming down, but it stops around {time}."),
    ("veille.next.dry", "Dry until at least {end}."),
    ("veille.next.episode", "Between {start} and {end}: {kind}, {amount}."),
    ("veille.next.episodeOpen", "From {start}: {kind}, still going after {end}."),
    ("veille.next.persists", "No let-up before {end}: {kind}, {amount} in all."),
    ("veille.next.lull", "Ends around {time}, then dry until {end}."),
    ("veille.next.lullReturn", "Ends around {time}, but it comes back around {back}."),
    ("veille.gusts", "Gusts up to {gusts} around {time}."),
    ("veille.temperature", "{temp} at {time}, feels like {feels}."),
    ("veille.checked", "Checked at {time} · next check at {next}"),
    ("veille.unavailable", "No quarter-hourly forecast for this city right now."),
    ("veille.loading", "The lookout is checking the sky…"),
    ("veille.chart", "Precipitation every fifteen minutes, over two hours"),
    ("veille.kind.pluie.faible", "light rain"),
    ("veille.kind.pluie.moderee", "moderate rain"),
    ("veille.kind.pluie.forte", "heavy rain"),
    ("veille.kind.neige.faible", "light snow"),
    ("veille.kind.neige.moderee", "moderate snow"),
    ("veille.kind.neige.forte", "heavy snow"),
    ("veille.kind.orage", "thunderstorm"),
    ("plan.reason.villes", "Saving several cities needs the subscription."),
    ("plan.feature.villes", "Several saved cities, one tap from each other"),
];

const ES: [(&str, &str); 125] = [
    ("wmo.clearSky", "Cielo despejado"),
    ("wmo.mainlyClear", "Poco nuboso"),
    ("wmo.partlyCloudy", "Parcialmente nuboso"),
    ("wmo.overcast", "Cubierto"),
    ("wmo.fog", "Niebla"),
    ("wmo.rimeFog", "Niebla helada"),
    ("wmo.lightDrizzle", "Llovizna débil"),
    ("wmo.drizzle", "Llovizna"),
    ("wmo.denseDrizzle", "Llovizna intensa"),
    ("wmo.freezingDrizzle", "Llovizna engelante"),
    ("wmo.denseFreezingDrizzle", "Llovizna engelante intensa"),
    ("wmo.slightRain", "Lluvia débil"),
    ("wmo.rain", "Lluvia"),
    ("wmo.heavyRain", "Lluvia fuerte"),
    ("wmo.freezingRain", "Lluvia engelante"),
    ("wmo.heavyFreezingRain", "Lluvia engelante fuerte"),
    ("wmo.slightSnow", "Nieve débil"),
    ("wmo.snow", "Nieve"),
    ("wmo.heavySnow", "Nieve fuerte"),
    ("wmo.snowGrains", "Cinarra"),
    ("wmo.showers", "Chubascos"),
    ("wmo.moderateShowers", "Chubascos moderados"),
    ("wmo.violentShowers", "Chubascos violentos"),
    ("wmo.snowShowers", "Chubascos de nieve"),
    ("wmo.heavySnowShowers", "Chubascos de nieve fuertes"),
    ("wmo.thunderstorm", "Tormenta"),
    ("wmo.thunderstormHail", "Tormenta con granizo"),
    ("wmo.thunderstormHeavyHail", "Tormenta con granizo fuerte"),
    ("api.unreachable", "Servicio meteorológico inaccesible. Compruebe su conexión."),
    ("api.status", "El servicio meteorológico ha respondido {status}."),
    ("api.malformed", "Respuesta ilegible del servicio meteorológico."),
    ("consensus.title", "Acuerdo de las fuentes"),
    ("consensus.forte", "Fuerte"),
    ("consensus.moyenne", "Medio"),
    ("consensus.faible", "Débil"),
    ("consensus.detail", "{count} fuentes · diferencia de {spread}"),
    ("consensus.rainDisagreement", "discrepan sobre la lluvia"),
    ("consensus.unavailable", "Comparación no disponible"),
    ("consensus.median", "Valor retenido: {value}"),
    ("sources.title", "Lo que dice cada fuente"),
    ("sources.dry", "seco"),
    ("plan.libre", "Klima"),
    ("plan.pro", "Klima Pro"),
    ("plan.reason.recoupement", "Comparar varios institutos requiere la suscripción."),
    ("plan.reason.alertes", "Recibir avisos sin abrir la aplicación requiere la suscripción."),
    ("search.myField", "Mi ciudad"),
    ("rain.title", "Lluvia"),
    ("rain.none", "Sin lluvia en {hours} h"),
    ("rain.now", "Llueve — amaina hacia las {time}"),
    ("rain.nowLasting", "Llueve, y para rato"),
    ("rain.soon", "Lluvia hacia las {time}"),
    ("rain.detail", "{probability} de probabilidad · {amount}"),
    ("advice.title", "Para llevar"),
    ("advice.none", "Nada especial: aprovéchelo."),
    ("advice.parapluie", "Un paraguas"),
    ("advice.manteau", "Un abrigo"),
    ("advice.cremeSolaire", "Crema solar"),
    ("advice.lunettes", "Gafas de sol"),
    ("advice.hydratation", "Agua: va a hacer calor"),
    ("advice.gel", "Cuidado: aceras heladas posibles"),
    ("advice.vent", "Rachas: cuidado con el paraguas"),
    ("uv.faible", "Bajo"),
    ("uv.modere", "Moderado"),
    ("uv.eleve", "Alto"),
    ("uv.tresEleve", "Muy alto"),
    ("uv.extreme", "Extremo"),
    ("air.bonne", "Buena"),
    ("air.correcte", "Razonable"),
    ("air.moyenne", "Regular"),
    ("air.mediocre", "Mala"),
    ("air.tresMediocre", "Muy mala"),
    ("air.extremementMediocre", "Extremadamente mala"),
    ("pollen.aulne", "Aliso"),
    ("pollen.bouleau", "Abedul"),
    ("pollen.graminees", "Gramíneas"),
    ("pollen.armoise", "Artemisa"),
    ("pollen.olivier", "Olivo"),
    ("pollen.ambroisie", "Ambrosía"),
    ("pollenLevel.faible", "Bajo"),
    ("pollenLevel.modere", "Moderado"),
    ("pollenLevel.eleve", "Alto"),
    ("pollenLevel.tresEleve", "Muy alto"),
    ("alert.pluie.title", "Lluvia inminente"),
    ("alert.pluie.body", "Llega en breve: {probability} % de probabilidad."),
    ("alert.orage.title", "Tormenta en camino"),
    ("alert.orage.body", "Se espera una tormenta en las próximas horas."),
    ("alert.gel.title", "Helada prevista"),
    ("alert.gel.body", "Hasta {temperature} °C: aceras y parabrisas helados."),
    ("alert.chaleur.title", "Calor intenso"),
    ("alert.chaleur.body", "Hasta {temperature} °C: beba agua, busque la sombra."),
    ("alert.vent.title", "Viento fuerte"),
    ("alert.vent.body", "Rachas de hasta {gusts} km/h."),
    ("plan.reason.air", "La calidad del aire y el polen requieren la suscripción."),
    ("plan.feature.recoupement", "Seis fuentes contrastadas, y su grado de acuerdo"),
    ("plan.feature.alertes", "Avisos de lluvia, tormenta, helada y calor"),
    ("plan.feature.air", "Calidad del aire y polen"),
    ("veille.title", "El vigía"),
    ("veille.subtitle", "Relee el cielo cada cuarto de hora."),
    ("veille.now", "Los próximos 30 minutos"),
    ("veille.next", "Las próximas 2 horas"),
    ("veille.now.dry", "Ni una gota en la próxima media hora."),
    ("veille.now.starts", "A partir de las {time}: {kind}."),
    ("veille.now.continues", "Está cayendo, y seguirá toda la media hora: {kind}."),
    ("veille.now.stops", "Todavía cae, pero para hacia las {time}."),
    ("veille.next.dry", "Seco hasta al menos las {end}."),
    ("veille.next.episode", "Entre las {start} y las {end}: {kind}, {amount}."),
    ("veille.next.episodeOpen", "A partir de las {start}: {kind}, y sigue después de las {end}."),
    ("veille.next.persists", "No para antes de las {end}: {kind}, {amount} en total."),
    ("veille.next.lull", "Termina hacia las {time}, luego seco hasta las {end}."),
    ("veille.next.lullReturn", "Termina hacia las {time}, pero vuelve hacia las {back}."),
    ("veille.gusts", "Rachas de hasta {gusts} hacia las {time}."),
    ("veille.temperature", "{temp} a las {time}, sensación de {feels}."),
    ("veille.checked", "Leído a las {time} · próxima lectura a las {next}"),
    ("veille.unavailable", "No hay previsión por cuartos de hora para esta ciudad ahora mismo."),
    ("veille.loading", "El vigía está mirando el cielo…"),
    ("veille.chart", "Precipitación cada cuarto de hora, durante dos horas"),
    ("veille.kind.pluie.faible", "lluvia débil"),
    ("veille.kind.pluie.moderee", "lluvia moderada"),
    ("veille.kind.pluie.forte", "lluvia fuerte"),
    ("veille.kind.neige.faible", "nieve débil"),
    ("veille.kind.neige.moderee", "nieve moderada"),
    ("veille.kind.neige.forte", "nevada fuerte"),
    ("veille.kind.orage", "tormenta"),
    ("plan.reason.villes", "Guardar varias ciudades requiere la suscripción."),
    ("plan.feature.villes", "Varias ciudades guardadas, a un toque una de otra"),
];

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{LANGUAGES, Language, REFERENCE_LANGUAGE};
    use crate::weather::weather_condition;

    #[test]
    fn les_trois_langues_portent_exactement_les_memes_cles() {
        let reference: Vec<&str> =
            SHARED_MESSAGES.catalog(REFERENCE_LANGUAGE).keys().copied().collect();
        for language in LANGUAGES {
            let keys: Vec<&str> = SHARED_MESSAGES.catalog(language).keys().copied().collect();
            assert_eq!(keys, reference, "{language}");
        }
    }

    #[test]
    fn aucun_texte_vide() {
        for language in LANGUAGES {
            for (key, value) in SHARED_MESSAGES.catalog(language) {
                assert!(!value.trim().is_empty(), "{language} · {key}");
            }
        }
    }

    #[test]
    fn un_motif_a_trous_a_les_memes_trous_partout() {
        for (key, modele) in SHARED_MESSAGES.catalog(REFERENCE_LANGUAGE) {
            let reference = jetons(modele);
            for language in LANGUAGES {
                let value = SHARED_MESSAGES.get(language, key).unwrap();
                assert_eq!(jetons(value), reference, "{language} · {key}");
            }
        }
    }


    #[test]
    fn tout_code_wmo_documente_est_traduit_dans_les_trois_langues() {
        let codes = [
            0, 1, 2, 3, 45, 48, 51, 53, 55, 56, 57, 61, 63, 65, 66, 67, 71, 73, 75, 77, 80, 81,
            82, 85, 86, 95, 96, 99,
        ];
        for code in codes {
            let key = weather_condition(code).label_key;
            for language in LANGUAGES {
                assert!(SHARED_MESSAGES.get(language, key).is_some(), "{language} · {key}");
            }
        }
    }

    #[test]
    fn les_libelles_partages_disent_klima_jamais_kliima() {
        // Deux noms, et ce n'est pas une coquille : le triangle et le second
        // « i » ne vivent que dans le catalogue iOS.
        for language in LANGUAGES {
            for (key, value) in SHARED_MESSAGES.catalog(language) {
                assert!(!value.contains("Kliima"), "{language} · {key}");
                assert!(!value.contains('\u{2023}'), "{language} · {key}");
            }
        }
        assert_eq!(SHARED_MESSAGES.get(Language::Fr, "plan.libre"), Some("Klima"));
        assert_eq!(SHARED_MESSAGES.get(Language::Fr, "plan.pro"), Some("Klima Pro"));
    }

    /// Les noms des trous d'un motif, triés.
    fn jetons(modele: &str) -> Vec<String> {
        let mut noms = Vec::new();
        let mut rest = modele;
        while let Some(start) = rest.find('{') {
            let after = &rest[start + 1..];
            let len = after
                .char_indices()
                .find(|(_, c)| !(c.is_ascii_alphanumeric() || *c == '_'))
                .map_or(after.len(), |(index, _)| index);
            if len > 0 && after[len..].starts_with('}') {
                noms.push(after[..len].to_owned());
                rest = &after[len + 1..];
            } else {
                rest = after;
            }
        }
        noms.sort();
        noms
    }
}
