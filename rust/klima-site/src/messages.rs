//! Textes du site de présentation.
//!
//! `klima_core::messages` : le traducteur lit les deux catalogues, celui du
//! site d'abord.

use std::sync::LazyLock;

use klima_core::i18n::{Catalog, MessageSet};

pub static SITE_MESSAGES: LazyLock<MessageSet> = LazyLock::new(|| MessageSet {
    fr: catalog(&FR),
    en: catalog(&EN),
    es: catalog(&ES),
});

fn catalog(entries: &[(&'static str, &'static str)]) -> Catalog {
    entries.iter().copied().collect()
}

const FR: [(&str, &str); 116] = [
    ("nav.today", "Météo du jour"),
    ("nav.data", "Données"),
    ("app.open", "Ouvrir l’application"),
    ("hero.cta.today", "Voir la météo du jour"),
    ("today.title", "La météo du jour"),
    ("today.loading", "Chargement de la journée…"),
    ("today.retry", "Réessayer"),
    ("today.error", "Impossible de charger la météo du jour."),
    ("today.rain", "Pluie"),
    ("today.rain.detail", "{probability} de probabilité"),
    ("today.gusts", "Rafales"),
    ("today.gusts.detail", "Maximum de la journée"),
    ("today.sunrise", "Lever"),
    ("today.sunrise.detail", "Coucher à {time}"),
    ("search.label", "Rechercher une commune"),
    ("data.title", "D’où viennent les chiffres"),
    ("data.rules.title", "Les mêmes règles sur les deux plateformes"),
    ("phone.conditions", "Conditions météo"),
    ("phone.loading", "Chargement"),
    ("nav.sources", "Sources"),
    ("sources.title", "Plusieurs fournisseurs, une seule réponse"),
    ("sources.method", "Comment se lit l’accord"),
    ("sources.methodBody", "La valeur retenue est la médiane, moins sensible qu’une moyenne à un modèle isolé. L’accord est jugé fort quand les modèles tiennent dans 1,5 °C et s’entendent sur la pluie ; faible au-delà de 3 °C d’écart. Quand ils divergent, Klima le dit plutôt que d’afficher une fausse précision."),
    ("sources.loading", "Recoupement en cours…"),
    ("sources.now", "Température à l’heure en cours"),
    ("sources.answered", "{answered} fournisseurs sur {queried} ont répondu"),
    ("footer.product", "Le produit"),
    ("footer.app", "Application web"),
    ("footer.dataTitle", "Les données"),
    ("footer.credits", "Crédits"),
    ("footer.author", "Maxime Nathan Lestage"),
    ("footer.role", "Conception et développement"),
    ("footer.rights", "© {year} Klima — Maxime Nathan Lestage. Tous droits réservés."),
    ("language.label", "Langue"),
    ("nav.indicators", "Ce que Klima dit"),
    ("hero.eyebrow", "iPhone, Apple Watch et web"),
    ("hero.title", "La météo de votre ville, dite simplement."),
    ("hero.lead", "Va-t-il pleuvoir, et quand ? Que faut-il emporter ? Le soleil tape-t-il, l’air est-il bon ? Klima répond aux questions qu’on se pose avant de sortir, à partir de neuf sources recoupées."),
    ("hero.cta.indicators", "Ce que Klima vous dit"),
    ("hero.note", "Données Open-Meteo et Copernicus, sans compte ni clé d’API. Gratuit, sans publicité."),
    ("today.lead", "Essayez sur votre ville. Le site s’en tient à aujourd’hui ; la semaine, l’heure par heure et les alertes sont dans l’application."),
    ("today.uv", "Indice UV"),
    ("today.next", "Pluie à venir"),
    ("today.next.detail", "Sur les douze prochaines heures"),
    ("today.advice.none", "Rien de particulier"),
    ("today.advice.detail", "Profitez-en."),
    ("today.feels", "Ressenti"),
    ("today.feels.detail", "{temperature} au thermomètre"),
    ("today.sun", "Soleil"),
    ("today.sun.detail", "Indice {index} au plus fort de la journée"),
    ("search.placeholder", "{commune} — changer de ville"),
    ("features.title", "Sept réponses avant de sortir"),
    ("features.lead", "Ni carte radar à déchiffrer, ni tableau de chiffres : Klima part de la prévision et rend des réponses — à quelle heure, quoi emporter, faut-il se méfier."),
    ("feature.rain.title", "Pluie à venir"),
    ("feature.rain.rule", "Une heure est pluvieuse dès {amount} ou {probability} de risque."),
    ("feature.rain.detail", "Klima dit quand la pluie arrive, combien elle versera et quand elle cesse, sur les {hours} prochaines heures."),
    ("feature.advice.title", "À emporter"),
    ("feature.advice.rule", "Un parapluie s’il pleut, un manteau sous {coat} ressentis, des lunettes dès l’indice UV {glasses}, de la crème dès {sunscreen}."),
    ("feature.advice.detail", "De l’eau quand il fait chaud, prudence quand ça gèle, et gare au parapluie quand les rafales forcissent."),
    ("feature.uv.title", "Indice UV"),
    ("feature.uv.rule", "L’échelle de l’Organisation mondiale de la santé, de faible à extrême."),
    ("feature.uv.detail", "L’indice est arrondi avant d’être classé, comme sur les bulletins : 2,6 se lit 3, et se dit « modéré »."),
    ("feature.air.title", "Qualité de l’air"),
    ("feature.air.rule", "L’indice européen et ses six classes, de bonne à extrêmement médiocre."),
    ("feature.air.detail", "Particules fines, dioxyde d’azote et ozone, prévus par Copernicus pour l’heure en cours."),
    ("feature.pollen.title", "Pollens"),
    ("feature.pollen.rule", "Aulne, bouleau, graminées, armoise, olivier, ambroisie."),
    ("feature.pollen.detail", "Le plus présent est nommé, avec son intensité. Prévus en Europe seulement : ailleurs, Klima n’en parle pas."),
    ("feature.alerts.title", "Alertes"),
    ("feature.alerts.rule", "La pluie dans les deux heures, l’orage, le gel, la chaleur dès {heat}, les rafales dès {gusts}."),
    ("feature.alerts.detail", "Jamais entre 22 h et 7 h, jamais deux fois de suite pour la même raison. Dans l’application iPhone."),
    ("figure.uv", "Dès l’indice {sunscreen}, Klima conseille la crème solaire."),
    ("data.model.title", "Les modèles des grands instituts"),
    ("data.model.body", "La prévision est recoupée, heure par heure, de neuf sources : les modèles de Météo-France, de l’ECMWF, du DWD, de la NOAA, du Met Office, d’Environnement Canada et de l’agence japonaise, redistribués par Open-Meteo, celui de MET Norway et une station du DWD. Klima retient la médiane — un modèle qui s’égare ne déplace rien — et tire le risque de pluie de la part des sources qui mouillent ; l’air et les pollens, des prévisions européennes de Copernicus. Klima n’en demande que ce qu’une ville regarde : pluie et risque, ressenti, vent et rafales, UV, pression."),
    ("data.rules.body", "Les seuils vivent au même endroit pour le web et pour iOS, et les deux suites de tests couvrent les mêmes cas. La réponse est la même, quel que soit l’écran par lequel on la lit."),
    ("data.hours.title", "Les heures de votre ville"),
    ("data.hours.body", "Les heures affichées sont celles de la ville consultée, pas celles de votre téléphone : « pluie vers 17 h » à Montréal veut dire 17 h à Montréal."),
    ("phone.rain", "Pluie"),
    ("phone.rain.none", "Aucune"),
    ("phone.rain.now", "En cours"),
    ("phone.feels", "Ressenti"),
    ("footer.credit", "Klima — la météo de votre ville. Données {link}, sans clé d’API."),
    ("footer.note", "Les prévisions restent des prévisions ; en cas de vigilance, suivez les consignes des autorités."),
    ("sources.lead", "Un seul service donne un chiffre. Plusieurs services donnent un chiffre et une idée de sa fiabilité. Klima interroge des fournisseurs indépendants — modèles nationaux et observations de station — et les recoupe pour chaque heure de la prévision ; la carte ci-dessous montre leur accord pour l’heure en cours."),
    ("sources.note", "Chaque fournisseur est interrogé séparément : une panne, un refus ou une absence de couverture n’en écarte qu’un. MET Norway n’est appelé que là où l’on peut se nommer, comme ses conditions l’exigent."),
    ("sources.unavailable", "Comparaison indisponible pour cette ville."),
    ("footer.tagline", "La météo de votre ville. Sur iPhone, sur la montre et sur le web."),
    ("footer.legal", "Klima donne des prévisions, pas des consignes. En cas de vigilance météo, suivez les avis des autorités."),
    ("footer.models", "Neuf sources recoupées"),
    ("footer.method", "Indice européen de l’air, échelle UV de l’OMS"),
    ("nav.images", "En images"),
    ("gallery.title", "Dans votre poche"),
    ("gallery.lead", "Trois écrans de l’application, tels qu’un téléphone les montre — ici un après-midi parisien, avec une averse vers 17 h."),
    ("gallery.home.title", "La pluie d’abord"),
    ("gallery.home.body", "Dès l’ouverture : la température, puis la pluie qui vient — à quelle heure, combien, et ce qu’il faut emporter."),
    ("gallery.home.alt", "L’écran d’accueil : Paris, 19 °C, pluie vers 17 h avec 84 % de risque, douze barres de pluie heure par heure, et à emporter un parapluie, un manteau et des lunettes de soleil."),
    ("gallery.tiles.title", "Tout ce qu’on regarde"),
    ("gallery.tiles.body", "Ressenti, humidité, vent, UV, pression, qualité de l’air et pollens — une tuile chacun, avec sa jauge quand elle aide."),
    ("gallery.tiles.alt", "Les tuiles : ressenti 19 °C, humidité 58 %, vent 13 km/h, indice UV 5 modéré, 3,8 mm de pluie dans la journée, pression 1 016 hPa, qualité de l’air correcte, pollens de graminées."),
    ("gallery.sources.title", "Neuf sources, nommées"),
    ("gallery.sources.body", "Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC, JMA, MET Norway et une station : chacune dit sa température, et celle qui s’écarte se voit en orange."),
    ("gallery.sources.alt", "La carte des sources : Météo-France, ECMWF, Deutscher Wetterdienst, NOAA en orange, MET Norway et une station du DWD, avec leur température, leur pluie et leur vent."),
    ("feature.veille.title", "Le guetteur : 30 minutes, puis 2 heures"),
    ("feature.veille.rule", "Un quart d’heure est mouillé dès {amount} ; la pluie est forte dès {heavy} par heure."),
    ("feature.veille.detail", "Il lit la prévision au quart d’heure et la relit tous les quarts d’heure, tout seul : ce qui tombe dans la demi-heure en cours, puis ce qui vient d’ici deux heures, à la minute près."),
    ("app.openShort", "L’app"),
    ("film.label", "Ce que Klima regarde avant que vous sortiez"),
    ("film.signe", "Klima. La météo de votre ville."),
    ("film.nuage", "Il va pleuvoir ?"),
    ("film.heures", "Klima dit à quelle heure, et combien."),
    ("film.radar", "Le radar voit ce que les modèles ratent."),
    ("film.sources", "Neuf sources votent, heure par heure."),
    ("film.parapluie", "Et ce qu’il faut emporter."),
    ("film.soleil", "Le soleil, l’air, les pollens."),
    ("film.appareils", "Sur iPhone, sur la montre et sur le web."),
    ("film.fin", "Un coup d’œil, avant de sortir."),
];

const EN: [(&str, &str); 116] = [
    ("nav.today", "Today"),
    ("nav.data", "Data"),
    ("app.open", "Open the app"),
    ("hero.cta.today", "See today’s weather"),
    ("today.title", "Today’s weather"),
    ("today.loading", "Loading today…"),
    ("today.retry", "Try again"),
    ("today.error", "Could not load today’s weather."),
    ("today.rain", "Rain"),
    ("today.rain.detail", "{probability} probability"),
    ("today.gusts", "Gusts"),
    ("today.gusts.detail", "Highest of the day"),
    ("today.sunrise", "Sunrise"),
    ("today.sunrise.detail", "Sunset at {time}"),
    ("search.label", "Search for a town"),
    ("data.title", "Where the figures come from"),
    ("data.rules.title", "The same rules on both platforms"),
    ("phone.conditions", "Conditions"),
    ("phone.loading", "Loading"),
    ("nav.sources", "Sources"),
    ("sources.title", "Several providers, one answer"),
    ("sources.method", "How agreement is read"),
    ("sources.methodBody", "The value used is the median, less sensitive than an average to a single outlying model. Agreement counts as strong when the models sit within 1.5 °C and concur on rain; weak beyond 3 °C apart. When they diverge, Klima says so rather than displaying false precision."),
    ("sources.loading", "Cross-checking…"),
    ("sources.now", "Temperature for the current hour"),
    ("sources.answered", "{answered} of {queried} providers answered"),
    ("footer.product", "Product"),
    ("footer.app", "Web app"),
    ("footer.dataTitle", "Data"),
    ("footer.credits", "Credits"),
    ("footer.author", "Maxime Nathan Lestage"),
    ("footer.role", "Design and development"),
    ("footer.rights", "© {year} Klima — Maxime Nathan Lestage. All rights reserved."),
    ("language.label", "Language"),
    ("nav.indicators", "What Klima tells you"),
    ("hero.eyebrow", "iPhone, Apple Watch and web"),
    ("hero.title", "Your city’s weather, plainly told."),
    ("hero.lead", "Will it rain, and when? What should you take? Is the sun strong, is the air clean? Klima answers the questions you ask before heading out, from nine cross-checked sources."),
    ("hero.cta.indicators", "What Klima tells you"),
    ("hero.note", "Open-Meteo and Copernicus data, no account and no API key. Free, no advertising."),
    ("today.lead", "Try it on your own city. The site stops at today; the week, the hour-by-hour detail and alerts live in the app."),
    ("today.uv", "UV index"),
    ("today.next", "Rain ahead"),
    ("today.next.detail", "Over the next twelve hours"),
    ("today.advice.none", "Nothing special"),
    ("today.advice.detail", "Enjoy it."),
    ("today.feels", "Feels like"),
    ("today.feels.detail", "{temperature} on the thermometer"),
    ("today.sun", "Sun"),
    ("today.sun.detail", "Index {index} at its strongest today"),
    ("search.placeholder", "{commune} — change city"),
    ("features.title", "Seven answers before you head out"),
    ("features.lead", "No radar map to decode, no table of numbers: Klima starts from the forecast and gives answers — what time, what to take, what to watch for."),
    ("feature.rain.title", "Rain ahead"),
    ("feature.rain.rule", "An hour counts as rainy from {amount} or a {probability} chance."),
    ("feature.rain.detail", "Klima says when the rain arrives, how much it will bring and when it stops, over the next {hours} hours."),
    ("feature.advice.title", "Take with you"),
    ("feature.advice.rule", "An umbrella if it rains, a coat below {coat} feels-like, sunglasses from UV index {glasses}, sunscreen from {sunscreen}."),
    ("feature.advice.detail", "Water when it is hot, care when it freezes, and mind your umbrella when the gusts pick up."),
    ("feature.uv.title", "UV index"),
    ("feature.uv.rule", "The World Health Organization scale, from low to extreme."),
    ("feature.uv.detail", "The index is rounded before it is classed, as in forecasts: 2.6 reads 3, and counts as “moderate”."),
    ("feature.air.title", "Air quality"),
    ("feature.air.rule", "The European index and its six classes, from good to extremely poor."),
    ("feature.air.detail", "Fine particles, nitrogen dioxide and ozone, forecast by Copernicus for the current hour."),
    ("feature.pollen.title", "Pollen"),
    ("feature.pollen.rule", "Alder, birch, grass, mugwort, olive, ragweed."),
    ("feature.pollen.detail", "The most present is named, with its intensity. Forecast in Europe only: elsewhere, Klima says nothing."),
    ("feature.alerts.title", "Alerts"),
    ("feature.alerts.rule", "Rain within two hours, thunderstorms, frost, heat from {heat}, gusts from {gusts}."),
    ("feature.alerts.detail", "Never between 10 pm and 7 am, never twice in a row for the same reason. In the iPhone app."),
    ("figure.uv", "From index {sunscreen}, Klima suggests sunscreen."),
    ("data.model.title", "The models of the major institutes"),
    ("data.model.body", "The forecast is blended, hour by hour, from nine sources: the Météo-France, ECMWF, DWD, NOAA, Met Office, Environment Canada and Japan Meteorological Agency models, redistributed by Open-Meteo, MET Norway’s model and a DWD station. Klima keeps the median — one model going astray moves nothing — and takes the chance of rain from the share of sources that forecast rain; air and pollen from Copernicus European forecasts. Klima only asks for what a city looks at: rain and its chance, feels-like, wind and gusts, UV, pressure."),
    ("data.rules.body", "Thresholds live in one place for web and iOS, and both test suites cover the same cases. The answer is the same whichever screen you read it on."),
    ("data.hours.title", "Your city’s hours"),
    ("data.hours.body", "Times shown are those of the city you look at, not your phone’s: “rain around 5 pm” in Montreal means 5 pm in Montreal."),
    ("phone.rain", "Rain"),
    ("phone.rain.none", "None"),
    ("phone.rain.now", "Now"),
    ("phone.feels", "Feels like"),
    ("footer.credit", "Klima — your city’s weather. Data from {link}, no API key."),
    ("footer.note", "Forecasts remain forecasts; in a weather warning, follow official advice."),
    ("sources.lead", "One service gives a number. Several services give a number and a sense of how far to trust it. Klima queries independent providers — national models and station observations — and blends them for every hour of the forecast; the card below shows how far they agree for the current hour."),
    ("sources.note", "Each provider is queried separately: an outage, a refusal or a lack of coverage rules out only one. MET Norway is only called where we can identify ourselves, as its terms require."),
    ("sources.unavailable", "Comparison unavailable for this city."),
    ("footer.tagline", "Your city’s weather. On iPhone, on the watch and on the web."),
    ("footer.legal", "Klima gives forecasts, not instructions. In a weather warning, follow the authorities’ advice."),
    ("footer.models", "Nine cross-checked sources"),
    ("footer.method", "European air index, WHO UV scale"),
    ("nav.images", "In pictures"),
    ("gallery.title", "In your pocket"),
    ("gallery.lead", "Three screens from the app, as a phone shows them — here a Paris afternoon with a shower around 5 pm."),
    ("gallery.home.title", "Rain first"),
    ("gallery.home.body", "As soon as it opens: the temperature, then the rain ahead — what time, how much, and what to take with you."),
    ("gallery.home.alt", "The home screen: Paris, 19 °C, rain around 5 pm with an 84% chance, twelve hourly rain bars, and to take: an umbrella, a coat and sunglasses."),
    ("gallery.tiles.title", "Everything you look at"),
    ("gallery.tiles.body", "Feels-like, humidity, wind, UV, pressure, air quality and pollen — one tile each, with a gauge where it helps."),
    ("gallery.tiles.alt", "The tiles: feels like 19 °C, humidity 58%, wind 13 km/h, UV index 5 moderate, 3.8 mm of rain today, pressure 1,016 hPa, fair air quality, grass pollen."),
    ("gallery.sources.title", "Neuf sources, by name"),
    ("gallery.sources.body", "Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC, JMA, MET Norway and a station: each gives its temperature, and the one that strays shows in orange."),
    ("gallery.sources.alt", "The sources card: Météo-France, ECMWF, Deutscher Wetterdienst, NOAA in orange, MET Norway and a DWD station, with their temperature, rain and wind."),
    ("feature.veille.title", "The lookout: 30 minutes, then 2 hours"),
    ("feature.veille.rule", "A quarter hour counts as wet from {amount}; rain is heavy from {heavy} an hour."),
    ("feature.veille.detail", "It reads the quarter-hourly forecast and rereads it every fifteen minutes on its own: what falls in the current half hour, then what is coming over the next two hours, to the minute."),
    ("app.openShort", "App"),
    ("film.label", "What Klima looks at before you head out"),
    ("film.signe", "Klima. Your city’s weather."),
    ("film.nuage", "Will it rain?"),
    ("film.heures", "Klima tells you when, and how much."),
    ("film.radar", "The radar sees what the models miss."),
    ("film.sources", "Nine sources vote, hour by hour."),
    ("film.parapluie", "And what to take with you."),
    ("film.soleil", "The sun, the air, the pollen."),
    ("film.appareils", "On iPhone, Apple Watch and the web."),
    ("film.fin", "One glance, before you head out."),
];

const ES: [(&str, &str); 116] = [
    ("nav.today", "El tiempo de hoy"),
    ("nav.data", "Datos"),
    ("app.open", "Abrir la aplicación"),
    ("hero.cta.today", "Ver el tiempo de hoy"),
    ("today.title", "El tiempo de hoy"),
    ("today.loading", "Cargando el día…"),
    ("today.retry", "Reintentar"),
    ("today.error", "No se ha podido cargar el tiempo de hoy."),
    ("today.rain", "Lluvia"),
    ("today.rain.detail", "{probability} de probabilidad"),
    ("today.gusts", "Rachas"),
    ("today.gusts.detail", "Máximo del día"),
    ("today.sunrise", "Amanecer"),
    ("today.sunrise.detail", "Anochecer a las {time}"),
    ("search.label", "Buscar un municipio"),
    ("data.title", "De dónde salen las cifras"),
    ("data.rules.title", "Las mismas reglas en ambas plataformas"),
    ("phone.conditions", "Condiciones"),
    ("phone.loading", "Cargando"),
    ("nav.sources", "Fuentes"),
    ("sources.title", "Varios proveedores, una sola respuesta"),
    ("sources.method", "Cómo se lee el acuerdo"),
    ("sources.methodBody", "El valor retenido es la mediana, menos sensible que una media a un modelo aislado. El acuerdo se considera fuerte cuando los modelos caben en 1,5 °C y coinciden sobre la lluvia; débil más allá de 3 °C de diferencia. Cuando discrepan, Klima lo dice en lugar de mostrar una falsa precisión."),
    ("sources.loading", "Contrastando…"),
    ("sources.now", "Temperatura de la hora en curso"),
    ("sources.answered", "{answered} de {queried} proveedores han respondido"),
    ("footer.product", "El producto"),
    ("footer.app", "Aplicación web"),
    ("footer.dataTitle", "Los datos"),
    ("footer.credits", "Créditos"),
    ("footer.author", "Maxime Nathan Lestage"),
    ("footer.role", "Diseño y desarrollo"),
    ("footer.rights", "© {year} Klima — Maxime Nathan Lestage. Todos los derechos reservados."),
    ("language.label", "Idioma"),
    ("nav.indicators", "Lo que dice Klima"),
    ("hero.eyebrow", "iPhone, Apple Watch y web"),
    ("hero.title", "El tiempo de su ciudad, dicho con claridad."),
    ("hero.lead", "¿Va a llover, y cuándo? ¿Qué hay que llevar? ¿Pega fuerte el sol, es bueno el aire? Klima responde a lo que uno se pregunta antes de salir, a partir de nueve fuentes contrastadas."),
    ("hero.cta.indicators", "Lo que Klima le dice"),
    ("hero.note", "Datos de Open-Meteo y Copernicus, sin cuenta ni clave de API. Gratis, sin publicidad."),
    ("today.lead", "Pruébelo con su ciudad. El sitio se limita a hoy; la semana, el detalle hora a hora y las alertas están en la aplicación."),
    ("today.uv", "Índice UV"),
    ("today.next", "Lluvia por venir"),
    ("today.next.detail", "En las próximas doce horas"),
    ("today.advice.none", "Nada especial"),
    ("today.advice.detail", "Aprovéchelo."),
    ("today.feels", "Sensación"),
    ("today.feels.detail", "{temperature} en el termómetro"),
    ("today.sun", "Sol"),
    ("today.sun.detail", "Índice {index} en el momento más fuerte del día"),
    ("search.placeholder", "{commune} — cambiar de ciudad"),
    ("features.title", "Siete respuestas antes de salir"),
    ("features.lead", "Ni mapa de radar que descifrar, ni tabla de cifras: Klima parte de la previsión y da respuestas — a qué hora, qué llevar, de qué cuidarse."),
    ("feature.rain.title", "Lluvia por venir"),
    ("feature.rain.rule", "Una hora es lluviosa a partir de {amount} o de un {probability} de probabilidad."),
    ("feature.rain.detail", "Klima dice cuándo llega la lluvia, cuánto dejará y cuándo cesa, en las próximas {hours} horas."),
    ("feature.advice.title", "Para llevar"),
    ("feature.advice.rule", "Un paraguas si llueve, un abrigo por debajo de {coat} de sensación, gafas desde el índice UV {glasses}, crema desde {sunscreen}."),
    ("feature.advice.detail", "Agua cuando hace calor, prudencia cuando hiela, y cuidado con el paraguas cuando arrecian las rachas."),
    ("feature.uv.title", "Índice UV"),
    ("feature.uv.rule", "La escala de la Organización Mundial de la Salud, de bajo a extremo."),
    ("feature.uv.detail", "El índice se redondea antes de clasificarse, como en los boletines: 2,6 se lee 3, y cuenta como «moderado»."),
    ("feature.air.title", "Calidad del aire"),
    ("feature.air.rule", "El índice europeo y sus seis clases, de buena a extremadamente mala."),
    ("feature.air.detail", "Partículas finas, dióxido de nitrógeno y ozono, previstos por Copernicus para la hora en curso."),
    ("feature.pollen.title", "Polen"),
    ("feature.pollen.rule", "Aliso, abedul, gramíneas, artemisa, olivo, ambrosía."),
    ("feature.pollen.detail", "Se nombra el más presente, con su intensidad. Previsto solo en Europa: en otros lugares, Klima no dice nada."),
    ("feature.alerts.title", "Alertas"),
    ("feature.alerts.rule", "La lluvia en las dos horas siguientes, la tormenta, la helada, el calor desde {heat}, las rachas desde {gusts}."),
    ("feature.alerts.detail", "Nunca entre las 22 h y las 7 h, nunca dos veces seguidas por el mismo motivo. En la aplicación de iPhone."),
    ("figure.uv", "Desde el índice {sunscreen}, Klima aconseja crema solar."),
    ("data.model.title", "Los modelos de los grandes institutos"),
    ("data.model.body", "La previsión se combina, hora a hora, a partir de nueve fuentes: los modelos de Météo-France, el ECMWF, el DWD, la NOAA, el Met Office, Environment Canada y la agencia japonesa, redistribuidos por Open-Meteo, el de MET Norway y una estación del DWD. Klima se queda con la mediana — un modelo que se desvía no mueve nada — y saca la probabilidad de lluvia de la proporción de fuentes que anuncian lluvia; el aire y el polen, de las previsiones europeas de Copernicus. Klima solo pide lo que mira una ciudad: lluvia y probabilidad, sensación térmica, viento y rachas, UV, presión."),
    ("data.rules.body", "Los umbrales viven en un mismo lugar para la web y para iOS, y ambas baterías de pruebas cubren los mismos casos. La respuesta es la misma, sea cual sea la pantalla."),
    ("data.hours.title", "Las horas de su ciudad"),
    ("data.hours.body", "Las horas mostradas son las de la ciudad consultada, no las de su teléfono: «lluvia hacia las 17 h» en Montreal significa las 17 h en Montreal."),
    ("phone.rain", "Lluvia"),
    ("phone.rain.none", "Ninguna"),
    ("phone.rain.now", "Ahora"),
    ("phone.feels", "Sensación"),
    ("footer.credit", "Klima — el tiempo de su ciudad. Datos de {link}, sin clave de API."),
    ("footer.note", "Las previsiones siguen siendo previsiones; en caso de aviso, siga las indicaciones oficiales."),
    ("sources.lead", "Un solo servicio da una cifra. Varios servicios dan una cifra y una idea de su fiabilidad. Klima consulta proveedores independientes — modelos nacionales y observaciones de estación — y los combina para cada hora de la previsión; la tarjeta de abajo muestra su acuerdo para la hora en curso."),
    ("sources.note", "Cada proveedor se consulta por separado: una caída, un rechazo o una falta de cobertura solo descarta uno. MET Norway solo se llama donde podemos identificarnos, como exigen sus condiciones."),
    ("sources.unavailable", "Comparación no disponible para esta ciudad."),
    ("footer.tagline", "El tiempo de su ciudad. En el iPhone, en el reloj y en la web."),
    ("footer.legal", "Klima da previsiones, no instrucciones. En caso de aviso meteorológico, siga las indicaciones de las autoridades."),
    ("footer.models", "Nueve fuentes contrastadas"),
    ("footer.method", "Índice europeo del aire, escala UV de la OMS"),
    ("nav.images", "En imágenes"),
    ("gallery.title", "En su bolsillo"),
    ("gallery.lead", "Tres pantallas de la aplicación, tal como las muestra un teléfono — aquí una tarde en París con un chubasco hacia las 17 h."),
    ("gallery.home.title", "Primero, la lluvia"),
    ("gallery.home.body", "Nada más abrirla: la temperatura y luego la lluvia que viene — a qué hora, cuánta y qué hay que llevar."),
    ("gallery.home.alt", "La pantalla de inicio: París, 19 °C, lluvia hacia las 17 h con un 84 % de probabilidad, doce barras de lluvia hora a hora y, para llevar, un paraguas, un abrigo y gafas de sol."),
    ("gallery.tiles.title", "Todo lo que se mira"),
    ("gallery.tiles.body", "Sensación, humedad, viento, UV, presión, calidad del aire y polen — una tarjeta para cada uno, con su indicador cuando ayuda."),
    ("gallery.tiles.alt", "Las tarjetas: sensación 19 °C, humedad 58 %, viento 13 km/h, índice UV 5 moderado, 3,8 mm de lluvia en el día, presión 1016 hPa, calidad del aire razonable, polen de gramíneas."),
    ("gallery.sources.title", "Nueve fuentes, con nombre"),
    ("gallery.sources.body", "Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC, JMA, MET Norway y una estación: cada una da su temperatura, y la que se aparta se ve en naranja."),
    ("gallery.sources.alt", "La tarjeta de fuentes: Météo-France, ECMWF, Deutscher Wetterdienst, NOAA en naranja, MET Norway y una estación del DWD, con su temperatura, lluvia y viento."),
    ("feature.veille.title", "El vigía: 30 minutos, luego 2 horas"),
    ("feature.veille.rule", "Un cuarto de hora es lluvioso desde {amount}; la lluvia es fuerte desde {heavy} por hora."),
    ("feature.veille.detail", "Lee la previsión por cuartos de hora y la relee cada cuarto de hora, solo: lo que cae en la media hora en curso, y lo que viene en las próximas dos horas, al minuto."),
    ("app.openShort", "La app"),
    ("film.label", "Lo que Klima mira antes de que salgas"),
    ("film.signe", "Klima. El tiempo de tu ciudad."),
    ("film.nuage", "¿Va a llover?"),
    ("film.heures", "Klima te dice a qué hora, y cuánto."),
    ("film.radar", "El radar ve lo que los modelos no ven."),
    ("film.sources", "Nueve fuentes votan, hora a hora."),
    ("film.parapluie", "Y qué llevar contigo."),
    ("film.soleil", "El sol, el aire, el polen."),
    ("film.appareils", "En iPhone, en el Apple Watch y en la web."),
    ("film.fin", "Un vistazo, antes de salir."),
];

#[cfg(test)]
mod tests {
    use super::*;
    use klima_core::i18n::{LANGUAGES, REFERENCE_LANGUAGE};

    #[test]
    fn les_trois_langues_portent_exactement_les_memes_cles() {
        let reference: Vec<&str> =
            SITE_MESSAGES.catalog(REFERENCE_LANGUAGE).keys().copied().collect();
        for language in LANGUAGES {
            let keys: Vec<&str> = SITE_MESSAGES.catalog(language).keys().copied().collect();
            assert_eq!(keys, reference, "{language}");
        }
    }

    #[test]
    fn aucun_texte_vide() {
        for language in LANGUAGES {
            for (key, value) in SITE_MESSAGES.catalog(language) {
                assert!(!value.trim().is_empty(), "{language} · {key}");
            }
        }
    }

    #[test]
    fn un_motif_a_trous_a_les_memes_trous_partout() {
        let jetons = |valeur: &str| {
            let mut noms: Vec<String> = Vec::new();
            let mut reste = valeur;
            while let Some(debut) = reste.find('{') {
                let apres = &reste[debut + 1..];
                let len = apres
                    .char_indices()
                    .find(|(_, c)| !(c.is_ascii_alphanumeric() || *c == '_'))
                    .map_or(apres.len(), |(i, _)| i);
                if len > 0 && apres[len..].starts_with('}') {
                    noms.push(apres[..len].to_owned());
                    reste = &apres[len + 1..];
                } else {
                    reste = apres;
                }
            }
            noms.sort();
            noms
        };

        for (key, modele) in SITE_MESSAGES.catalog(REFERENCE_LANGUAGE) {
            let reference = jetons(modele);
            for language in LANGUAGES {
                let valeur = SITE_MESSAGES.get(language, key).unwrap();
                assert_eq!(jetons(valeur), reference, "{language} · {key}");
            }
        }
    }

    /// Le site ne publie pas de lien vers le code source : c'est une règle du
    /// dépôt, et un libellé est le premier endroit où elle se perdrait.
    #[test]
    fn aucun_lien_vers_le_code_source() {
        for language in LANGUAGES {
            for (key, value) in SITE_MESSAGES.catalog(language) {
                let minuscule = value.to_lowercase();
                assert!(!minuscule.contains("github"), "{language} · {key}");
                assert!(!minuscule.contains("code source"), "{language} · {key}");
                assert!(!minuscule.contains("source code"), "{language} · {key}");
            }
        }
    }
}
