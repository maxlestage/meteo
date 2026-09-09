import { describe, expect, test } from 'bun:test'
import { weatherCondition } from './weather'
import { sharedMessages } from './messages'
import { LANGUAGES } from './i18n'

describe('codes météo WMO', () => {
  test('ciel dégagé', () => {
    expect(weatherCondition(0)).toEqual({ labelKey: 'wmo.clearSky', icon: 'clear' })
  })

  test('bruine, comme sur la côte basque', () => {
    expect(weatherCondition(53).labelKey).toBe('wmo.drizzle')
    expect(weatherCondition(53).icon).toBe('drizzle')
  })

  test('les trois intensités de pluie partagent le même pictogramme', () => {
    expect([61, 63, 65].map((c) => weatherCondition(c).icon)).toEqual(['rain', 'rain', 'rain'])
  })

  test('orage avec grêle', () => {
    expect(weatherCondition(96).icon).toBe('thunder')
  })

  test('code inconnu : repli sans planter', () => {
    expect(weatherCondition(42)).toEqual({ labelKey: 'wmo.overcast', icon: 'cloudy' })
  })

  test('tout code documenté est traduit dans les trois langues', () => {
    const codes = [0, 1, 2, 3, 45, 48, 51, 53, 55, 56, 57, 61, 63, 65, 66, 67, 71, 73, 75, 77, 80,
      81, 82, 85, 86, 95, 96, 99]
    for (const code of codes) {
      const { labelKey } = weatherCondition(code)
      for (const language of LANGUAGES) {
        expect(sharedMessages[language][labelKey]).toBeTruthy()
      }
    }
  })
})
