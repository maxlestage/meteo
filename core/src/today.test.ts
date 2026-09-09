import { describe, expect, test } from 'bun:test'
import { dayDigest } from './today'
import type { AgroForecast } from './openMeteo'
import type { DailySample, HourlySample } from './agro'

/** 12 mai 2026, 18 h 00 heure de Paris. */
const startOfDayUtc = Date.UTC(2026, 4, 11, 22, 0) // minuit à Paris

const hour = (offsetHours: number, overrides: Partial<HourlySample> = {}): HourlySample => ({
  time: new Date(startOfDayUtc + offsetHours * 3_600_000),
  weatherCode: 3,
  isDay: true,
  precipitationProbability: 20,
  temperature: 18,
  relativeHumidity: 65,
  dewPoint: 11,
  precipitation: 0,
  windSpeed: 8,
  windGusts: 14,
  soilTemperature6cm: 15,
  soilMoisture3to9cm: 0.24,
  et0: 0.2,
  vapourPressureDeficit: 0.7,
  ...overrides,
})

const day = (dayOffset: number, overrides: Partial<DailySample> = {}): DailySample => ({
  date: new Date(startOfDayUtc + dayOffset * 86_400_000),
  weatherCode: 61,
  temperatureMin: 9,
  temperatureMax: 21,
  precipitationSum: 2.4,
  precipitationProbabilityMax: 60,
  et0Sum: 3.6,
  windGustsMax: 32,
  sunrise: new Date(startOfDayUtc + 6.5 * 3_600_000),
  sunset: new Date(startOfDayUtc + 21.4 * 3_600_000),
  ...overrides,
})

/** Prévision partant de 18 h : il reste 6 h aujourd'hui, puis demain. */
const forecast = (overrides: Partial<AgroForecast> = {}): AgroForecast => ({
  parcelle: { name: 'Chartres', latitude: 48.44, longitude: 1.48 },
  timezone: 'Europe/Paris',
  elevation: 155,
  current: {
    time: new Date(startOfDayUtc + 18 * 3_600_000),
    temperature: 19,
    apparentTemperature: 19,
    weatherCode: 61,
    isDay: true,
    relativeHumidity: 70,
    windSpeed: 10,
    windGusts: 20,
  },
  hourly: Array.from({ length: 30 }, (_, i) => hour(18 + i)),
  daily: [day(0), day(1)],
  fetchedAt: new Date(startOfDayUtc + 18 * 3_600_000),
  ...overrides,
})

describe('synthèse du jour', () => {
  test('ne retient que les heures restantes de la journée', () => {
    const digest = dayDigest(forecast())
    expect(digest).not.toBeNull()
    // De 18 h à 23 h inclus : six heures avant minuit.
    expect(digest!.remainingHours).toHaveLength(6)
  })

  test('reprend les cumuls du jour et calcule son bilan', () => {
    const digest = dayDigest(forecast())!
    expect(digest.precipitationSum).toBe(2.4)
    expect(digest.et0Sum).toBe(3.6)
    expect(digest.balance).toBe(-1.2)
    expect(digest.weatherCode).toBe(61)
  })

  test('propose une fenêtre de traitement d’ici ce soir', () => {
    const digest = dayDigest(forecast())!
    expect(digest.spray).not.toBeNull()
    expect(digest.spray!.score).toBe(100)
  })

  test('aucune fenêtre si le vent souffle jusqu’à la nuit', () => {
    const windy = forecast({
      hourly: Array.from({ length: 30 }, (_, i) => hour(18 + i, { windSpeed: 38 })),
    })
    expect(dayDigest(windy)!.spray).toBeNull()
  })

  test('le gel se juge sur la nuit, qui déborde sur le lendemain', () => {
    const freezing = forecast({
      hourly: Array.from({ length: 30 }, (_, i) =>
        hour(18 + i, { temperature: i >= 8 ? -3 : 10, dewPoint: -5 }),
      ),
    })
    const digest = dayDigest(freezing)!
    expect(digest.frost.severity).toBe('modere')
    expect(digest.frost.hoarFrost).toBe(true)
  })

  test('série journalière vide', () => {
    expect(dayDigest(forecast({ daily: [] }))).toBeNull()
  })
})
