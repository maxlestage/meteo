import { describe, expect, test } from 'bun:test'
import { CELL_DEGREES } from './grid'
import { locatesOnStart, parcelleFromPosition, type ParcelleOrigin } from './position'

describe('quand demander la position', () => {

  test('faute de mieux, oui', () => {
    expect(locatesOnStart('defaut')).toBe(true)
  })

  // Les deux cas où il y a un choix derrière la parcelle affichée. Aller
  // chercher la position par-dessus déferait ce choix.
  test('une adresse partagée désigne une parcelle : on n’y touche pas', () => {
    expect(locatesOnStart('adresse')).toBe(false)
  })

  test('une parcelle déjà consultée a été choisie : on n’y touche pas non plus', () => {
    expect(locatesOnStart('memoire')).toBe(false)
  })

  test('les trois origines sont traitées', () => {
    const origines: ParcelleOrigin[] = ['adresse', 'memoire', 'defaut']
    expect(origines.filter(locatesOnStart)).toEqual(['defaut'])
  })
})

describe('la parcelle d’une position', () => {

  test('porte le nom que lui donne l’interface', () => {
    const parcelle = parcelleFromPosition('Ma position', { latitude: 48.4468, longitude: 1.4892 })
    expect(parcelle.name).toBe('Ma position')
  })

  // Le point de la règle : une parcelle finit dans l'adresse de la page, et
  // une adresse se partage.
  test('est arrondie à la maille, pas au mètre près', () => {
    const parcelle = parcelleFromPosition('Ma position', {
      latitude: 48.44681234,
      longitude: 1.48923456,
    })
    expect(parcelle.latitude).toBe(48.44)
    expect(parcelle.longitude).toBe(1.48)
  })

  test('ne déplace jamais de plus d’une demi-maille', () => {
    const points = [
      { latitude: 48.4468, longitude: 1.4892 },
      { latitude: -33.8688, longitude: 151.2093 },
      { latitude: 0.0001, longitude: -0.0001 },
      { latitude: 64.1466, longitude: -21.9426 },
    ]
    for (const point of points) {
      const parcelle = parcelleFromPosition('x', point)
      expect(Math.abs(parcelle.latitude - point.latitude)).toBeLessThanOrEqual(CELL_DEGREES / 2)
      expect(Math.abs(parcelle.longitude - point.longitude)).toBeLessThanOrEqual(CELL_DEGREES / 2)
    }
  })

  test('deux positions du même carré donnent la même parcelle', () => {
    const a = parcelleFromPosition('x', { latitude: 48.4468, longitude: 1.4892 })
    const b = parcelleFromPosition('x', { latitude: 48.4490, longitude: 1.4870 })
    expect(a).toEqual(b)
  })
})
