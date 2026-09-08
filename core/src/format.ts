/**
 * Nombres à la française : virgule décimale, et espace insécable avant l'unité
 * pour qu'un « 4,8 mm » ne se coupe jamais en fin de ligne.
 */

const NBSP = ' '

/** « 4,8 » */
export function decimal(value: number, digits = 1): string {
  return value.toLocaleString('fr-FR', {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  })
}

/** « 4,8 mm » */
export function withUnit(value: number, unit: string, digits = 1): string {
  return `${decimal(value, digits)}${NBSP}${unit}`
}

/** « +2,7 mm » ou « -1,2 mm » : le signe rend le bilan lisible d'un coup d'œil. */
export function signedWithUnit(value: number, unit: string, digits = 1): string {
  const sign = value > 0 ? '+' : ''
  return `${sign}${withUnit(value, unit, digits)}`
}

/** « 27 % » — les pourcentages n'ont pas de décimale ici. */
export function percent(value: number): string {
  return `${Math.round(value)}${NBSP}%`
}
