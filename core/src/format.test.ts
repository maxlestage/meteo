import { describe, expect, test } from 'bun:test'
import { describeBlocker, formats } from './format'
import { sharedMessages } from './messages'
import { translator } from './i18n'

const NBSP = ' '

describe('nombres selon la langue', () => {
  test('virgule décimale en français et en espagnol, point en anglais', () => {
    expect(formats('fr').decimal(4.8)).toBe('4,8')
    expect(formats('es').decimal(4.8)).toBe('4,8')
    expect(formats('en').decimal(4.8)).toBe('4.8')
  })

  test('unité collée par une espace insécable', () => {
    expect(formats('fr').unit(2.1, 'mm')).toBe(`2,1${NBSP}mm`)
    expect(formats('en').unit(2.1, 'mm')).toBe(`2.1${NBSP}mm`)
  })

  test('le bilan porte son signe', () => {
    expect(formats('fr').signedUnit(2.7, 'mm')).toBe(`+2,7${NBSP}mm`)
    expect(formats('fr').signedUnit(-1.2, 'mm')).toBe(`-1,2${NBSP}mm`)
    // Un bilan nul n'est pas « positif » : pas de signe.
    expect(formats('fr').signedUnit(0, 'mm')).toBe(`0,0${NBSP}mm`)
  })

  test('pourcentages ponctués selon la langue', () => {
    expect(formats('fr').percent(26.6)).toBe(`27${NBSP}%`)
    expect(formats('en').percent(26.6)).toBe('27%')
  })

  test('températures arrondies', () => {
    expect(formats('fr').temperature(16.4)).toBe('16°')
    expect(formats('fr').temperature(-1.4)).toBe('-1°')
    // Un demi-degré sous zéro s'affiche « 0° », jamais « -0° ».
    expect(formats('en').temperature(-0.5)).toBe('0°')
  })
})

describe('motifs de blocage', () => {
  const t = (language: 'fr' | 'en' | 'es') => translator(language, sharedMessages)

  test('vent hors limite, dans les trois langues', () => {
    const blocker = { kind: 'windTooStrong', wind: 24, limit: 19 } as const
    expect(describeBlocker(blocker, t('fr'), formats('fr'))).toBe(
      `Vent 24${NBSP}km/h (max 19${NBSP}km/h)`,
    )
    expect(describeBlocker(blocker, t('en'), formats('en'))).toBe(
      `Wind 24${NBSP}km/h (limit 19${NBSP}km/h)`,
    )
    expect(describeBlocker(blocker, t('es'), formats('es'))).toBe(
      `Viento 24${NBSP}km/h (máx. 19${NBSP}km/h)`,
    )
  })

  test('motif sans valeur', () => {
    expect(describeBlocker({ kind: 'windTooWeak' }, t('en'), formats('en'))).toBe(
      'Wind too light, risk of thermal inversion',
    )
  })

  test('pluie et VPD portent leurs unités', () => {
    expect(describeBlocker({ kind: 'rain', amount: 1.4 }, t('fr'), formats('fr'))).toBe(
      `Pluie 1,4${NBSP}mm dans les 2 h`,
    )
    expect(
      describeBlocker({ kind: 'vapourPressureDeficit', vpd: 1.25 }, t('en'), formats('en')),
    ).toBe(`VPD 1.25${NBSP}kPa, droplets evaporate`)
  })
})
