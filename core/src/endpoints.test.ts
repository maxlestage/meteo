import { afterEach, describe, expect, test } from 'bun:test'
import { DIRECT_ENDPOINTS, endpoints, relayEndpoints, useDirectProviders, useRelay } from './endpoints'

afterEach(useDirectProviders)

describe('acheminement', () => {
  test('en direct par défaut', () => {
    expect(endpoints()).toBe(DIRECT_ENDPOINTS)
    expect(endpoints().transport).toBe('direct')
  })

  test('le relais remplace les quatre adresses d’un coup', () => {
    useRelay('https://relais.klima')
    const via = endpoints()

    expect(via.transport).toBe('relais')
    for (const url of [via.openMeteoForecast, via.openMeteoSearch, via.metNorway, via.brightSky]) {
      expect(url.startsWith('https://relais.klima/')).toBe(true)
    }
    // Aucune adresse de fournisseur ne subsiste : tout passe par le relais.
    expect(JSON.stringify(via)).not.toContain('open-meteo.com')
    expect(JSON.stringify(via)).not.toContain('met.no')
  })

  test('une barre oblique en trop ne double pas dans l’URL', () => {
    expect(relayEndpoints('https://relais.klima//').openMeteoForecast)
      .toBe('https://relais.klima/v1/open-meteo/forecast')
  })

  test('on peut revenir au direct', () => {
    useRelay('https://relais.klima')
    useDirectProviders()
    expect(endpoints().transport).toBe('direct')
  })
})
