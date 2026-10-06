"""Génère les catalogues de chaînes iOS (.xcstrings) à partir d'une table lisible."""
import json, sys, os

# clé : (fr, en, es)
STRINGS = {
    # --- Codes météo WMO ---
    "wmo.clearSky": ("Ciel dégagé", "Clear sky", "Cielo despejado"),
    "wmo.mainlyClear": ("Peu nuageux", "Mainly clear", "Poco nuboso"),
    "wmo.partlyCloudy": ("Partiellement nuageux", "Partly cloudy", "Parcialmente nuboso"),
    "wmo.overcast": ("Couvert", "Overcast", "Cubierto"),
    "wmo.fog": ("Brouillard", "Fog", "Niebla"),
    "wmo.rimeFog": ("Brouillard givrant", "Freezing fog", "Niebla helada"),
    "wmo.lightDrizzle": ("Bruine légère", "Light drizzle", "Llovizna débil"),
    "wmo.drizzle": ("Bruine", "Drizzle", "Llovizna"),
    "wmo.denseDrizzle": ("Bruine dense", "Heavy drizzle", "Llovizna intensa"),
    "wmo.freezingDrizzle": ("Bruine verglaçante", "Freezing drizzle", "Llovizna engelante"),
    "wmo.denseFreezingDrizzle": ("Bruine verglaçante dense", "Heavy freezing drizzle", "Llovizna engelante intensa"),
    "wmo.slightRain": ("Pluie faible", "Light rain", "Lluvia débil"),
    "wmo.rain": ("Pluie", "Rain", "Lluvia"),
    "wmo.heavyRain": ("Pluie forte", "Heavy rain", "Lluvia fuerte"),
    "wmo.freezingRain": ("Pluie verglaçante", "Freezing rain", "Lluvia engelante"),
    "wmo.heavyFreezingRain": ("Pluie verglaçante forte", "Heavy freezing rain", "Lluvia engelante fuerte"),
    "wmo.slightSnow": ("Neige faible", "Light snow", "Nieve débil"),
    "wmo.snow": ("Neige", "Snow", "Nieve"),
    "wmo.heavySnow": ("Neige forte", "Heavy snow", "Nieve fuerte"),
    "wmo.snowGrains": ("Grains de neige", "Snow grains", "Cinarra"),
    "wmo.showers": ("Averses", "Showers", "Chubascos"),
    "wmo.moderateShowers": ("Averses modérées", "Moderate showers", "Chubascos moderados"),
    "wmo.violentShowers": ("Averses violentes", "Violent showers", "Chubascos violentos"),
    "wmo.snowShowers": ("Averses de neige", "Snow showers", "Chubascos de nieve"),
    "wmo.heavySnowShowers": ("Averses de neige fortes", "Heavy snow showers", "Chubascos de nieve fuertes"),
    "wmo.thunderstorm": ("Orage", "Thunderstorm", "Tormenta"),
    "wmo.thunderstormHail": ("Orage et grêle", "Thunderstorm with hail", "Tormenta con granizo"),
    "wmo.thunderstormHeavyHail": ("Orage et forte grêle", "Thunderstorm with heavy hail", "Tormenta con granizo fuerte"),

    # --- États du domaine ---

    # --- Motifs de blocage ---

    # --- Service ---
    "api.unreachable": ("Service météo injoignable. Vérifiez votre connexion.",
                        "Weather service unreachable. Check your connection.",
                        "Servicio meteorológico inaccesible. Compruebe su conexión."),
    "api.status": ("Le service météo a répondu %1$@.", "The weather service replied %1$@.",
                   "El servicio meteorológico ha respondido %1$@."),
    "api.malformed": ("Réponse illisible du service météo.", "Unreadable response from the weather service.",
                      "Respuesta ilegible del servicio meteorológico."),
    "location.denied": ("Localisation refusée. Recherchez la ville à la main.", "Location denied. Search for the city instead.", "Ubicación denegada. Busque la ciudad a mano."),
    "location.unavailable": ("Position indisponible pour le moment.", "Location unavailable right now.",
                             "Ubicación no disponible por ahora."),

    # --- Application ---
    "app.loading": ("Chargement de la météo…", "Loading the weather…", "Cargando el tiempo…"),
    "app.retry": ("Réessayer", "Try again", "Reintentar"),
    "app.locate": ("Me localiser", "Locate me", "Ubicarme"),
    "app.search": ("Rechercher une ville", "Search for a city", "Buscar una ciudad"),
    "app.error": ("Impossible de charger la prévision.", "Could not load the forecast.", "No se ha podido cargar la previsión."),
    "app.source": ("Données Open-Meteo — modèles Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC et JMA. Ville à %1$@ d’altitude.", "Open-Meteo data — Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC and JMA models. City at %1$@ elevation.", "Datos de Open-Meteo — modelos de Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC y JMA. Ciudad a %1$@ de altitud."),
    "search.myField": ("Ma ville", "My city", "Mi ciudad"),

    "hourly.title": ("Conditions météo", "Conditions", "Condiciones"),
    "hourly.now": ("Maint.", "Now", "Ahora"),
    "daily.title": ("Prévision sur 7 jours", "7-day forecast", "Previsión a 7 días"),
    "daily.today": ("Auj.", "Today", "Hoy"),


    # --- Activité en direct ---
    # --- Recoupement des modèles ---
    "consensus.title": ("Accord des sources", "Source agreement", "Acuerdo de las fuentes"),
    "consensus.forte": ("Fort", "Strong", "Fuerte"),
    "consensus.moyenne": ("Moyen", "Moderate", "Medio"),
    "consensus.faible": ("Faible", "Weak", "Débil"),
    "consensus.detail": ("%1$@ sources · écart %2$@", "%1$@ sources · %2$@ apart", "%1$@ fuentes · diferencia de %2$@"),
    "consensus.rainDisagreement": ("désaccord sur la pluie", "they disagree on rain",
                                   "discrepan sobre la lluvia"),
    "consensus.median": ("Valeur retenue : %1$@", "Value used: %1$@", "Valor retenido: %1$@"),
    "sources.title": ("Ce que dit chaque source", "What each source says", "Lo que dice cada fuente"),
    "sources.dry": ("sec", "dry", "seco"),

    # --- Alertes : ce que Kliima dit sans qu'on ouvre l'application ---

    # --- Ce qu'un palier ferme ---
    # L'espace avant le triangle est insécable (U+00A0) : dans une tuile
    # étroite, « Avec Kliima ‣ Pro » se coupait en « Avec Kliima » et
    # « ‣ Pro », le signe orphelin sur sa ligne. Le nom affiché sous l'icône,
    # lui, garde son espace ordinaire.
    "plan.libre": ("Kliima ‣", "Kliima ‣", "Kliima ‣"),
    "plan.pro": ("Kliima ‣ Pro", "Kliima ‣ Pro", "Kliima ‣ Pro"),
    "plan.reason.recoupement": ("Comparer plusieurs sources demande l’abonnement.", "Comparing several sources needs the subscription.", "Comparar varias fuentes requiere la suscripción."),
    "plan.reason.alertes": ("Être prévenu sans ouvrir l'application demande l'abonnement.",
                            "Being warned without opening the app needs the subscription.",
                            "Recibir avisos sin abrir la aplicación requiere la suscripción."),

    # --- Écran d'abonnement ---
    "paywall.title": ("Kliima ‣ Pro", "Kliima ‣ Pro", "Kliima ‣ Pro"),
    "plan.feature.recoupement": ("Neuf sources recoupées, et leur niveau d’accord", "Neuf sources cross-checked, and how far they agree", "Nueve fuentes contrastadas, y su grado de acuerdo"),
    "plan.reason.villes": ("Enregistrer plusieurs villes demande Kliima ‣ Pro.", "Saving several cities needs Kliima ‣ Pro.", "Guardar varias ciudades requiere Kliima ‣ Pro."),
    "plan.feature.villes": ("Plusieurs villes enregistrées, de l’une à l’autre d’un geste", "Several saved cities, one tap from each other", "Varias ciudades guardadas, a un toque una de otra"),
    "plan.feature.alertes": ("Prévenu de la pluie, de l’orage, du gel et de la chaleur", "Warned of rain, storms, frost and heat", "Avisos de lluvia, tormenta, helada y calor"),
    "paywall.free": ("Une ville et la journée entière restent gratuites, sans compte ni publicité.", "One city and the whole day stay free, no account and no adverts.", "Una ciudad y el día entero siguen siendo gratis, sin cuenta ni publicidad."),
    "paywall.buy": ("S'abonner — %1$@ par mois", "Subscribe — %1$@ a month", "Suscribirse — %1$@ al mes"),
    "paywall.restore": ("Restaurer un achat", "Restore a purchase", "Restaurar una compra"),
    "paywall.terms": ("Abonnement mensuel, renouvelé automatiquement, résiliable à tout moment depuis les réglages de l'App Store.",
                      "Monthly subscription, renewed automatically, cancellable at any time from your App Store settings.",
                      "Suscripción mensual, renovada automáticamente, cancelable en cualquier momento desde los ajustes de la App Store."),
    "paywall.close": ("Fermer", "Close", "Cerrar"),
    "paywall.unavailable": ("Boutique injoignable pour l'instant.", "Store unreachable for now.",
                            "Tienda no disponible por ahora."),

    # --- Compte : n'apparaît que si un relais est configuré ---
    "account.title": ("Compte d'essai", "Trial account", "Cuenta de prueba"),
    "account.hint": ("Connectez-vous avec Apple : si votre compte est invité, le palier s'ouvre sans passer par la boutique. Rien à saisir.",
                     "Sign in with Apple: if your account is invited, the tier opens without going through the store. Nothing to type.",
                     "Inicie sesión con Apple: si su cuenta está invitada, el plan se abre sin pasar por la tienda. Nada que escribir."),
    "account.signedIn": ("Connecté : %1$@", "Signed in: %1$@", "Conectado: %1$@"),
    "account.notInvited": ("Ce compte n'est pas encore invité : c'est l'adresse ci-dessus qu'il faut ajouter à la liste. Pour qu'Apple en donne une autre, se déconnecter ne suffit pas — il garde le choix fait à la première connexion. Passez par Réglages › votre nom › Se connecter avec Apple › Kliima ‣ › Ne plus utiliser, puis reconnectez-vous en choisissant « Partager mon adresse e-mail ».",
                           "This account is not invited yet: the address above is the one to add to the list. To make Apple give a different one, signing out is not enough — it keeps the choice made at the first sign-in. Go to Settings › your name › Sign in with Apple › Kliima ‣ › Stop Using, then sign in again and choose “Share My Email”.",
                           "Esta cuenta aún no está invitada: la dirección de arriba es la que hay que añadir a la lista. Para que Apple dé otra, cerrar sesión no basta: conserva la elección hecha al primer inicio. Vaya a Ajustes › su nombre › Iniciar sesión con Apple › Kliima ‣ › Dejar de usar, y vuelva a iniciar sesión eligiendo «Compartir mi correo»."),
    "account.signOut": ("Se déconnecter", "Sign out", "Cerrar sesión"),
    "account.error.closed": ("Les comptes ne sont pas encore ouverts sur le serveur.",
                             "Accounts are not open on the server yet.",
                             "Las cuentas aún no están abiertas en el servidor."),
    "account.error.refused": ("Le serveur n'a pas reconnu la preuve d'Apple. Réessayez ; si cela persiste, signalez-le.",
                              "The server did not accept Apple's proof. Try again; if it persists, report it.",
                              "El servidor no reconoció la prueba de Apple. Inténtelo de nuevo; si persiste, avíselo."),
    "account.error.apple": ("Apple n'a pas ouvert la connexion (code %1$@).",
                            "Apple did not open the sign-in (code %1$@).",
                            "Apple no abrió el inicio de sesión (código %1$@)."),
    "account.active": ("Palier Pro ouvert par ce compte.", "Pro tier opened by this account.",
                       "Plan Pro abierto por esta cuenta."),
    "account.error.unreachable": ("Serveur injoignable pour l'instant. Réessayez plus tard.",
                                  "Server unreachable for now. Try again later.",
                                  "Servidor inaccesible por ahora. Inténtelo más tarde."),
    "tile.locked": ("Avec Kliima ‣ Pro", "With Kliima ‣ Pro", "Con Kliima ‣ Pro"),

    "weather.title": ("Météo", "Weather", "Tiempo"),
    "weather.widgetDescription": ("Le temps qu’il fait dans votre ville.", "The weather in your city.", "El tiempo en su ciudad."),
    "weather.feelsLike": ("Ressenti %1$@", "Feels like %1$@", "Sensación %1$@"),
    "weather.follow": ("Suivre la météo", "Follow the weather", "Seguir el tiempo"),
    "weather.stop": ("Arrêter le suivi météo", "Stop following", "Dejar de seguir"),
    "widget.unavailable": ("Prévision indisponible", "Forecast unavailable", "Previsión no disponible"),

    "tile.wind": ("Vent", "Wind", "Viento"),
    "tile.sunrise": ("Lever", "Sunrise", "Amanecer"),
    "tile.sunrise.caption": ("Coucher à %1$@.", "Sunset at %1$@.", "Anochecer a las %1$@."),

    # --- La ville : pluie, conseils, soleil, air ---
    "rain.title": ("Pluie", "Rain", "Lluvia"),
    "rain.none": ("Pas de pluie d’ici %1$@\u00a0h", "No rain for %1$@\u00a0h", "Sin lluvia en %1$@\u00a0h"),
    "rain.now": ("Il pleut — accalmie vers %1$@", "Raining — easing around %1$@", "Llueve — amaina hacia las %1$@"),
    "rain.nowLasting": ("Il pleut, et pour un moment", "Raining, and set to last", "Llueve, y para rato"),
    "rain.soon": ("Pluie vers %1$@", "Rain around %1$@", "Lluvia hacia las %1$@"),
    "rain.detail": ("%1$@ de risque · %2$@", "%1$@ chance · %2$@", "%1$@ de probabilidad · %2$@"),
    "advice.title": ("À emporter", "Take with you", "Para llevar"),
    "advice.none": ("Rien de particulier : profitez-en.", "Nothing special: enjoy it.", "Nada especial: aprovéchelo."),
    "advice.parapluie": ("Un parapluie", "An umbrella", "Un paraguas"),
    "advice.manteau": ("Un manteau", "A coat", "Un abrigo"),
    "advice.cremeSolaire": ("De la crème solaire", "Sunscreen", "Crema solar"),
    "advice.lunettes": ("Des lunettes de soleil", "Sunglasses", "Gafas de sol"),
    "advice.hydratation": ("De l’eau : il va faire chaud", "Water: it will be hot", "Agua: va a hacer calor"),
    "advice.gel": ("Prudence : trottoirs gelés possibles", "Careful: icy pavements possible", "Cuidado: aceras heladas posibles"),
    "advice.vent": ("Rafales : gare au parapluie", "Gusts: mind your umbrella", "Rachas: cuidado con el paraguas"),
    "uv.faible": ("Faible", "Low", "Bajo"),
    "uv.modere": ("Modéré", "Moderate", "Moderado"),
    "uv.eleve": ("Élevé", "High", "Alto"),
    "uv.tresEleve": ("Très élevé", "Very high", "Muy alto"),
    "uv.extreme": ("Extrême", "Extreme", "Extremo"),
    "air.bonne": ("Bonne", "Good", "Buena"),
    "air.correcte": ("Correcte", "Fair", "Razonable"),
    "air.moyenne": ("Moyenne", "Moderate", "Regular"),
    "air.mediocre": ("Médiocre", "Poor", "Mala"),
    "air.tresMediocre": ("Très médiocre", "Very poor", "Muy mala"),
    "air.extremementMediocre": ("Extrêmement médiocre", "Extremely poor", "Extremadamente mala"),
    "pollen.aulne": ("Aulne", "Alder", "Aliso"),
    "pollen.bouleau": ("Bouleau", "Birch", "Abedul"),
    "pollen.graminees": ("Graminées", "Grass", "Gramíneas"),
    "pollen.armoise": ("Armoise", "Mugwort", "Artemisa"),
    "pollen.olivier": ("Olivier", "Olive", "Olivo"),
    "pollen.ambroisie": ("Ambroisie", "Ragweed", "Ambrosía"),
    "pollenLevel.faible": ("Faible", "Low", "Bajo"),
    "pollenLevel.modere": ("Modéré", "Moderate", "Moderado"),
    "pollenLevel.eleve": ("Élevé", "High", "Alto"),
    "pollenLevel.tresEleve": ("Très élevé", "Very high", "Muy alto"),

    # --- Tuiles ---
    "tile.wind.caption": ("Rafales %1$@.", "Gusts %1$@.", "Rachas %1$@."),
    "tile.feelsLike": ("Ressenti", "Feels like", "Sensación"),
    "tile.feelsLike.caption": ("Il fait %1$@ au thermomètre.", "The thermometer reads %1$@.", "El termómetro marca %1$@."),
    "tile.humidity": ("Humidité", "Humidity", "Humedad"),
    "tile.humidity.caption": ("Point de rosée %1$@.", "Dew point %1$@.", "Punto de rocío %1$@."),
    "tile.uv": ("Indice UV", "UV index", "Índice UV"),
    "tile.uv.caption": ("Jusqu’à %1$@ aujourd’hui (%2$@).", "Up to %1$@ today (%2$@).", "Hasta %1$@ hoy (%2$@)."),
    "tile.pressure": ("Pression", "Pressure", "Presión"),
    "tile.pressure.caption": ("Ramenée au niveau de la mer.", "At sea level.", "Reducida al nivel del mar."),
    "tile.rainToday": ("Pluie du jour", "Rain today", "Lluvia de hoy"),
    "tile.rainToday.caption": ("Risque maximal %1$@.", "Highest chance %1$@.", "Probabilidad máxima %1$@."),
    "tile.air": ("Qualité de l’air", "Air quality", "Calidad del aire"),
    "tile.air.caption": ("Indice européen %1$@ · particules fines %2$@.", "European index %1$@ · fine particles %2$@.", "Índice europeo %1$@ · partículas finas %2$@."),
    "tile.pollen": ("Pollens", "Pollen", "Polen"),
    "tile.pollen.caption": ("%1$@ grains/m³ — %2$@.", "%1$@ grains/m³ — %2$@.", "%1$@ granos/m³ — %2$@."),
    "tile.pollen.none": ("Aucun", "None", "Ninguno"),
    "tile.pollen.noneCaption": ("Rien de notable dans l’air.", "Nothing notable in the air.", "Nada destacable en el aire."),

    # --- Alertes : ce que Kliima ‣ dit sans qu'on ouvre l'application ---
    "alert.pluie.title": ("Pluie imminente", "Rain on its way", "Lluvia inminente"),
    "alert.pluie.body": ("Elle arrive d’ici peu : %1$@ de risque.", "Arriving shortly: %1$@ chance.", "Llega en breve: %1$@ de probabilidad."),
    "alert.orage.title": ("Orage en approche", "Thunderstorm approaching", "Tormenta en camino"),
    "alert.orage.body": ("Un orage est prévu dans les prochaines heures.", "A thunderstorm is expected in the coming hours.", "Se espera una tormenta en las próximas horas."),
    "alert.gel.title": ("Gel annoncé", "Frost ahead", "Helada prevista"),
    "alert.gel.body": ("Jusqu’à %1$@ : trottoirs et pare-brise gelés.", "Down to %1$@: icy pavements and windscreens.", "Hasta %1$@: aceras y parabrisas helados."),
    "alert.chaleur.title": ("Forte chaleur", "Intense heat", "Calor intenso"),
    "alert.chaleur.body": ("Jusqu’à %1$@ : buvez, cherchez l’ombre.", "Up to %1$@: drink, find shade.", "Hasta %1$@: beba agua, busque la sombra."),
    "alert.vent.title": ("Vent violent", "Strong wind", "Viento fuerte"),
    "alert.vent.body": ("Rafales jusqu’à %1$@.", "Gusts up to %1$@.", "Rachas de hasta %1$@."),
    "plan.reason.air": ("La qualité de l’air et les pollens demandent l’abonnement.", "Air quality and pollen need the subscription.", "La calidad del aire y el polen requieren la suscripción."),
    "plan.feature.air": ("Qualité de l’air et pollens", "Air quality and pollen", "Calidad del aire y polen"),

    # --- Mes villes : une au palier libre, autant qu'on veut avec Kliima ‣ Pro ---
    "villes.title": ("Mes villes", "My cities", "Mis ciudades"),
    "villes.current": ("Ville affichée", "Showing now", "Ciudad mostrada"),
    "villes.save": ("Enregistrer", "Save", "Guardar"),
    "villes.saved": ("Enregistrée", "Saved", "Guardada"),
    "villes.empty": ("Aucune ville enregistrée. Cherchez-en une, puis enregistrez-la ici.", "No saved cities yet. Search for one, then save it here.", "Ninguna ciudad guardada. Busque una y guárdela aquí."),
    "villes.locked": ("Fermée — elle revient avec Kliima ‣ Pro", "Locked — back with Kliima ‣ Pro", "Cerrada — vuelve con Kliima ‣ Pro"),
    "villes.free": ("Le palier libre garde une ville. Kliima ‣ Pro en garde autant que vous voulez, chacune à un geste.", "The free tier keeps one city. Kliima ‣ Pro keeps as many as you like, each one tap away.", "El nivel gratuito guarda una ciudad. Kliima ‣ Pro guarda tantas como quiera, cada una a un toque."),
    "villes.pro": ("Kliima ‣ Pro : autant de villes que vous voulez. Glissez pour retirer, « Modifier » pour réordonner.", "Kliima ‣ Pro: as many cities as you like. Swipe to remove, “Edit” to reorder.", "Kliima ‣ Pro: tantas ciudades como quiera. Deslice para quitar, «Editar» para reordenar."),
    "villes.done": ("OK", "Done", "Listo"),

    # --- La prévision recoupée de toutes les sources ---
    "forecast.blended": ("Recoupée de %1$@ sources", "Blended from %1$@ sources", "Combinada de %1$@ fuentes"),
    "ciel.observed": (
        "Vu à l’aéroport de %1$@ (%2$@\u00a0km) à %3$@ : %4$@.",
        "Seen at %1$@ airport (%2$@\u00a0km) at %3$@: %4$@.",
        "Visto en el aeropuerto de %1$@ (%2$@\u00a0km) a las %3$@: %4$@.",
    ),
    "ciel.credit": (
        "Relevé d’aéroport (METAR) — NOAA, domaine public",
        "Airport report (METAR) — NOAA, public domain",
        "Parte de aeropuerto (METAR) — NOAA, dominio público",
    ),

    # --- Apparence : celle du téléphone, claire ou sombre ---
    "theme.title": ("Apparence", "Appearance", "Apariencia"),
    "theme.auto": ("Automatique", "Automatic", "Automática"),
    "theme.light": ("Clair", "Light", "Clara"),
    "theme.dark": ("Sombre", "Dark", "Oscura"),

    # --- Le guetteur : la demi-heure en cours et les deux heures à venir ---
    "veille.title": ("Le guetteur", "The lookout", "El vigía"),
    "veille.subtitle": ("Il relit le ciel tous les quarts d’heure.", "It rereads the sky every fifteen minutes.", "Relee el cielo cada cuarto de hora."),
    "veille.now": ("Les 30 minutes en cours", "The next 30 minutes", "Los próximos 30 minutos"),
    "veille.next": ("Les 2 heures à venir", "The next 2 hours", "Las próximas 2 horas"),
    "veille.now.dry": ("Pas une goutte d’ici une demi-heure.", "Not a drop for the next half hour.", "Ni una gota en la próxima media hora."),
    "veille.now.starts": ("À partir de %1$@ : %2$@.", "From %1$@: %2$@.", "A partir de las %1$@: %2$@."),
    "veille.now.continues": ("Ça tombe, et toute la demi-heure : %1$@.", "It’s coming down, and will for the whole half hour: %1$@.", "Está cayendo, y seguirá toda la media hora: %1$@."),
    "veille.now.stops": ("Ça tombe encore, mais ça s’arrête vers %1$@.", "Still coming down, but it stops around %1$@.", "Todavía cae, pero para hacia las %1$@."),
    "veille.next.dry": ("Sec jusqu’à %1$@ au moins.", "Dry until at least %1$@.", "Seco hasta al menos las %1$@."),
    "veille.next.episode": ("Entre %1$@ et %2$@ : %3$@, %4$@.", "Between %1$@ and %2$@: %3$@, %4$@.", "Entre las %1$@ y las %2$@: %3$@, %4$@."),
    "veille.next.episodeOpen": ("À partir de %1$@ : %3$@, et encore après %2$@.", "From %1$@: %3$@, still going after %2$@.", "A partir de las %1$@: %3$@, y sigue después de las %2$@."),
    "veille.next.persists": ("Ça ne s’arrête pas avant %1$@ : %2$@, %3$@ en tout.", "No let-up before %1$@: %2$@, %3$@ in all.", "No para antes de las %1$@: %2$@, %3$@ en total."),
    "veille.next.lull": ("Fin vers %1$@, puis sec jusqu’à %2$@.", "Ends around %1$@, then dry until %2$@.", "Termina hacia las %1$@, luego seco hasta las %2$@."),
    "veille.next.lullReturn": ("Fin vers %1$@, mais ça reprend vers %2$@.", "Ends around %1$@, but it comes back around %2$@.", "Termina hacia las %1$@, pero vuelve hacia las %2$@."),
    "veille.gusts": ("Rafales jusqu’à %1$@ vers %2$@.", "Gusts up to %1$@ around %2$@.", "Rachas de hasta %1$@ hacia las %2$@."),
    "veille.temperature": ("%1$@ à %2$@, ressenti %3$@.", "%1$@ at %2$@, feels like %3$@.", "%1$@ a las %2$@, sensación de %3$@."),
    "veille.checked": ("Relu à %1$@ · prochaine lecture à %2$@", "Checked at %1$@ · next check at %2$@", "Leído a las %1$@ · próxima lectura a las %2$@"),
    "veille.unavailable": ("Pas de prévision au quart d’heure pour cette ville en ce moment.", "No quarter-hourly forecast for this city right now.", "No hay previsión por cuartos de hora para esta ciudad ahora mismo."),
    "veille.loading": ("Le guetteur regarde le ciel…", "The lookout is checking the sky…", "El vigía está mirando el cielo…"),
    "veille.chart": ("Précipitations au quart d’heure, sur deux heures", "Precipitation every fifteen minutes, over two hours", "Precipitación cada cuarto de hora, durante dos horas"),
    "veille.kind.pluie.faible": ("pluie faible", "light rain", "lluvia débil"),
    "veille.kind.pluie.moderee": ("pluie modérée", "moderate rain", "lluvia moderada"),
    "veille.kind.pluie.forte": ("forte pluie", "heavy rain", "lluvia fuerte"),
    "veille.kind.neige.faible": ("neige faible", "light snow", "nieve débil"),
    "veille.kind.neige.moderee": ("neige modérée", "moderate snow", "nieve moderada"),
    "veille.kind.neige.forte": ("forte neige", "heavy snow", "nevada fuerte"),
    "veille.kind.orage": ("orage", "thunderstorm", "tormenta"),
}

INFO_PLIST = {
    "CFBundleDisplayName": ("Kliima ‣", "Kliima ‣", "Kliima ‣"),
    "NSLocationWhenInUseUsageDescription": (
        "Votre position sert à afficher la météo de la ville où vous êtes.",
        "Your location is used to show the weather where you are.",
        "Su ubicación sirve para mostrar el tiempo de la ciudad donde está.",
    ),
}


def catalog(entries):
    strings = {}
    for key, (fr, en, es) in entries.items():
        strings[key] = {
            "extractionState": "manual",
            "localizations": {
                lang: {"stringUnit": {"state": "translated", "value": value}}
                for lang, value in (("en", en), ("es", es), ("fr", fr))
            },
        }
    return {"sourceLanguage": "fr", "strings": dict(sorted(strings.items())), "version": "1.0"}


def write(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as fh:
        json.dump(data, fh, ensure_ascii=False, indent=2)
        fh.write("\n")
    print(f"{path} : {len(data['strings'])} clés × 3 langues")


base = sys.argv[1]
write(os.path.join(base, "Localizable.xcstrings"), catalog(STRINGS))
write(os.path.join(base, "InfoPlist.xcstrings"), catalog(INFO_PLIST))
