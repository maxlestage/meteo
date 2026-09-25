/**
 * La page ne défile jamais de côté — sur les deux plateformes.
 *
 * Ce n'est pas une préférence d'implémentation mais une décision de produit :
 * on lit Klima en faisant glisser vers le bas, jamais vers la droite. Le
 * bandeau des vingt-quatre heures était un défilement horizontal sans
 * indicateur — on en voyait six et il fallait deviner que les autres
 * existaient. Il se replie désormais en grille, des deux côtés.
 *
 * Le test lit les sources plutôt que le rendu : vérifier le rendu demanderait
 * un navigateur et un simulateur dans l'intégration continue, alors que la
 * règle tient en une déclaration qu'on peut relire. Il vit à la racine parce
 * qu'il porte sur le web et sur iOS à la fois, comme la règle.
 */

import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const RACINE = import.meta.dir
const lire = (chemin: string) => readFileSync(join(RACINE, chemin), 'utf8')

describe('le web ne défile pas de côté', () => {

  const STYLES = lire('web/src/index.css')

  test('aucune règle ne rend un bloc défilable de côté', () => {
    const coupables = [...STYLES.matchAll(/overflow(-x)?\s*:\s*(auto|scroll)/g)].map((m) => m[0])
    expect(coupables).toEqual([])
  })

  // La grille est ce qui remplace le défilement : repassée en rangée unique,
  // les heures ressortiraient de l'écran sans que la règle ci-dessus s'en
  // aperçoive.
  test('le bandeau horaire se replie en grille', () => {
    const bloc = STYLES.match(/\.strip\s*\{([^}]*)\}/)
    expect(bloc).not.toBeNull()
    expect(bloc![1]).toContain('display: grid')
    expect(bloc![1]).toMatch(/grid-template-columns:\s*repeat\(auto-fi[tl]/)
  })
})

describe('iOS non plus', () => {

  const VUES = 'ios/Kliima/Views'
  const fichiers = readdirSync(join(RACINE, VUES)).filter((f) => f.endsWith('.swift'))

  test('il y a bien des vues à vérifier', () => {
    expect(fichiers.length).toBeGreaterThan(0)
  })

  test('aucune vue ne défile horizontalement', () => {
    const coupables = fichiers.filter((f) => lire(join(VUES, f)).includes('ScrollView(.horizontal'))
    expect(coupables).toEqual([])
  })

  test('le bandeau horaire y est une grille qui se replie', () => {
    const source = lire(join(VUES, 'HourlyStripView.swift'))
    expect(source).toContain('LazyVGrid')
    expect(source).toContain('GridItem(.adaptive(')
  })
})
