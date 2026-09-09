import { describe, expect, test } from 'bun:test'
import { consensus, consensusFromOutcomes } from './consensus'
import { OPEN_METEO_SOURCES, weatherSource, type SourceReading } from './providers'

const reading = (index: number, temperature: number, precipitation = 0, windSpeed = 10): SourceReading => ({
  source: OPEN_METEO_SOURCES[index]!,
  temperature,
  precipitation,
  windSpeed,
})

describe('recoupement des modèles', () => {
  test('un seul modèle ne fait pas un consensus', () => {
    expect(consensus([reading(0, 18)])).toBeNull()
    expect(consensus([])).toBeNull()
  })

  test('la valeur retenue est la médiane, pas la moyenne', () => {
    // 18, 18,5, 19 et un modèle isolé à 25 : la médiane ignore l'écart.
    const result = consensus([
      reading(0, 18),
      reading(1, 18.5),
      reading(2, 19),
      reading(3, 25),
    ])!
    // (18,5 + 19) / 2 = 18,75, arrondi à la décimale comme le reste du domaine.
    expect(result.temperature.median).toBe(18.8)
    expect(result.temperature.min).toBe(18)
    expect(result.temperature.max).toBe(25)
    expect(result.temperature.spread).toBe(7)
  })

  test('médiane d’un nombre impair de modèles', () => {
    const result = consensus([reading(0, 12), reading(1, 18), reading(2, 15)])!
    expect(result.temperature.median).toBe(15)
  })

  test('modèles serrés : accord fort', () => {
    const result = consensus([reading(0, 18), reading(1, 18.4), reading(2, 19)])!
    expect(result.temperature.spread).toBe(1)
    expect(result.agreeOnRain).toBe(true)
    expect(result.agreement).toBe('forte')
  })

  test('deux degrés d’écart : accord moyen', () => {
    const result = consensus([reading(0, 17), reading(1, 19)])!
    expect(result.agreement).toBe('moyenne')
  })

  test('plus de trois degrés d’écart : accord faible', () => {
    const result = consensus([reading(0, 15), reading(1, 19.5)])!
    expect(result.agreement).toBe('faible')
  })

  test('désaccord sur la pluie : l’accord ne peut pas être fort', () => {
    const result = consensus([reading(0, 18, 0), reading(1, 18.2, 1.4)])!
    expect(result.temperature.spread).toBeLessThan(1.5)
    expect(result.agreeOnRain).toBe(false)
    expect(result.agreement).toBe('moyenne')
  })

  test('une trace de pluie sous le seuil ne compte pas comme un désaccord', () => {
    const result = consensus([reading(0, 18, 0), reading(1, 18.2, 0.05)])!
    expect(result.agreeOnRain).toBe(true)
    expect(result.agreement).toBe('forte')
  })

  test('tous d’accord sur la pluie', () => {
    const result = consensus([reading(0, 14, 2.1), reading(1, 14.5, 1.8), reading(2, 14.2, 3)])!
    expect(result.agreeOnRain).toBe(true)
    expect(result.agreement).toBe('forte')
  })

  test('le vent est recoupé comme le reste', () => {
    const result = consensus([reading(0, 18, 0, 12), reading(1, 18, 0, 24)])!
    expect(result.windSpeed.median).toBe(18)
    expect(result.windSpeed.spread).toBe(12)
  })
})

describe('table des sources', () => {
  test('quatre modèles chez Open-Meteo, tous d’instituts différents', () => {
    expect(OPEN_METEO_SOURCES).toHaveLength(4)
    expect(new Set(OPEN_METEO_SOURCES.map((s) => s.institution)).size).toBe(4)
    expect(new Set(OPEN_METEO_SOURCES.map((s) => s.id)).size).toBe(4)
  })

  test('recherche par identifiant', () => {
    expect(weatherSource('meteofrance_seamless')?.institution).toBe('Météo-France')
    expect(weatherSource('inconnu')).toBeUndefined()
  })
})

describe('comptage des fournisseurs', () => {
  test('retient combien de fournisseurs ont répondu', () => {
    const result = consensusFromOutcomes([
      { provider: { id: 'a' } as never, readings: [reading(0, 18), reading(1, 18.4)] },
      { provider: { id: 'b' } as never, readings: [] },
      { provider: { id: 'c' } as never, readings: [reading(2, 18.6)] },
    ])!
    expect(result.readings).toHaveLength(3)
    expect(result.providersAnswered).toBe(2)
    expect(result.providersQueried).toBe(3)
  })

  test('aucun fournisseur n’a répondu', () => {
    expect(
      consensusFromOutcomes([{ provider: { id: 'a' } as never, readings: [], error: new Error('x') }]),
    ).toBeNull()
  })
})
