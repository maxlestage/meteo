import { describe, expect, test } from 'bun:test'
import type { AgroSummary, HourlySample } from './agro'
import {
  DEFAULT_ALERT_OPTIONS,
  deferPastQuietHours,
  EMPTY_ALERT_STATE,
  evaluateAlerts,
  isQuiet,
  recordSent,
  recordSoil,
  type AlertOptions,
  type AlertState,
} from './alerts'

const NOW = new Date('2026-04-15T09:00:00')
const options: AlertOptions = { ...DEFAULT_ALERT_OPTIONS, now: NOW }

const hourAt = (offset: number) => new Date(NOW.getTime() + offset * 3_600_000)

function hour(offset: number, over: Partial<HourlySample> = {}): HourlySample {
  return {
    time: hourAt(offset),
    temperature: 14,
    weatherCode: 3,
    isDay: true,
    precipitationProbability: 10,
    relativeHumidity: 60,
    dewPoint: 7,
    precipitation: 0,
    windSpeed: 9,
    windGusts: 15,
    soilTemperature6cm: 12,
    soilMoisture3to9cm: 0.22,
    et0: 0.1,
    vapourPressureDeficit: 0.6,
    ...over,
  }
}

function summary(over: Partial<AgroSummary> = {}): AgroSummary {
  return {
    water: { precipitation: 10, evapotranspiration: 8, balance: 2, status: 'equilibre', irrigationAdvice: 0 },
    soil: { moisture: 0.22, temperature: 12, state: 'ressuye', trafficable: true, sowable: true },
    disease: { level: 'faible', leafWetnessHours: 2 },
    frost: { severity: 'aucun', minTemperature: 6, hoarFrost: false },
    gdd: 120,
    nextSpray: null,
    ...over,
  }
}

const spray = (startOffset: number, endOffset: number, score = 90) => ({
  start: hourAt(startOffset),
  end: hourAt(endOffset),
  score,
})

describe('plage de silence', () => {
  test('elle enjambe minuit', () => {
    expect(isQuiet(22, 21, 6)).toBe(true)
    expect(isQuiet(3, 21, 6)).toBe(true)
    expect(isQuiet(6, 21, 6)).toBe(false)
    expect(isQuiet(14, 21, 6)).toBe(false)
  })

  test('hors silence, l’envoi part tout de suite', () => {
    const send = new Date('2026-04-15T14:00:00')
    expect(deferPastQuietHours(send, new Date('2026-04-15T17:00:00'), DEFAULT_ALERT_OPTIONS))
      .toEqual(send)
  })

  test('la nuit, l’envoi attend la reprise du matin', () => {
    const deferred = deferPastQuietHours(
      new Date('2026-04-15T23:30:00'),
      new Date('2026-04-16T10:00:00'),
      DEFAULT_ALERT_OPTIONS,
    )
    expect(deferred).toEqual(new Date('2026-04-16T06:00:00'))
  })

  test('avant l’aube, la reprise est le matin même, pas le lendemain', () => {
    const deferred = deferPastQuietHours(
      new Date('2026-04-16T03:00:00'),
      new Date('2026-04-16T10:00:00'),
      DEFAULT_ALERT_OPTIONS,
    )
    expect(deferred).toEqual(new Date('2026-04-16T06:00:00'))
  })

  test('un événement passé avant la reprise est abandonné, pas retardé', () => {
    // Mieux vaut se taire que raconter la veille.
    expect(deferPastQuietHours(
      new Date('2026-04-15T23:30:00'),
      new Date('2026-04-16T04:00:00'),
      DEFAULT_ALERT_OPTIONS,
    )).toBeNull()
  })
})

describe('fenêtre de traitement', () => {
  test('annoncée quand il reste le temps de sortir le pulvérisateur', () => {
    const alerts = evaluateAlerts(
      summary({ nextSpray: spray(4, 7) }), [], EMPTY_ALERT_STATE, options,
    )
    expect(alerts.map((a) => a.kind)).toEqual(['fenetre'])
    expect(alerts[0]!.params).toEqual({ score: 90 })
    expect(alerts[0]!.at).toEqual(hourAt(4))
  })

  test('dans une heure, on se tait : on ne sort pas le pulvérisateur en une heure', () => {
    const alerts = evaluateAlerts(
      summary({ nextSpray: spray(1, 4) }), [], EMPTY_ALERT_STATE, options,
    )
    expect(alerts).toHaveLength(0)
  })

  test('dans trois jours, il est trop tôt pour en parler', () => {
    const alerts = evaluateAlerts(
      summary({ nextSpray: spray(72, 75) }), [], EMPTY_ALERT_STATE, options,
    )
    expect(alerts).toHaveLength(0)
  })

  test('le domaine renvoie des clés, jamais des phrases', () => {
    const alerts = evaluateAlerts(
      summary({ nextSpray: spray(4, 7) }), [], EMPTY_ALERT_STATE, options,
    )
    expect(alerts[0]!.titleKey).toBe('alert.fenetre.title')
    expect(alerts[0]!.bodyKey).toBe('alert.fenetre.body')
    expect(alerts[0]!.titleKey).not.toContain(' ')
  })
})

describe('gel', () => {
  test('annoncé avec la température attendue', () => {
    const alerts = evaluateAlerts(
      summary({ frost: { severity: 'modere', minTemperature: -2.4, hoarFrost: true } }),
      [hour(0, { temperature: 3 }), hour(10, { temperature: -1 })], EMPTY_ALERT_STATE, options,
    )
    expect(alerts.map((a) => a.kind)).toEqual(['gel'])
    expect(alerts[0]!.at).toEqual(hourAt(10))
    expect(alerts[0]!.params).toEqual({ temperature: -2.4 })
  })

  test('pas de gel, pas d’alerte', () => {
    expect(evaluateAlerts(summary(), [], EMPTY_ALERT_STATE, options)).toHaveLength(0)
  })
})

describe('sol devenu portant', () => {
  test('c’est le passage qui compte, pas l’état', () => {
    const state: AlertState = { ...EMPTY_ALERT_STATE, lastSoilState: 'sature' }
    const alerts = evaluateAlerts(summary(), [], state, options)
    expect(alerts.map((a) => a.kind)).toEqual(['sol'])
  })

  test('sans état précédent, on se tait', () => {
    // Au premier lancement on ne sait pas d'où l'on vient : annoncer « le sol
    // est ressuyé » à quelqu'un dont le sol l'est depuis un mois est du bruit.
    expect(evaluateAlerts(summary(), [], EMPTY_ALERT_STATE, options)).toHaveLength(0)
  })

  test('un sol déjà ressuyé la veille ne redéclenche rien', () => {
    const state: AlertState = { ...EMPTY_ALERT_STATE, lastSoilState: 'ressuye' }
    expect(evaluateAlerts(summary(), [], state, options)).toHaveLength(0)
  })
})

describe('pluie lavante', () => {
  test('annoncée quand elle tombe dans les six heures après la fenêtre', () => {
    const alerts = evaluateAlerts(
      summary({ nextSpray: spray(4, 7) }),
      [hour(8, { precipitation: 2.4 })], EMPTY_ALERT_STATE, options,
    )
    expect(alerts.map((a) => a.kind)).toEqual(['fenetre', 'pluie'])
    expect(alerts[1]!.params).toEqual({ rain: 2.4 })
  })

  test('une bruine sous le seuil ne compte pas', () => {
    const alerts = evaluateAlerts(
      summary({ nextSpray: spray(4, 7) }),
      [hour(8, { precipitation: 0.05 })], EMPTY_ALERT_STATE, options,
    )
    expect(alerts.map((a) => a.kind)).toEqual(['fenetre'])
  })

  test('une pluie bien après la fenêtre ne lave rien', () => {
    const alerts = evaluateAlerts(
      summary({ nextSpray: spray(4, 7) }),
      [hour(20, { precipitation: 4 })], EMPTY_ALERT_STATE, options,
    )
    expect(alerts.map((a) => a.kind)).toEqual(['fenetre'])
  })
})

describe('délai de garde', () => {
  test('une alerte déjà partie ne repart pas dans la foulée', () => {
    const state = recordSent(EMPTY_ALERT_STATE, [
      { kind: 'fenetre', at: NOW, titleKey: '', bodyKey: '', params: {} },
    ], new Date(NOW.getTime() - 3_600_000))

    expect(evaluateAlerts(summary({ nextSpray: spray(4, 7) }), [], state, options))
      .toHaveLength(0)
  })

  test('passé le délai, elle repart', () => {
    const state = recordSent(EMPTY_ALERT_STATE, [
      { kind: 'fenetre', at: NOW, titleKey: '', bodyKey: '', params: {} },
    ], new Date(NOW.getTime() - 7 * 3_600_000))

    expect(evaluateAlerts(summary({ nextSpray: spray(4, 7) }), [], state, options))
      .toHaveLength(1)
  })

  test('le délai de garde d’une nature n’en bâillonne pas une autre', () => {
    const state = recordSent(EMPTY_ALERT_STATE, [
      { kind: 'fenetre', at: NOW, titleKey: '', bodyKey: '', params: {} },
    ], NOW)

    const alerts = evaluateAlerts(
      summary({
        nextSpray: spray(4, 7),
        frost: { severity: 'faible', minTemperature: -0.5, hoarFrost: false },
      }), [], state, options,
    )
    expect(alerts.map((a) => a.kind)).toEqual(['gel'])
  })

  test('l’état retenu est sérialisable et n’altère pas l’ancien', () => {
    const before = recordSoil(EMPTY_ALERT_STATE, 'sature')
    const after = recordSent(before, [
      { kind: 'gel', at: NOW, titleKey: '', bodyKey: '', params: {} },
    ], NOW)

    expect(before.lastSent.gel).toBeUndefined()
    expect(after.lastSoilState).toBe('sature')
    expect(JSON.parse(JSON.stringify(after)).lastSent.gel).toBe(NOW.toISOString())
  })
})

describe('le calme est le cas normal', () => {
  test('une journée sans rien à dire ne dit rien', () => {
    expect(evaluateAlerts(summary(), [hour(0), hour(1), hour(2)], EMPTY_ALERT_STATE, options))
      .toEqual([])
  })
})
