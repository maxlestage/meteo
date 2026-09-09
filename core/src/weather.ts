/**
 * Traduction des codes météo WMO renvoyés par Open-Meteo.
 *
 * Le code renvoie une clé, pas un libellé : le texte affiché dépend de la
 * langue et vit dans les catalogues (`sharedMessages`, et leur équivalent iOS).
 * La même table est implémentée dans `ios/Klima/Models/WeatherCondition.swift`.
 */

/** Famille de pictogramme, déclinée jour / nuit à l'affichage. */
export type ConditionIcon =
  | 'clear'
  | 'partly'
  | 'cloudy'
  | 'fog'
  | 'drizzle'
  | 'rain'
  | 'showers'
  | 'snow'
  | 'thunder'

export interface WeatherCondition {
  /** Clé de catalogue, par exemple « wmo.drizzle ». */
  labelKey: string
  icon: ConditionIcon
}

const condition = (name: string, icon: ConditionIcon): WeatherCondition => ({
  labelKey: `wmo.${name}`,
  icon,
})

const CONDITIONS: ReadonlyMap<number, WeatherCondition> = new Map([
  [0, condition('clearSky', 'clear')],
  [1, condition('mainlyClear', 'partly')],
  [2, condition('partlyCloudy', 'partly')],
  [3, condition('overcast', 'cloudy')],
  [45, condition('fog', 'fog')],
  [48, condition('rimeFog', 'fog')],
  [51, condition('lightDrizzle', 'drizzle')],
  [53, condition('drizzle', 'drizzle')],
  [55, condition('denseDrizzle', 'drizzle')],
  [56, condition('freezingDrizzle', 'drizzle')],
  [57, condition('denseFreezingDrizzle', 'drizzle')],
  [61, condition('slightRain', 'rain')],
  [63, condition('rain', 'rain')],
  [65, condition('heavyRain', 'rain')],
  [66, condition('freezingRain', 'rain')],
  [67, condition('heavyFreezingRain', 'rain')],
  [71, condition('slightSnow', 'snow')],
  [73, condition('snow', 'snow')],
  [75, condition('heavySnow', 'snow')],
  [77, condition('snowGrains', 'snow')],
  [80, condition('showers', 'showers')],
  [81, condition('moderateShowers', 'showers')],
  [82, condition('violentShowers', 'showers')],
  [85, condition('snowShowers', 'snow')],
  [86, condition('heavySnowShowers', 'snow')],
  [95, condition('thunderstorm', 'thunder')],
  [96, condition('thunderstormHail', 'thunder')],
  [99, condition('thunderstormHeavyHail', 'thunder')],
])

const UNKNOWN = condition('overcast', 'cloudy')

/** Clé de libellé et pictogramme d'un code WMO. Un code inconnu retombe sur « couvert ». */
export function weatherCondition(code: number): WeatherCondition {
  return CONDITIONS.get(code) ?? UNKNOWN
}
