import { describe, expect, test } from 'bun:test'
import { weatherCondition } from './weather'

describe('codes météo WMO', () => {
  test('ciel dégagé', () => {
    expect(weatherCondition(0)).toEqual({ label: 'Ciel dégagé', icon: 'clear' })
  })

  test('bruine, comme sur la côte basque', () => {
    expect(weatherCondition(53).label).toBe('Bruine')
    expect(weatherCondition(53).icon).toBe('drizzle')
  })

  test('les trois intensités de pluie partagent le même pictogramme', () => {
    expect([61, 63, 65].map((c) => weatherCondition(c).icon)).toEqual(['rain', 'rain', 'rain'])
  })

  test('orage avec grêle', () => {
    expect(weatherCondition(96).icon).toBe('thunder')
  })

  test('code inconnu : repli sans planter', () => {
    expect(weatherCondition(42)).toEqual({ label: 'Couvert', icon: 'cloudy' })
  })
})
