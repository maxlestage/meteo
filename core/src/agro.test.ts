import { describe, expect, test } from 'bun:test'
import {
  cumulativeGdd,
  diseasePressure,
  evaluateSprayHour,
  frostRisk,
  growingDegreeDays,
  nextSprayOpportunity,
  soilCondition,
  sprayWindows,
  summarize,
  waterBalance,
  type DailySample,
  type HourlySample,
} from './agro'

const hour = (overrides: Partial<HourlySample> = {}, index = 0): HourlySample => ({
  time: new Date(Date.UTC(2026, 4, 12, 6 + index)),
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

const day = (overrides: Partial<DailySample> = {}): DailySample => ({
  date: new Date(Date.UTC(2026, 4, 12)),
  weatherCode: 3,
  temperatureMin: 10,
  temperatureMax: 22,
  precipitationSum: 2,
  precipitationProbabilityMax: 30,
  et0Sum: 3.5,
  windGustsMax: 25,
  sunrise: new Date(Date.UTC(2026, 4, 12, 4, 20)),
  sunset: new Date(Date.UTC(2026, 4, 12, 19, 40)),
  ...overrides,
})

describe('degrés-jours', () => {
  test('moyenne au-dessus de la base', () => {
    expect(growingDegreeDays(10, 22)).toBe(6) // (10+22)/2 = 16 → 16 - 10
  })

  test('journée trop froide ne capitalise rien', () => {
    expect(growingDegreeDays(2, 8)).toBe(0)
  })

  test('plafonnement de la moyenne à 30 °C', () => {
    expect(growingDegreeDays(30, 42)).toBe(20)
  })

  test('cumul sur plusieurs jours', () => {
    expect(cumulativeGdd([day(), day({ temperatureMin: 14, temperatureMax: 26 })])).toBe(16)
  })
})

describe('bilan hydrique', () => {
  test('déficit déclenche un conseil d’irrigation', () => {
    const days = Array.from({ length: 7 }, () => day({ precipitationSum: 0, et0Sum: 4 }))
    const result = waterBalance(days)
    expect(result.evapotranspiration).toBe(28)
    expect(result.balance).toBe(-28)
    expect(result.status).toBe('deficit')
    expect(result.irrigationAdvice).toBe(28)
  })

  test('pluies abondantes : excédent, pas d’irrigation', () => {
    const result = waterBalance([day({ precipitationSum: 30, et0Sum: 3 })])
    expect(result.status).toBe('excedent')
    expect(result.irrigationAdvice).toBe(0)
  })

  test('série équilibrée', () => {
    expect(waterBalance([day({ precipitationSum: 4, et0Sum: 3.5 })]).status).toBe('equilibre')
  })
})

describe('fenêtres de pulvérisation', () => {
  test('conditions idéales', () => {
    const w = evaluateSprayHour([hour()], 0)
    expect(w.score).toBe(100)
    expect(w.verdict).toBe('favorable')
    expect(w.blockers).toHaveLength(0)
  })

  test('vent réglementaire dépassé', () => {
    const w = evaluateSprayHour([hour({ windSpeed: 24 })], 0)
    expect(w.verdict).toBe('defavorable')
    expect(w.blockers[0]).toContain('Vent 24 km/h')
  })

  test('pluie attendue à l’heure suivante', () => {
    const w = evaluateSprayHour([hour(), hour({ precipitation: 1.4 }, 1)], 0)
    expect(w.verdict).toBe('defavorable')
    expect(w.blockers.join(' ')).toContain('Pluie')
  })

  test('vent nul : inversion thermique signalée', () => {
    const w = evaluateSprayHour([hour({ windSpeed: 1 })], 0)
    expect(w.blockers.join(' ')).toContain('inversion thermique')
    expect(w.verdict).toBe('acceptable')
  })

  test('score borné à zéro', () => {
    const w = evaluateSprayHour([hour({ windSpeed: 40, windGusts: 60, precipitation: 5, temperature: 32 })], 0)
    expect(w.score).toBe(0)
  })

  test('la dernière heure de la série est évaluable', () => {
    expect(() => sprayWindows([hour(), hour({}, 1)])).not.toThrow()
  })

  test('index hors série', () => {
    expect(() => evaluateSprayHour([hour()], 5)).toThrow(RangeError)
  })
})

describe('prochaine opportunité', () => {
  test('retient la première plage de 2 h consécutives', () => {
    const hours = [
      hour({ windSpeed: 30 }),
      hour({}, 1),
      hour({}, 2),
      hour({}, 3),
    ]
    const opportunity = nextSprayOpportunity(sprayWindows(hours))
    expect(opportunity).not.toBeNull()
    expect(opportunity!.start.getUTCHours()).toBe(7)
    expect(opportunity!.end.getUTCHours()).toBe(9)
    expect(opportunity!.score).toBe(100)
  })

  test('aucune fenêtre quand tout est défavorable', () => {
    const hours = [hour({ windSpeed: 45 }), hour({ windSpeed: 45 }, 1)]
    expect(nextSprayOpportunity(sprayWindows(hours))).toBeNull()
  })
})

describe('gel', () => {
  test('nuit douce', () => {
    expect(frostRisk(6, 3).severity).toBe('aucun')
  })

  test('gelée blanche', () => {
    const risk = frostRisk(-0.5, -2)
    expect(risk.severity).toBe('faible')
    expect(risk.hoarFrost).toBe(true)
  })

  test('gel sévère', () => {
    expect(frostRisk(-6, -8).severity).toBe('severe')
  })
})

describe('pression maladie', () => {
  test('feuillage humide et doux : pression élevée', () => {
    const hours = Array.from({ length: 14 }, (_, i) => hour({ relativeHumidity: 95, temperature: 16 }, i))
    const result = diseasePressure(hours)
    expect(result.leafWetnessHours).toBe(14)
    expect(result.level).toBe('elevee')
  })

  test('humidité forte mais trop froid : pas d’humectation comptée', () => {
    const hours = Array.from({ length: 14 }, (_, i) => hour({ relativeHumidity: 97, temperature: 3 }, i))
    expect(diseasePressure(hours).level).toBe('faible')
  })
})

describe('sol', () => {
  test('sol saturé : pas de portance', () => {
    const soil = soilCondition([hour({ soilMoisture3to9cm: 0.41 })])
    expect(soil.state).toBe('sature')
    expect(soil.trafficable).toBe(false)
    expect(soil.sowable).toBe(false)
  })

  test('sol ressuyé et chaud : semis possible', () => {
    const soil = soilCondition([hour({ soilMoisture3to9cm: 0.22, soilTemperature6cm: 12 })])
    expect(soil.state).toBe('ressuye')
    expect(soil.sowable).toBe(true)
  })

  test('sol froid : semis déconseillé', () => {
    expect(soilCondition([hour({ soilTemperature6cm: 5 })]).sowable).toBe(false)
  })

  test('série vide', () => {
    expect(soilCondition([]).trafficable).toBe(false)
  })
})

describe('synthèse', () => {
  test('assemble tous les indicateurs', () => {
    const hours = Array.from({ length: 24 }, (_, i) => hour({}, i))
    const days = Array.from({ length: 7 }, () => day())
    const summary = summarize(hours, days)
    expect(summary.gdd).toBe(42)
    expect(summary.soil.state).toBe('ressuye')
    expect(summary.nextSpray).not.toBeNull()
    expect(summary.frost.severity).toBe('aucun')
  })
})
