import { describe, expect, test } from 'bun:test'
import type { DailySample } from './agro'
import { accumulate, isComplete } from './cumuls'

function day(iso: string, over: Partial<DailySample> = {}): DailySample {
  return {
    date: new Date(iso),
    weatherCode: 3,
    sunrise: new Date(`${iso}T06:00:00`),
    sunset: new Date(`${iso}T21:00:00`),
    temperatureMin: 8,
    temperatureMax: 20,
    precipitationSum: 2,
    precipitationProbabilityMax: 40,
    et0Sum: 3,
    windGustsMax: 30,
    ...over,
  }
}

const series = [
  day('2026-04-10T00:00:00'),
  day('2026-04-11T00:00:00', { precipitationSum: 5, et0Sum: 2 }),
  day('2026-04-12T00:00:00', { precipitationSum: 0, et0Sum: 4 }),
]

describe('cumuls depuis une date', () => {
  test('additionne pluie, ET0 et bilan sur la période', () => {
    const cumul = accumulate(series, new Date('2026-04-10T00:00:00'))!

    expect(cumul.days).toBe(3)
    expect(cumul.precipitation).toBe(7)
    expect(cumul.evapotranspiration).toBe(9)
    expect(cumul.balance).toBe(-2)
  })

  test('capitalise les degrés-jours au-dessus de la base', () => {
    // (8 + 20) / 2 = 14 ; base 10 → 4 degrés-jours par journée, trois journées.
    expect(accumulate(series, new Date('2026-04-10T00:00:00'))!.gdd).toBe(12)
  })

  test('une date en cours de série ne compte que ce qui suit', () => {
    const cumul = accumulate(series, new Date('2026-04-11T00:00:00'))!
    expect(cumul.days).toBe(2)
    expect(cumul.precipitation).toBe(5)
  })

  test('l’heure de la date demandée n’exclut pas sa journée', () => {
    // Un semis noté à 14 h reste un semis du 11.
    const cumul = accumulate(series, new Date('2026-04-11T14:30:00'))!
    expect(cumul.days).toBe(2)
    expect(cumul.from).toEqual(new Date('2026-04-11T00:00:00'))
  })

  test('une demande antérieure à la série se dit incomplète', () => {
    // Un cumul depuis un semis d'octobre calculé sur une série qui commence en
    // avril serait faux. On le sert, mais on dit ce qui manque.
    const cumul = accumulate(series, new Date('2026-04-05T00:00:00'))!

    expect(cumul.days).toBe(3)
    expect(cumul.missingDays).toBe(5)
    expect(isComplete(cumul)).toBe(false)
    expect(cumul.requestedFrom).toEqual(new Date('2026-04-05T00:00:00'))
    expect(cumul.from).toEqual(new Date('2026-04-10T00:00:00'))
  })

  test('une couverture complète le dit aussi', () => {
    expect(isComplete(accumulate(series, new Date('2026-04-10T00:00:00'))!)).toBe(true)
  })

  test('une date postérieure à la série ne renvoie rien plutôt qu’un zéro', () => {
    // Zéro millimètre et « pas de données » ne veulent pas dire la même chose.
    expect(accumulate(series, new Date('2026-05-01T00:00:00'))).toBeNull()
  })

  test('une série vide ne renvoie rien', () => {
    expect(accumulate([], new Date('2026-04-10T00:00:00'))).toBeNull()
  })

  test('les bornes effectives encadrent ce qui a été compté', () => {
    const cumul = accumulate(series, new Date('2026-04-10T00:00:00'))!
    expect(cumul.from).toEqual(new Date('2026-04-10T00:00:00'))
    expect(cumul.to).toEqual(new Date('2026-04-12T00:00:00'))
  })

  test('une base de degrés-jours différente change le résultat', () => {
    // Base 6 : (8 + 20) / 2 = 14 → 8 par journée.
    expect(accumulate(series, new Date('2026-04-10T00:00:00'), 6)!.gdd).toBe(24)
  })
})
