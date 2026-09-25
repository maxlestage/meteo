import { afterEach, describe, expect, test } from 'bun:test'
import {
  DIRECT_ENDPOINTS,
  endpoints,
  MEME_ORIGINE,
  relayEndpoints,
  relayFrom,
  useDirectProviders,
  useRelay,
} from './endpoints'

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

describe('relayFrom', () => {

  test('rien de configuré : chaque navigateur pour soi', () => {
    expect(relayFrom(undefined, 'https://klima.example')).toBeNull()
    expect(relayFrom('', 'https://klima.example')).toBeNull()
    expect(relayFrom('   ', 'https://klima.example')).toBeNull()
  })

  test('une adresse configurée est prise telle quelle', () => {
    expect(relayFrom('https://relais.example', 'https://klima.example'))
      .toBe('https://relais.example')
  })

  // Le cas de l'hébergement unique : le relais sert la page, et son adresse
  // n'est connue qu'au moment de l'affichage.
  test('« meme-origine » désigne l’hôte qui sert la page', () => {
    expect(relayFrom(MEME_ORIGINE, 'https://klima-abc.herokuapp.com'))
      .toBe('https://klima-abc.herokuapp.com')
  })

  test('les espaces autour du réglage ne comptent pas', () => {
    expect(relayFrom('  meme-origine  ', 'https://x.example')).toBe('https://x.example')
  })
})
