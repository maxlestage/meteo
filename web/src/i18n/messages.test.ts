import { describe, expect, test } from 'bun:test'
import { LANGUAGES } from '@klima/core'
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
