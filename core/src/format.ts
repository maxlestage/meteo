/**
 * Mise en forme des nombres selon la langue : la virgule décimale du français
 * et de l'espagnol, le point de l'anglais, et l'espace insécable devant l'unité
 * là où la typographie l'exige — le tout délégué à `Intl`.
 */
import { LOCALES, type Language, type Translate } from './i18n'
import type { SprayBlocker } from './agro'

const NBSP = ' '

export interface Formats {
  /** « 4,8 » en français, « 4.8 » en anglais. */
  decimal(value: number, digits?: number): string
  /** « 4,8 mm » */
  unit(value: number, unit: string, digits?: number): string
  /** « +2,7 mm » : le signe rend un bilan lisible d'un coup d'œil. */
  signedUnit(value: number, unit: string, digits?: number): string
  /** « 27 % » — ponctuation propre à chaque langue. */
  percent(value: number): string
  /** « 16° », arrondi comme sur un bulletin météo. */
  temperature(value: number): string
}

export function formats(language: Language): Formats {
  const locale = LOCALES[language]

  const decimal = (value: number, digits = 1) =>
    value.toLocaleString(locale, {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    })

  const unit = (value: number, unitLabel: string, digits = 1) =>
    `${decimal(value, digits)}${NBSP}${unitLabel}`

  return {
    decimal,
    unit,
    signedUnit: (value, unitLabel, digits = 1) =>
      `${value > 0 ? '+' : ''}${unit(value, unitLabel, digits)}`,
    percent: (value) =>
      (value / 100).toLocaleString(locale, { style: 'percent', maximumFractionDigits: 0 }),
    temperature: (value) => `${Math.round(value)}°`,
  }
}

/** Formule un motif de blocage dans la langue courante, unités comprises. */
export function describeBlocker(blocker: SprayBlocker, t: Translate, f: Formats): string {
  switch (blocker.kind) {
    case 'windTooStrong':
      return t('spray.windTooStrong', {
        wind: f.unit(blocker.wind, 'km/h', 0),
        limit: f.unit(blocker.limit, 'km/h', 0),
      })
    case 'windTooWeak':
      return t('spray.windTooWeak')
    case 'gusts':
      return t('spray.gusts', { gusts: f.unit(blocker.gusts, 'km/h', 0) })
    case 'rain':
      return t('spray.rain', { amount: f.unit(blocker.amount, 'mm') })
    case 'tooHot':
      return t('spray.tooHot', { temperature: f.unit(blocker.temperature, '°C', 0) })
    case 'tooCold':
      return t('spray.tooCold', { temperature: f.unit(blocker.temperature, '°C', 0) })
    case 'dryAir':
      return t('spray.dryAir', { humidity: f.percent(blocker.humidity) })
    case 'vapourPressureDeficit':
      return t('spray.vapourPressureDeficit', { vpd: f.unit(blocker.vpd, 'kPa', 2) })
  }
}
