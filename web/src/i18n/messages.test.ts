import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { LANGUAGES, sharedMessages } from '@klima/core'
import { webMessages } from './messages'

describe('catalogue de l’application web', () => {
  test('les trois langues portent exactement les mêmes clés', () => {
    const reference = Object.keys(webMessages.fr).sort()
    for (const language of LANGUAGES) {
      expect(Object.keys(webMessages[language]).sort()).toEqual(reference)
    }
  })

  test('un motif à trous a les mêmes trous partout', () => {
    const tokens = (value: string) => (value.match(/\{(\w+)\}/g) ?? []).sort().join(',')
    for (const key of Object.keys(webMessages.fr)) {
      const reference = tokens(webMessages.fr[key]!)
      for (const language of LANGUAGES) {
        expect(tokens(webMessages[language][key]!), `${language} · ${key}`).toBe(reference)
      }
    }
  })
})

/**
 * Toute clé écrite en clair dans le code a une traduction.
 *
 * C'est la panne qu'un test de catalogue ne voit pas : les trois langues
 * peuvent porter exactement les mêmes clés et l'écran afficher quand même
 * « search.myField », parce que la clé demandée n'est dans aucune des deux
 * tables. C'est arrivé le jour où la vitrine s'est mise à nommer la position :
 * le libellé n'existait que dans le catalogue de l'application.
 *
 * Seules les clés littérales sont vérifiées — `t(condition.labelKey)` se
 * résout à l'exécution, et le catalogue partagé a ses propres tests pour ça.
 */
describe('les clés demandées existent', () => {

  const SOURCE = join(import.meta.dir, '..')

  function fichiers(dossier: string): string[] {
    return readdirSync(dossier, { withFileTypes: true }).flatMap((entree) => {
      const chemin = join(dossier, entree.name)
      if (entree.isDirectory()) return fichiers(chemin)
      return /\.tsx?$/.test(entree.name) && !entree.name.endsWith('.test.ts') ? [chemin] : []
    })
  }

  test('aucune clé littérale ne manque au catalogue', () => {
    const connues = new Set([
      ...Object.keys(sharedMessages.fr),
      ...Object.keys(webMessages.fr),
    ])

    const manquantes = new Set<string>()
    for (const fichier of fichiers(SOURCE)) {
      const source = readFileSync(fichier, 'utf8')
      for (const trouve of source.matchAll(/\bt\(\s*'([^']+)'/g)) {
        if (!connues.has(trouve[1]!)) manquantes.add(trouve[1]!)
      }
    }

    expect([...manquantes]).toEqual([])
  })
})
