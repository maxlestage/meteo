import { describe, expect, test } from 'bun:test'
import { accorde, accordDepuis, etiquette } from './pro'

describe('ce que le déploiement accorde', () => {
  test('rien de configuré : la boutique décide seule', () => {
    for (const valeur of [undefined, '', '   ', '0', 'non', 'false']) {
      const accord = accordDepuis(valeur)
      expect(accord.type).toBe('aucun')
      expect(accorde(accord, null)).toBe(false)
      expect(accorde(accord, 'n’importe quoi')).toBe(false)
    }
  })

  test('« tous » accorde à qui demande', () => {
    for (const valeur of ['tous', 'TOUS', '1', 'oui', 'true', 'all']) {
      const accord = accordDepuis(valeur)
      expect(accord.type).toBe('tous')
      expect(accorde(accord, null)).toBe(true)
    }
  })

  test('un code n’accorde qu’au code', () => {
    const accord = accordDepuis('sillon-2026-dUx7')

    expect(accorde(accord, 'sillon-2026-dUx7')).toBe(true)
    expect(accorde(accord, 'sillon-2026-dUx8')).toBe(false)
    expect(accorde(accord, 'sillon')).toBe(false)
    expect(accorde(accord, '')).toBe(false)
    // Demander sans rien présenter n'ouvre pas davantage.
    expect(accorde(accord, null)).toBe(false)
  })

  test('l’étiquette ne dit jamais le code', () => {
    const accord = accordDepuis('sillon-2026-dUx7')
    expect(etiquette(accord)).toBe('sur code')
    expect(etiquette(accord)).not.toContain('sillon')
  })
})
