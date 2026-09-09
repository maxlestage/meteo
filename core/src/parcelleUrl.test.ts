import { describe, expect, test } from 'bun:test'
import type { Parcelle } from './openMeteo'
import { parcelleFromParams, parcelleToParams, sameParcelle, urlForParcelle } from './parcelleUrl'

const reims: Parcelle = {
  name: 'Reims',
  latitude: 49.2628,
  longitude: 4.0347,
  admin: 'Grand Est',
  country: 'France',
}

describe('la parcelle dans l’adresse', () => {
  test('un aller-retour rend la même parcelle', () => {
    expect(parcelleFromParams(parcelleToParams(reims))).toEqual(reims)
  })

  test('une parcelle sans région ni pays reste lisible', () => {
    const nu: Parcelle = { name: 'Le Clos', latitude: 48.4468, longitude: 1.4892 }
    expect(parcelleFromParams(parcelleToParams(nu))).toEqual(nu)
  })

  test('l’adresse se lit à l’œil nu', () => {
    expect(parcelleToParams(reims).toString())
      .toBe('parcelle=Reims&lat=49.2628&lon=4.0347&admin=Grand+Est&pays=France')
  })

  test('une adresse sans parcelle ne renvoie rien', () => {
    expect(parcelleFromParams(new URLSearchParams(''))).toBeNull()
    expect(parcelleFromParams(new URLSearchParams('lat=49&lon=4'))).toBeNull()
  })

  test('des coordonnées manquantes ne valent pas zéro', () => {
    // Sans ce garde-fou, on irait chercher la météo du golfe de Guinée.
    expect(parcelleFromParams(new URLSearchParams('parcelle=Reims'))).toBeNull()
  })

  test('une adresse bricolée hors bornes est refusée', () => {
    for (const query of [
      'parcelle=X&lat=95&lon=4',
      'parcelle=X&lat=49&lon=200',
      'parcelle=X&lat=abc&lon=4',
    ]) {
      expect(parcelleFromParams(new URLSearchParams(query))).toBeNull()
    }
  })

  test('un nom vide ne fait pas une parcelle', () => {
    expect(parcelleFromParams(new URLSearchParams('parcelle=%20%20&lat=49&lon=4'))).toBeNull()
  })

  test('deux adresses de la même parcelle se reconnaissent', () => {
    expect(sameParcelle(reims, { ...reims, latitude: 49.26281 })).toBe(true)
    expect(sameParcelle(reims, { ...reims, latitude: 49.27 })).toBe(false)
    expect(sameParcelle(null, null)).toBe(true)
    expect(sameParcelle(reims, null)).toBe(false)
  })

  test('changer de parcelle garde le fragment de la page', () => {
    // Sur la vitrine, le fragment dit où l'on est : le perdre ferait sauter la
    // page en haut au moment de choisir une commune.
    expect(urlForParcelle('https://klima.test/meteo/#aujourdhui', reims))
      .toBe('/meteo/?parcelle=Reims&lat=49.2628&lon=4.0347&admin=Grand+Est&pays=France#aujourdhui')
  })

  test('changer de parcelle remplace l’ancienne au lieu de s’empiler', () => {
    const first = urlForParcelle('https://klima.test/app/', reims)
    const second = urlForParcelle(`https://klima.test${first}`, { name: 'Chartres', latitude: 48.4468, longitude: 1.4892 })
    expect(second).toBe('/app/?parcelle=Chartres&lat=48.4468&lon=1.4892')
    expect(second).not.toContain('Reims')
  })

  test('les autres paramètres de l’adresse survivent', () => {
    expect(urlForParcelle('https://klima.test/app/?debug=1', reims)).toContain('debug=1')
  })
})
