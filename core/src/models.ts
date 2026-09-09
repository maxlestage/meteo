/**
 * Les modèles de prévision interrogés pour se recouper.
 *
 * Open-Meteo redistribue les sorties brutes de plusieurs services météo
 * nationaux. Les comparer, c'est comparer de vraies sources indépendantes —
 * quatre centres de calcul différents, pas quatre habillages du même modèle.
 */

export interface WeatherModel {
  /** Identifiant Open-Meteo, passé au paramètre `models`. */
  id: string
  /** Nom du modèle, tel que le nomme son service. */
  name: string
  /** Service qui le produit. */
  institution: string
  /** Code pays du service, pour le drapeau affiché. */
  country: string
}

export const WEATHER_MODELS: readonly WeatherModel[] = [
  {
    id: 'meteofrance_seamless',
    name: 'AROME / ARPEGE',
    institution: 'Météo-France',
    country: 'FR',
  },
  {
    id: 'ecmwf_ifs025',
    name: 'IFS',
    institution: 'ECMWF',
    country: 'EU',
  },
  {
    id: 'icon_seamless',
    name: 'ICON',
    institution: 'Deutscher Wetterdienst',
    country: 'DE',
  },
  {
    id: 'gfs_seamless',
    name: 'GFS',
    institution: 'NOAA',
    country: 'US',
  },
]

export function weatherModel(id: string): WeatherModel | undefined {
  return WEATHER_MODELS.find((model) => model.id === id)
}
