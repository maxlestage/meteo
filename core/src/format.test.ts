import { describe, expect, test } from 'bun:test'
import { decimal, percent, signedWithUnit, withUnit } from './format'

const NBSP = ' '

describe('nombres à la française', () => {
  test('virgule décimale', () => {
    expect(decimal(4.8)).toBe('4,8')
    expect(decimal(2)).toBe('2,0')
    expect(decimal(21.53, 2)).toBe('21,53')
  })

  test('espace insécable avant l’unité', () => {
    expect(withUnit(2.1, 'mm')).toBe(`2,1${NBSP}mm`)
    expect(percent(26.6)).toBe(`27${NBSP}%`)
  })

  test('le bilan porte son signe', () => {
    expect(signedWithUnit(2.7, 'mm')).toBe(`+2,7${NBSP}mm`)
    expect(signedWithUnit(-1.2, 'mm')).toBe(`-1,2${NBSP}mm`)
    // Un bilan nul n'est pas « positif » : pas de signe.
    expect(signedWithUnit(0, 'mm')).toBe(`0,0${NBSP}mm`)
  })
})
