import { describe, expect, test } from 'bun:test'
import { sharedMessages } from './messages'
import { detectLanguage, interpolate, LANGUAGES, translator } from './i18n'

describe('catalogue partagé', () => {
  test('les trois langues portent exactement les mêmes clés', () => {
    const reference = Object.keys(sharedMessages.fr).sort()
    for (const language of LANGUAGES) {
      expect(Object.keys(sharedMessages[language]).sort()).toEqual(reference)
    }
  })

  test('aucun texte vide', () => {
    for (const language of LANGUAGES) {
      for (const [key, value] of Object.entries(sharedMessages[language])) {
        expect(value.trim().length, `${language} · ${key}`).toBeGreaterThan(0)
      }
    }
  })

  test('un motif à trous a les mêmes trous partout', () => {
    const tokens = (value: string) => (value.match(/\{(\w+)\}/g) ?? []).sort().join(',')
    for (const key of Object.keys(sharedMessages.fr)) {
      const reference = tokens(sharedMessages.fr[key]!)
      for (const language of LANGUAGES) {
        expect(tokens(sharedMessages[language][key]!), `${language} · ${key}`).toBe(reference)
      }
    }
  })
})

describe('choix de la langue', () => {
  test('reconnaît une variante régionale', () => {
    expect(detectLanguage(['fr-BE', 'nl'])).toBe('fr')
    expect(detectLanguage(['es-MX'])).toBe('es')
    expect(detectLanguage(['en-US'])).toBe('en')
  })

  test('retombe sur la langue de référence', () => {
    expect(detectLanguage(['de-DE', 'it'])).toBe('fr')
    expect(detectLanguage([])).toBe('fr')
  })

  test('prend la première langue reconnue, pas la première tout court', () => {
    expect(detectLanguage(['de', 'es-ES', 'en'])).toBe('es')
  })
})

describe('traduction', () => {
  test('interpole les valeurs', () => {
    expect(interpolate('Vent {wind} (max {limit})', { wind: '24 km/h', limit: '19 km/h' })).toBe(
      'Vent 24 km/h (max 19 km/h)',
    )
  })

  test('laisse un jeton inconnu en place plutôt que d’effacer', () => {
    expect(interpolate('Vent {wind}', {})).toBe('Vent {wind}')
  })

  test('une clé absente se voit', () => {
    expect(translator('en', sharedMessages)('nexiste.pas')).toBe('nexiste.pas')
  })

  test('une langue incomplète retombe sur la référence', () => {
    const partial = { fr: { greeting: 'Bonjour' }, en: {}, es: {} }
    expect(translator('en', partial)('greeting')).toBe('Bonjour')
  })
})
