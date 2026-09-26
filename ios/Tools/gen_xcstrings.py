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
    "soil.sature": ("Saturé", "Waterlogged", "Encharcado"),
    "soil.ressuye": ("Ressuyé", "Drained", "Oreado"),
    "soil.sec": ("Sec", "Dry", "Seco"),
    "soil.trafficable": ("Portance correcte", "Bears machinery", "Soporta la maquinaria"),
    "soil.compaction": ("Risque de tassement", "Compaction risk", "Riesgo de compactación"),
    "water.deficit": ("Déficit", "Deficit", "Déficit"),
    "water.equilibre": ("Équilibre", "Balanced", "Equilibrio"),
    "water.excedent": ("Excédent", "Surplus", "Excedente"),
    "frost.aucun": ("Aucun", "None", "Ninguna"),
    "frost.faible": ("Faible", "Slight", "Leve"),
    "frost.modere": ("Modéré", "Moderate", "Moderada"),
    "frost.severe": ("Sévère", "Severe", "Severa"),
    "frost.hoarFrost": ("gelée blanche probable", "hoar frost likely", "escarcha probable"),
    "disease.faible": ("Faible", "Low", "Baja"),
    "disease.moyenne": ("Moyenne", "Moderate", "Media"),
    "disease.elevee": ("Élevée", "High", "Alta"),
    "spray.favorable": ("Favorable", "Favourable", "Favorable"),
    "spray.acceptable": ("Acceptable", "Acceptable", "Aceptable"),
    "spray.defavorable": ("Défavorable", "Unsuitable", "Desfavorable"),

    # --- Motifs de blocage ---
    "spray.windTooStrong": ("Vent %1$@ (max %2$@)", "Wind %1$@ (limit %2$@)", "Viento %1$@ (máx. %2$@)"),
    "spray.windTooWeak": ("Vent trop faible, risque d’inversion thermique",
                          "Wind too light, risk of thermal inversion",
                          "Viento demasiado flojo, riesgo de inversión térmica"),
    "spray.gusts": ("Rafales %1$@", "Gusts %1$@", "Rachas %1$@"),
    "spray.rain": ("Pluie %1$@ dans les 2 h", "Rain %1$@ within 2 h", "Lluvia %1$@ en 2 h"),
    "spray.tooHot": ("Température %1$@, trop chaud", "Temperature %1$@, too warm", "Temperatura %1$@, demasiado calor"),
    "spray.tooCold": ("Température %1$@, trop froid", "Temperature %1$@, too cold", "Temperatura %1$@, demasiado frío"),
    "spray.dryAir": ("Hygrométrie %1$@, air trop sec", "Humidity %1$@, air too dry", "Humedad %1$@, aire demasiado seco"),
    "spray.vapourPressureDeficit": ("VPD %1$@, évaporation des gouttelettes",
                                    "VPD %1$@, droplets evaporate",
                                    "DPV %1$@, las gotas se evaporan"),

    # --- Service ---
    "api.unreachable": ("Service météo injoignable. Vérifiez votre connexion.",
                        "Weather service unreachable. Check your connection.",
                        "Servicio meteorológico inaccesible. Compruebe su conexión."),
    "api.status": ("Le service météo a répondu %1$@.", "The weather service replied %1$@.",
                   "El servicio meteorológico ha respondido %1$@."),
    "api.malformed": ("Réponse illisible du service météo.", "Unreadable response from the weather service.",
                      "Respuesta ilegible del servicio meteorológico."),
    "location.denied": ("Localisation refusée. Recherchez la commune à la main.",
                        "Location denied. Search for the town instead.",
                        "Ubicación denegada. Busque el municipio a mano."),
    "location.unavailable": ("Position indisponible pour le moment.", "Location unavailable right now.",
                             "Ubicación no disponible por ahora."),

    # --- Application ---
    "app.loading": ("Chargement des données agronomiques…", "Loading agronomic data…",
                    "Cargando los datos agronómicos…"),
    "app.retry": ("Réessayer", "Try again", "Reintentar"),
    "app.locate": ("Me localiser", "Locate me", "Ubicarme"),
    "app.search": ("Rechercher une commune", "Search for a town", "Buscar un municipio"),
    "app.error": ("Impossible de charger la prévision agricole.", "Could not load the agricultural forecast.",
                  "No se ha podido cargar la previsión agrícola."),
    "app.source": ("Données Open-Meteo — modèle agricole : humidité et température du sol, ET0 FAO-56, déficit de pression de vapeur. Parcelle à %1$@.",
                   "Open-Meteo data — agricultural model: soil moisture and temperature, FAO-56 ET0, vapour pressure deficit. Field at %1$@.",
                   "Datos de Open-Meteo — modelo agrícola: humedad y temperatura del suelo, ET0 FAO-56, déficit de presión de vapor. Parcela a %1$@."),
    "search.myField": ("Ma parcelle", "My field", "Mi parcela"),

    "hourly.title": ("Conditions météo", "Conditions", "Condiciones"),
    "hourly.now": ("Maint.", "Now", "Ahora"),
    "daily.title": ("Prévision sur 7 jours", "7-day forecast", "Previsión a 7 días"),
    "daily.today": ("Auj.", "Today", "Hoy"),

    "spray.title": ("Fenêtre de traitement", "Spraying window", "Ventana de tratamiento"),
    "spray.none": ("Aucune fenêtre sur 7 jours", "No window in the next 7 days", "Ninguna ventana en 7 días"),
    "spray.score": ("Score %1$@/100 sur la plage", "Score %1$@/100 over the window",
                    "Puntuación %1$@/100 en la franja"),
    "spray.mainBlocker": ("Blocage principal : %1$@", "Main obstacle: %1$@", "Principal impedimento: %1$@"),
    "spray.unsuitable": ("Conditions défavorables", "Conditions unsuitable", "Condiciones desfavorables"),
    "spray.now": ("Maintenant", "Now", "Ahora"),
    "spray.plus12": ("+12 h", "+12 h", "+12 h"),
    "spray.plus24": ("+24 h", "+24 h", "+24 h"),

    # --- Activité en direct ---
    "activity.window": ("Fenêtre", "Window", "Ventana"),
    "activity.allClear": ("Conditions réunies", "Conditions are right", "Condiciones favorables"),
    "activity.score": ("%1$@/100", "%1$@/100", "%1$@/100"),
    # --- Recoupement des modèles ---
    "consensus.title": ("Accord des modèles", "Model agreement", "Acuerdo de los modelos"),
    "consensus.forte": ("Fort", "Strong", "Fuerte"),
    "consensus.moyenne": ("Moyen", "Moderate", "Medio"),
    "consensus.faible": ("Faible", "Weak", "Débil"),
    "consensus.detail": ("%1$@ modèles · écart %2$@", "%1$@ models · %2$@ apart",
                         "%1$@ modelos · diferencia de %2$@"),
    "consensus.rainDisagreement": ("désaccord sur la pluie", "they disagree on rain",
                                   "discrepan sobre la lluvia"),
    "consensus.median": ("Valeur retenue : %1$@", "Value used: %1$@", "Valor retenido: %1$@"),

    # --- Alertes : ce que Kliima dit sans qu'on ouvre l'application ---
    "alert.fenetre.title": ("Fenêtre de traitement", "Spraying window", "Ventana de tratamiento"),
    "alert.fenetre.body": ("Conditions réunies, score %1$@/100 sur la plage.",
                           "Conditions are right, %1$@/100 over the window.",
                           "Condiciones reunidas, %1$@/100 en la franja."),
    "alert.gel.title": ("Gel cette nuit", "Frost tonight", "Helada esta noche"),
    "alert.gel.body": ("Jusqu'à %1$@ °C attendus.", "Down to %1$@ °C expected.",
                       "Hasta %1$@ °C previstos."),
    "alert.sol.title": ("Sol ressuyé", "Soil drained", "Suelo oreado"),
    "alert.sol.body": ("Humidité retombée à %1$@ : la parcelle porte.",
                       "Moisture back to %1$@: the field bears machinery.",
                       "Humedad de nuevo en %1$@: la parcela soporta la maquinaria."),
    "alert.pluie.title": ("Pluie après la fenêtre", "Rain after the window", "Lluvia tras la ventana"),
    "alert.pluie.body": ("%1$@ mm attendus : un traitement risque d'être lavé.",
                         "%1$@ mm expected: a treatment could be washed off.",
                         "%1$@ mm previstos: un tratamiento podría lavarse."),

    # --- Ce qu'un palier ferme ---
    "plan.libre": ("Kliima ‣", "Kliima ‣", "Kliima ‣"),
    "plan.pro": ("Kliima ‣ Pro", "Kliima ‣ Pro", "Kliima ‣ Pro"),
    "plan.reason.recoupement": ("Comparer plusieurs instituts demande l'abonnement.",
                                "Comparing several institutes needs the subscription.",
                                "Comparar varios institutos requiere la suscripción."),
    "plan.reason.alertes": ("Être prévenu sans ouvrir l'application demande l'abonnement.",
                            "Being warned without opening the app needs the subscription.",
                            "Recibir avisos sin abrir la aplicación requiere la suscripción."),
    "plan.reason.cumuls": ("Les cumuls depuis une date demandent l'abonnement.",
                           "Totals since a chosen date need the subscription.",
                           "Los acumulados desde una fecha requieren la suscripción."),
    "plan.reason.registre": ("L'export des conditions de traitement demande l'abonnement.",
                             "Exporting treatment conditions needs the subscription.",
                             "Exportar las condiciones de tratamiento requiere la suscripción."),

    # --- Écran d'abonnement ---
    "paywall.title": ("Kliima ‣ Pro", "Kliima ‣ Pro", "Kliima ‣ Pro"),
    "plan.feature.recoupement": ("Cinq instituts recoupés, et leur niveau d'accord",
                                    "Five institutes cross-checked, and how far they agree",
                                    "Cinco institutos contrastados y su nivel de acuerdo"),
    "plan.feature.alertes": ("Prévenu sans ouvrir l'application",
                                "Warned without opening the app",
                                "Avisado sin abrir la aplicación"),
    "plan.feature.cumuls": ("Cumuls depuis le semis ou le dernier traitement",
                               "Totals since sowing or the last treatment",
                               "Acumulados desde la siembra o el último tratamiento"),
    "plan.feature.registre": ("Export des conditions à l'heure du traitement",
                                 "Export of the conditions at the hour of treatment",
                                 "Exportación de las condiciones a la hora del tratamiento"),
    "paywall.free": ("Une parcelle et la journée entière restent gratuites, sans compte ni publicité.",
                     "One field and the whole day stay free, no account and no adverts.",
                     "Una parcela y el día entero siguen siendo gratis, sin cuenta ni publicidad."),
    "paywall.buy": ("S'abonner — %1$@ par mois", "Subscribe — %1$@ a month", "Suscribirse — %1$@ al mes"),
    "paywall.restore": ("Restaurer un achat", "Restore a purchase", "Restaurar una compra"),
    "paywall.terms": ("Abonnement mensuel, renouvelé automatiquement, résiliable à tout moment depuis les réglages de l'App Store.",
                      "Monthly subscription, renewed automatically, cancellable at any time from your App Store settings.",
                      "Suscripción mensual, renovada automáticamente, cancelable en cualquier momento desde los ajustes de la App Store."),
    "paywall.close": ("Fermer", "Close", "Cerrar"),
    "paywall.unavailable": ("Boutique injoignable pour l'instant.", "Store unreachable for now.",
                            "Tienda no disponible por ahora."),
    "tile.locked": ("Avec Kliima ‣ Pro", "With Kliima ‣ Pro", "Con Kliima ‣ Pro"),

    "widget.description": ("La prochaine fenêtre de traitement sur votre parcelle.",
                           "The next spraying window on your field.",
                           "La próxima ventana de tratamiento en su parcela."),
    "widget.unavailable": ("Prévision indisponible", "Forecast unavailable", "Previsión no disponible"),
    "activity.follow": ("Suivre cette fenêtre", "Follow this window", "Seguir esta ventana"),
    "activity.stop": ("Arrêter le suivi", "Stop following", "Dejar de seguir"),

    "tile.soil": ("Humidité du sol", "Soil moisture", "Humedad del suelo"),
    "tile.soil.caption": ("%1$@ vol. · %2$@ à 6 cm. %3$@", "%1$@ vol. · %2$@ at 6 cm. %3$@",
                          "%1$@ vol. · %2$@ a 6 cm. %3$@"),
    "tile.water": ("Bilan hydrique", "Water balance", "Balance hídrico"),
    "tile.water.irrigation": ("Irrigation conseillée : %1$@ sur 7 jours.", "Irrigation advised: %1$@ over 7 days.",
                              "Riego aconsejado: %1$@ en 7 días."),
    "tile.water.caption": ("Pluie %1$@, ET0 %2$@ sur 7 jours.", "Rain %1$@, ET0 %2$@ over 7 days.",
                           "Lluvia %1$@, ET0 %2$@ en 7 días."),
    "tile.wind": ("Vent", "Wind", "Viento"),
    "tile.wind.caption": ("Rafales %1$@. Limite de pulvérisation : %2$@.",
                          "Gusts %1$@. Spraying limit: %2$@.", "Rachas %1$@. Límite de pulverización: %2$@."),
    "tile.frost": ("Risque de gel", "Frost risk", "Riesgo de helada"),
    "tile.frost.caption": ("Mini %1$@ cette nuit%2$@.", "Low of %1$@ tonight%2$@.",
                           "Mínima de %1$@ esta noche%2$@."),
    "tile.disease": ("Pression maladie", "Disease pressure", "Presión de enfermedad"),
    "tile.disease.caption": ("%1$@ h d’humectation du feuillage sur 24 h.",
                             "%1$@ h of leaf wetness over 24 h.", "%1$@ h de humectación foliar en 24 h."),
    "tile.gdd": ("Degrés-jours", "Growing degree days", "Grados-día"),
    "tile.gdd.caption": ("Cumul sur 7 jours, base %1$@.", "Cumulated over 7 days, base %1$@.",
                         "Acumulado en 7 días, base %1$@."),
    "tile.sunrise": ("Lever", "Sunrise", "Amanecer"),
    "tile.sunrise.caption": ("Coucher à %1$@.", "Sunset at %1$@.", "Anochecer a las %1$@."),
    "tile.sowing": ("Semis", "Drilling", "Siembra"),
    "tile.sowing.yes": ("Possible", "Possible", "Posible"),
    "tile.sowing.no": ("Déconseillé", "Not advised", "Desaconsejada"),
    "tile.sowing.caption": ("Sol à %1$@ à 6 cm ; il faut 8 °C et un sol ressuyé.",
                            "Soil at %1$@ at 6 cm; it needs 8 °C and drained soil.",
                            "Suelo a %1$@ a 6 cm; hacen falta 8 °C y suelo oreado."),
}

INFO_PLIST = {
    "CFBundleDisplayName": ("Kliima ‣", "Kliima ‣", "Kliima ‣"),
    "NSLocationWhenInUseUsageDescription": (
        "Votre position sert à caler la prévision agricole sur votre parcelle.",
        "Your location is used to centre the agricultural forecast on your field.",
        "Su ubicación sirve para centrar la previsión agrícola en su parcela.",
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
