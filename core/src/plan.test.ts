import { describe, expect, test } from 'bun:test'
import {
  allows,
  canAddParcelle,
  FEATURES,
  limitsFor,
  partitionParcelles,
  PLANS,
  upgradeReasonKey,
} from './plan'

describe('paliers', () => {
  test('le palier libre donne la journée entière, sans rien amputer', () => {
    // La règle du modèle : on facture l'échelle et l'anticipation, jamais la
    // réponse du jour. Sept jours de prévision des deux côtés.
    expect(limitsFor('libre').jours).toBe(limitsFor('pro').jours)
    expect(limitsFor('libre').parcelles).toBe(1)
  })

  test('le palier libre n’ouvre aucune fonction payante', () => {
    for (const feature of FEATURES) {
      expect(allows('libre', feature)).toBe(false)
    }
  })

  test('le palier pro les ouvre toutes', () => {
    for (const feature of FEATURES) {
      expect(allows('pro', feature)).toBe(true)
    }
  })

  test('la deuxième parcelle est la limite du palier libre', () => {
    expect(canAddParcelle('libre', 0)).toBe(true)
    expect(canAddParcelle('libre', 1)).toBe(false)
  })

  test('le palier pro n’a pas de plafond de parcelles', () => {
    expect(canAddParcelle('pro', 0)).toBe(true)
    expect(canAddParcelle('pro', 500)).toBe(true)
  })

  test('une résiliation verrouille les parcelles, elle n’en perd aucune', () => {
    const parcelles = ['Chartres', 'Reims', 'Toulouse']
    const acces = partitionParcelles('libre', parcelles)

    expect(acces.readable).toEqual(['Chartres'])
    expect(acces.locked).toEqual(['Reims', 'Toulouse'])
    // Rien n'a disparu : le réabonnement les rend telles quelles.
    expect([...acces.readable, ...acces.locked]).toEqual(parcelles)
  })

  test('au retour de l’abonnement, tout est de nouveau lisible', () => {
    const parcelles = ['Chartres', 'Reims', 'Toulouse']
    expect(partitionParcelles('pro', parcelles)).toEqual({ readable: parcelles, locked: [] })
  })

  test('la partition ne modifie pas la liste qu’on lui donne', () => {
    const parcelles = ['Chartres', 'Reims']
    partitionParcelles('libre', parcelles)
    expect(parcelles).toEqual(['Chartres', 'Reims'])
  })

  test('le motif de blocage est une clé, pas une phrase', () => {
    for (const feature of FEATURES) {
      const key = upgradeReasonKey(feature)
      expect(key).toBe(`plan.reason.${feature}`)
      expect(key).not.toContain(' ')
    }
  })

  test('chaque palier connu a ses limites', () => {
    for (const plan of PLANS) {
      expect(limitsFor(plan).jours).toBeGreaterThan(0)
      expect(limitsFor(plan).parcelles).toBeGreaterThan(0)
    }
  })
})
