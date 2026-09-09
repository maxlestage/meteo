import { describe, expect, test } from 'bun:test'
import { CELL_DEGREES, cellFor, cellKey, snap } from './grid'

describe('maille de cache', () => {
  test('deux points de la même cellule donnent la même clé', () => {
    // Chartres et un point à quelques centaines de mètres, du même côté de
    // la frontière de maille : une seule interrogation pour les deux.
    const a = cellFor(48.4468, 1.4892)
    const b = cellFor(48.449, 1.487)
    expect(cellKey('forecast', a)).toBe(cellKey('forecast', b))
  })

  test('deux points proches peuvent tomber de part et d’autre d’une frontière', () => {
    // Comportement assumé : découper en mailles crée des frontières, et deux
    // voisins peuvent les enjamber. Le coût est une interrogation de plus,
    // jamais une réponse fausse — chaque cellule reste à moins d’une
    // demi-maille du point demandé.
    const a = cellFor(48.4468, 1.4892)
    const b = cellFor(48.4512, 1.4892)
    expect(cellKey('forecast', a)).not.toBe(cellKey('forecast', b))
  })

  test('deux points séparés de plus d’une maille se distinguent', () => {
    const a = cellFor(48.4468, 1.4892)
    const b = cellFor(48.4468 + CELL_DEGREES * 1.5, 1.4892)
    expect(cellKey('forecast', a)).not.toBe(cellKey('forecast', b))
  })

  test('l’arrondi ne déplace jamais un point de plus d’une demi-maille', () => {
    for (const value of [48.4468, -1.9993, 0, 43.3, 89.99, -0.0099]) {
      expect(Math.abs(snap(value) - value)).toBeLessThanOrEqual(CELL_DEGREES / 2 + 1e-9)
    }
  })

  test('la clé ne dépend pas de la façon dont le nombre s’écrit', () => {
    // Sans arrondi décimal, 0,02 × 2422 s’écrirait 48.440000000000005.
    expect(cellKey('forecast', cellFor(48.44, 1.48))).toBe('forecast:48.44,1.48')
  })

  test('l’usage sépare deux caches sur la même cellule', () => {
    const cell = cellFor(48.4468, 1.4892)
    expect(cellKey('forecast', cell)).not.toBe(cellKey('consensus', cell))
  })
})
