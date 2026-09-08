/**
 * Traduction des codes météo WMO renvoyés par Open-Meteo.
 *
 * La même table est implémentée côté iOS
 * (ios/Klima/Models/WeatherCondition.swift).
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
  label: string
  icon: ConditionIcon
}

const CONDITIONS: ReadonlyMap<number, WeatherCondition> = new Map([
  [0, { label: 'Ciel dégagé', icon: 'clear' }],
  [1, { label: 'Peu nuageux', icon: 'partly' }],
  [2, { label: 'Partiellement nuageux', icon: 'partly' }],
  [3, { label: 'Couvert', icon: 'cloudy' }],
  [45, { label: 'Brouillard', icon: 'fog' }],
  [48, { label: 'Brouillard givrant', icon: 'fog' }],
  [51, { label: 'Bruine légère', icon: 'drizzle' }],
  [53, { label: 'Bruine', icon: 'drizzle' }],
  [55, { label: 'Bruine dense', icon: 'drizzle' }],
  [56, { label: 'Bruine verglaçante', icon: 'drizzle' }],
  [57, { label: 'Bruine verglaçante dense', icon: 'drizzle' }],
  [61, { label: 'Pluie faible', icon: 'rain' }],
  [63, { label: 'Pluie', icon: 'rain' }],
  [65, { label: 'Pluie forte', icon: 'rain' }],
  [66, { label: 'Pluie verglaçante', icon: 'rain' }],
  [67, { label: 'Pluie verglaçante forte', icon: 'rain' }],
  [71, { label: 'Neige faible', icon: 'snow' }],
  [73, { label: 'Neige', icon: 'snow' }],
  [75, { label: 'Neige forte', icon: 'snow' }],
  [77, { label: 'Grains de neige', icon: 'snow' }],
  [80, { label: 'Averses', icon: 'showers' }],
  [81, { label: 'Averses modérées', icon: 'showers' }],
  [82, { label: 'Averses violentes', icon: 'showers' }],
  [85, { label: 'Averses de neige', icon: 'snow' }],
  [86, { label: 'Averses de neige fortes', icon: 'snow' }],
  [95, { label: 'Orage', icon: 'thunder' }],
  [96, { label: 'Orage et grêle', icon: 'thunder' }],
  [99, { label: 'Orage et forte grêle', icon: 'thunder' }],
])

/** Libellé et pictogramme d'un code WMO. Un code inconnu retombe sur « Couvert ». */
export function weatherCondition(code: number): WeatherCondition {
  return CONDITIONS.get(code) ?? { label: 'Couvert', icon: 'cloudy' }
}
