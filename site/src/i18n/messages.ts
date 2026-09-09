/** Textes du site de présentation. Les libellés partagés viennent de `@klima/core`. */
import type { Catalog, Language } from '@klima/core'

export const siteMessages: Record<Language, Catalog> = {
  fr: {
    'nav.today': 'Météo du jour',
    'nav.indicators': 'Indicateurs',
    'nav.data': 'Données',

    'hero.eyebrow': 'Application iOS et web',
    'hero.title': 'La météo qui parle agronomie.',
    'hero.lead':
      'Klima lit la météo comme un agronome : humidité et température du sol, évapotranspiration de référence, fenêtres de pulvérisation, degrés-jours. Les variables agricoles brutes, traduites en décisions pour la parcelle.',
    'hero.cta.today': 'Voir la météo du jour',
    'hero.cta.indicators': 'Ce que Klima calcule',
    'hero.note': 'Données Open-Meteo, sans compte ni clé d’API. Gratuit, sans publicité.',

    'today.title': 'La météo du jour',
    'today.lead':
      'Essayez sur votre commune. Le site s’en tient à aujourd’hui ; la semaine et le détail heure par heure sont dans l’application.',
    'today.loading': 'Chargement de la journée…',
    'today.retry': 'Réessayer',
    'today.error': 'Impossible de charger la météo du jour.',

    'today.rain': 'Pluie',
    'today.rain.detail': '{probability} de probabilité',
    'today.gusts': 'Rafales',
    'today.gusts.detail': 'Maximum de la journée',
    'today.sunrise': 'Lever',
    'today.sunrise.detail': 'Coucher à {time}',
    'today.et0': 'ET0',
    'today.et0.detail': 'Évapotranspiration de référence',

    'today.spray': 'Fenêtre de traitement',
    'today.spray.none': 'Aucune d’ici ce soir',
    'today.spray.score': 'Score {score}/100 sur la plage',
    'today.spray.blocked': 'Vent, pluie ou température hors des clous',
    'today.balance': 'Bilan du jour',
    'today.balance.deficit': 'La parcelle puise dans sa réserve',
    'today.balance.ok': 'La pluie couvre l’évapotranspiration',
    'today.soil': 'État du sol',
    'today.soil.detail': '{moisture} vol. · {state}',
    'today.frost': 'Gel cette nuit',
    'today.frost.detail': 'Mini {temperature}{hoarFrost}',

    'search.placeholder': '{commune} — changer de commune',
    'search.label': 'Rechercher une commune',

    'features.title': 'Six indicateurs, une seule question',
    'features.lead':
      'Est-ce que je peux y aller aujourd’hui ? Klima part des variables agricoles brutes et rend une réponse, pas un tableau de chiffres.',

    'feature.water.title': 'Bilan hydrique',
    'feature.water.rule': 'Pluie moins évapotranspiration de référence, cumulées sur sept jours.',
    'feature.water.detail':
      'Passé un déficit de {deficit} sur la période, Klima chiffre l’irrigation à apporter.',
    'feature.soil.title': 'État du sol',
    'feature.soil.rule': 'Humidité volumique entre 3 et 9 cm, et température à 6 cm.',
    'feature.soil.detail':
      'Saturé au-delà de {wet} vol., sec en deçà de {dry}. Portance et aptitude au semis en découlent.',
    'feature.spray.title': 'Fenêtre de traitement',
    'feature.spray.rule':
      'Vent entre {min} et {max}, rafales sous {gusts}, pas de pluie dans les deux heures.',
    'feature.spray.detail':
      'S’y ajoutent la température ({tempMin} à {tempMax}), l’hygrométrie et un déficit de pression de vapeur sous {vpd}.',
    'feature.disease.title': 'Pression maladie',
    'feature.disease.rule': 'Heures d’humectation du feuillage : hygrométrie au-dessus de {humidity}.',
    'feature.disease.detail':
      'Comptées dans la plage de température favorable au champignon, {min} à {max}.',
    'feature.frost.title': 'Risque de gel',
    'feature.frost.rule': 'Température minimale de la nuit et point de rosée.',
    'feature.frost.detail': 'Distingue la gelée blanche du gel sévère, jusqu’à −4 °C et au-delà.',
    'feature.gdd.title': 'Degrés-jours',
    'feature.gdd.rule': 'Moyenne plafonnée, base {base}.',
    'feature.gdd.detail': 'La journée ne capitalise plus au-delà de {ceiling}.',

    'data.title': 'D’où viennent les chiffres',
    'data.model.title': 'Un modèle agricole, pas une météo grand public',
    'data.model.body':
      'Klima n’interroge que les variables agronomiques de l’API Open-Meteo : température et humidité du sol par couche, évapotranspiration de référence FAO-56, déficit de pression de vapeur — auxquelles s’ajoutent le vent, la pluie et l’hygrométrie nécessaires aux fenêtres de traitement.',
    'data.rules.title': 'Les mêmes règles sur les deux plateformes',
    'data.rules.body':
      'Les seuils vivent au même endroit pour le web et pour iOS, et les deux suites de tests couvrent les mêmes cas. Le conseil rendu au champ est identique, quel que soit l’écran par lequel on le lit.',
    'data.wind.title': 'Le vent fait loi',
    'data.wind.body':
      'Un vent au-delà de {limit}, des rafales fortes ou une pluie imminente rendent l’heure inexploitable pour un traitement, quel que soit le reste des conditions. Klima ne propose jamais une fenêtre hors des clous.',

    'phone.conditions': 'Conditions météo',
    'phone.balance': 'Bilan hydrique',
    'phone.spray': 'Traitement',
    'phone.spray.yes': 'Possible',
    'phone.spray.no': 'Non',
    'phone.loading': 'Chargement',

    'footer.credit': 'Klima — météo agricole. Données {link}, sans clé d’API.',
    'footer.note':
      'Les indicateurs sont des aides à la décision ; ils ne remplacent ni l’observation de la parcelle ni la réglementation en vigueur.',

    'language.label': 'Langue',
  },

  en: {
    'nav.today': 'Today',
    'nav.indicators': 'Indicators',
    'nav.data': 'Data',

    'hero.eyebrow': 'iOS and web app',
    'hero.title': 'Weather that speaks agronomy.',
    'hero.lead':
      'Klima reads the weather like an agronomist: soil moisture and temperature, reference evapotranspiration, spraying windows, growing degree days. Raw agricultural variables, turned into decisions for the field.',
    'hero.cta.today': 'See today’s weather',
    'hero.cta.indicators': 'What Klima works out',
    'hero.note': 'Open-Meteo data, no account and no API key. Free, no advertising.',

    'today.title': 'Today’s weather',
    'today.lead':
      'Try it on your own town. The site stops at today; the week and the hour-by-hour detail live in the app.',
    'today.loading': 'Loading today…',
    'today.retry': 'Try again',
    'today.error': 'Could not load today’s weather.',

    'today.rain': 'Rain',
    'today.rain.detail': '{probability} probability',
    'today.gusts': 'Gusts',
    'today.gusts.detail': 'Highest of the day',
    'today.sunrise': 'Sunrise',
    'today.sunrise.detail': 'Sunset at {time}',
    'today.et0': 'ET0',
    'today.et0.detail': 'Reference evapotranspiration',

    'today.spray': 'Spraying window',
    'today.spray.none': 'None left today',
    'today.spray.score': 'Score {score}/100 over the window',
    'today.spray.blocked': 'Wind, rain or temperature out of bounds',
    'today.balance': 'Balance today',
    'today.balance.deficit': 'The field is drawing on its reserve',
    'today.balance.ok': 'Rain covers evapotranspiration',
    'today.soil': 'Soil state',
    'today.soil.detail': '{moisture} vol. · {state}',
    'today.frost': 'Frost tonight',
    'today.frost.detail': 'Low of {temperature}{hoarFrost}',

    'search.placeholder': '{commune} — change town',
    'search.label': 'Search for a town',

    'features.title': 'Six indicators, one question',
    'features.lead':
      'Can I go out today? Klima starts from raw agricultural variables and returns an answer, not a table of figures.',

    'feature.water.title': 'Water balance',
    'feature.water.rule': 'Rain minus reference evapotranspiration, over seven days.',
    'feature.water.detail':
      'Beyond a {deficit} deficit over the period, Klima puts a figure on the irrigation needed.',
    'feature.soil.title': 'Soil state',
    'feature.soil.rule': 'Volumetric moisture between 3 and 9 cm, and temperature at 6 cm.',
    'feature.soil.detail':
      'Waterlogged above {wet} vol., dry below {dry}. Trafficability and fitness for drilling follow.',
    'feature.spray.title': 'Spraying window',
    'feature.spray.rule':
      'Wind between {min} and {max}, gusts under {gusts}, no rain within two hours.',
    'feature.spray.detail':
      'Then come temperature ({tempMin} to {tempMax}), humidity and a vapour pressure deficit under {vpd}.',
    'feature.disease.title': 'Disease pressure',
    'feature.disease.rule': 'Hours of leaf wetness: humidity above {humidity}.',
    'feature.disease.detail':
      'Counted within the temperature range the fungus favours, {min} to {max}.',
    'feature.frost.title': 'Frost risk',
    'feature.frost.rule': 'Overnight minimum temperature and dew point.',
    'feature.frost.detail': 'Tells hoar frost from severe frost, down to −4 °C and below.',
    'feature.gdd.title': 'Growing degree days',
    'feature.gdd.rule': 'Capped average, base {base}.',
    'feature.gdd.detail': 'The day stops accumulating above {ceiling}.',

    'data.title': 'Where the figures come from',
    'data.model.title': 'An agricultural model, not a consumer forecast',
    'data.model.body':
      'Klima only queries the agronomic variables of the Open-Meteo API: soil temperature and moisture by layer, FAO-56 reference evapotranspiration, vapour pressure deficit — plus the wind, rain and humidity the spraying windows need.',
    'data.rules.title': 'The same rules on both platforms',
    'data.rules.body':
      'The thresholds live in one place for web and for iOS, and both test suites cover the same cases. The advice reaching the field is identical, whichever screen you read it on.',
    'data.wind.title': 'Wind has the final word',
    'data.wind.body':
      'Wind above {limit}, strong gusts or imminent rain make an hour unusable for spraying, whatever the rest of the conditions. Klima never offers a window outside the rules.',

    'phone.conditions': 'Conditions',
    'phone.balance': 'Water balance',
    'phone.spray': 'Spraying',
    'phone.spray.yes': 'Possible',
    'phone.spray.no': 'No',
    'phone.loading': 'Loading',

    'footer.credit': 'Klima — agricultural weather. Data from {link}, no API key.',
    'footer.note':
      'These indicators support a decision; they replace neither walking the field nor the regulations in force.',

    'language.label': 'Language',
  },

  es: {
    'nav.today': 'El tiempo de hoy',
    'nav.indicators': 'Indicadores',
    'nav.data': 'Datos',

    'hero.eyebrow': 'Aplicación iOS y web',
    'hero.title': 'La meteorología que habla de agronomía.',
    'hero.lead':
      'Klima lee el tiempo como un agrónomo: humedad y temperatura del suelo, evapotranspiración de referencia, ventanas de pulverización, grados-día. Las variables agrícolas en bruto, convertidas en decisiones para la parcela.',
    'hero.cta.today': 'Ver el tiempo de hoy',
    'hero.cta.indicators': 'Lo que calcula Klima',
    'hero.note': 'Datos de Open-Meteo, sin cuenta ni clave de API. Gratis y sin publicidad.',

    'today.title': 'El tiempo de hoy',
    'today.lead':
      'Pruébelo en su municipio. El sitio se limita a hoy; la semana y el detalle hora a hora están en la aplicación.',
    'today.loading': 'Cargando el día…',
    'today.retry': 'Reintentar',
    'today.error': 'No se ha podido cargar el tiempo de hoy.',

    'today.rain': 'Lluvia',
    'today.rain.detail': '{probability} de probabilidad',
    'today.gusts': 'Rachas',
    'today.gusts.detail': 'Máximo del día',
    'today.sunrise': 'Amanecer',
    'today.sunrise.detail': 'Anochecer a las {time}',
    'today.et0': 'ET0',
    'today.et0.detail': 'Evapotranspiración de referencia',

    'today.spray': 'Ventana de tratamiento',
    'today.spray.none': 'Ninguna de aquí a esta noche',
    'today.spray.score': 'Puntuación {score}/100 en la franja',
    'today.spray.blocked': 'Viento, lluvia o temperatura fuera de rango',
    'today.balance': 'Balance del día',
    'today.balance.deficit': 'La parcela tira de su reserva',
    'today.balance.ok': 'La lluvia cubre la evapotranspiración',
    'today.soil': 'Estado del suelo',
    'today.soil.detail': '{moisture} vol. · {state}',
    'today.frost': 'Helada esta noche',
    'today.frost.detail': 'Mínima de {temperature}{hoarFrost}',

    'search.placeholder': '{commune} — cambiar de municipio',
    'search.label': 'Buscar un municipio',

    'features.title': 'Seis indicadores, una sola pregunta',
    'features.lead':
      '¿Puedo salir hoy? Klima parte de las variables agrícolas en bruto y devuelve una respuesta, no una tabla de cifras.',

    'feature.water.title': 'Balance hídrico',
    'feature.water.rule': 'Lluvia menos evapotranspiración de referencia, acumuladas en siete días.',
    'feature.water.detail':
      'Superado un déficit de {deficit} en el periodo, Klima cifra el riego necesario.',
    'feature.soil.title': 'Estado del suelo',
    'feature.soil.rule': 'Humedad volumétrica entre 3 y 9 cm, y temperatura a 6 cm.',
    'feature.soil.detail':
      'Encharcado por encima de {wet} vol., seco por debajo de {dry}. De ahí salen la portancia y la aptitud para sembrar.',
    'feature.spray.title': 'Ventana de tratamiento',
    'feature.spray.rule':
      'Viento entre {min} y {max}, rachas por debajo de {gusts}, sin lluvia en dos horas.',
    'feature.spray.detail':
      'A ello se suman la temperatura ({tempMin} a {tempMax}), la humedad y un déficit de presión de vapor por debajo de {vpd}.',
    'feature.disease.title': 'Presión de enfermedad',
    'feature.disease.rule': 'Horas de humectación foliar: humedad por encima de {humidity}.',
    'feature.disease.detail':
      'Contadas en el rango de temperatura que favorece al hongo, de {min} a {max}.',
    'feature.frost.title': 'Riesgo de helada',
    'feature.frost.rule': 'Temperatura mínima de la noche y punto de rocío.',
    'feature.frost.detail': 'Distingue la escarcha de la helada severa, hasta −4 °C y más allá.',
    'feature.gdd.title': 'Grados-día',
    'feature.gdd.rule': 'Media con techo, base {base}.',
    'feature.gdd.detail': 'El día deja de acumular por encima de {ceiling}.',

    'data.title': 'De dónde salen las cifras',
    'data.model.title': 'Un modelo agrícola, no una meteorología de consumo',
    'data.model.body':
      'Klima solo consulta las variables agronómicas de la API de Open-Meteo: temperatura y humedad del suelo por capa, evapotranspiración de referencia FAO-56, déficit de presión de vapor — más el viento, la lluvia y la humedad que necesitan las ventanas de tratamiento.',
    'data.rules.title': 'Las mismas reglas en ambas plataformas',
    'data.rules.body':
      'Los umbrales viven en un solo sitio para la web y para iOS, y ambas baterías de pruebas cubren los mismos casos. El consejo que llega al campo es idéntico, sea cual sea la pantalla.',
    'data.wind.title': 'El viento manda',
    'data.wind.body':
      'Un viento por encima de {limit}, rachas fuertes o lluvia inminente hacen que la hora sea inservible para un tratamiento, con independencia del resto. Klima nunca propone una ventana fuera de norma.',

    'phone.conditions': 'Condiciones',
    'phone.balance': 'Balance hídrico',
    'phone.spray': 'Tratamiento',
    'phone.spray.yes': 'Posible',
    'phone.spray.no': 'No',
    'phone.loading': 'Cargando',

    'footer.credit': 'Klima — meteorología agrícola. Datos de {link}, sin clave de API.',
    'footer.note':
      'Los indicadores son una ayuda a la decisión; no sustituyen ni la observación de la parcela ni la normativa vigente.',

    'language.label': 'Idioma',
  },
}
