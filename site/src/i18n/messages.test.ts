import { describe, expect, test } from 'bun:test'
import { LANGUAGES } from '@klima/core'
import { siteMessages } from './messages'

describe('catalogue du site de présentation', () => {
  test('les trois langues portent exactement les mêmes clés', () => {
    const reference = Object.keys(siteMessages.fr).sort()
    for (const language of LANGUAGES) {
      expect(Object.keys(siteMessages[language]).sort()).toEqual(reference)
    }
  })

  test('un motif à trous a les mêmes trous partout', () => {
    const tokens = (value: string) => (value.match(/\{(\w+)\}/g) ?? []).sort().join(',')
    for (const key of Object.keys(siteMessages.fr)) {
      const reference = tokens(siteMessages.fr[key]!)
      for (const language of LANGUAGES) {
        expect(tokens(siteMessages[language][key]!), `${language} · ${key}`).toBe(reference)
      }
    }
  })
})
