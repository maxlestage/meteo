import { afterEach, describe, expect, test } from 'bun:test'
import { brightSkyProvider } from './brightSky'
import { metNorwayProvider } from './metNorway'
import { openMeteoProvider } from './openMeteo'
import { useDirectProviders, useRelay } from '../endpoints'
import { fetchAllReadings, providersFor, attributionsFor, PROVIDERS } from './index'

const query = { latitude: 48.4468, longitude: 1.4892 }
const realFetch = globalThis.fetch

afterEach(() => {
  globalThis.fetch = realFetch
})

/** Remplace `fetch` le temps d'un test, en routant par domaine. */
function stub(routes: Record<string, () => Response>) {
  globalThis.fetch = ((input: string | URL | Request) => {
    const url = String(input)
    for (const [host, handler] of Object.entries(routes)) {
      if (url.includes(host)) return Promise.resolve(handler())
    }
    return Promise.reject(new Error(`hôte non simulé : ${url}`))
  }) as unknown as typeof fetch
}

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  })
}

/** Trois heures autour de maintenant, au format d'Open-Meteo. */
function hours(): string[] {
  const pad = (n: number) => String(n).padStart(2, '0')
  return [-1, 0, 1].map((offset) => {
    const d = new Date(Date.now() + offset * 3_600_000)
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:00`
  })
}

function openMeteoPayload(temperatures: Record<string, (number | null)[]>) {
  const time = hours()
  const hourly: Record<string, unknown> = { time }
  for (const [id, values] of Object.entries(temperatures)) {
    hourly[`temperature_2m_${id}`] = values
    hourly[`precipitation_${id}`] = values.map(() => 0)
    hourly[`wind_speed_10m_${id}`] = values.map(() => 12)
  }
  return { utc_offset_seconds: -new Date().getTimezoneOffset() * 60, hourly }
}

describe('Open-Meteo', () => {
  test('lit les colonnes suffixées par modèle, à l’heure en cours', async () => {
    stub({
      'api.open-meteo.com': () =>
        json(
          openMeteoPayload({
            meteofrance_seamless: [10, 18.2, 19],
            ecmwf_ifs025: [10, 18.6, 19],
            icon_seamless: [10, 19, 19],
            gfs_seamless: [10, 18.4, 19],
          }),
        ),
    })

    const readings = await openMeteoProvider.fetch(query)
    expect(readings).toHaveLength(4)
    expect(readings.map((r) => r.temperature).sort()).toEqual([18.2, 18.4, 18.6, 19])
    expect(readings.every((r) => r.source.provider === 'open-meteo')).toBe(true)
  })

  test('écarte un modèle sans valeur plutôt que de compter zéro', async () => {
    stub({
      'api.open-meteo.com': () =>
        json(
          openMeteoPayload({
            meteofrance_seamless: [10, 18, 19],
            ecmwf_ifs025: [null, null, null],
          }),
        ),
    })

    const readings = await openMeteoProvider.fetch(query)
    expect(readings).toHaveLength(1)
    expect(readings[0]!.source.institution).toBe('Météo-France')
  })
})

describe('MET Norway', () => {
  test('prend l’échéance la plus proche et convertit le vent en km/h', async () => {
    const now = new Date().toISOString().slice(0, 13) + ':00:00Z'
    stub({
      'api.met.no': () =>
        json({
          properties: {
            timeseries: [
              {
                time: new Date(Date.now() + 6 * 3_600_000).toISOString(),
                data: { instant: { details: { air_temperature: 30, wind_speed: 1 } } },
              },
              {
                time: now,
                data: {
                  instant: { details: { air_temperature: 18.3, wind_speed: 5 } },
                  next_1_hours: { details: { precipitation_amount: 0.4 } },
                },
              },
            ],
          },
        }),
    })

    const readings = await metNorwayProvider.fetch(query)
    expect(readings).toHaveLength(1)
    expect(readings[0]!.temperature).toBe(18.3)
    expect(readings[0]!.precipitation).toBe(0.4)
    // 5 m/s = 18 km/h
    expect(readings[0]!.windSpeed).toBeCloseTo(18, 5)
  })

  test('n’est pas appelé directement depuis le web : ses conditions imposent un User-Agent', () => {
    expect(metNorwayProvider.platforms).toEqual(['native'])
    expect(providersFor('web').map((p) => p.id)).not.toContain('met-norway')
    expect(providersFor('native').map((p) => p.id)).toContain('met-norway')
  })

  test('le relais le rend accessible au web : c’est lui qui se nomme', () => {
    useRelay('https://relais.klima')
    try {
      expect(providersFor('web').map((p) => p.id)).toContain('met-norway')
      // Et le client ne pose plus l'en-tête lui-même : le navigateur le
      // refuserait, et le relais l'a déjà mis.
      expect(providersFor('web')).toEqual(providersFor('native'))
    } finally {
      useDirectProviders()
    }
  })

  test('en direct le client se nomme, par le relais il s’en abstient', async () => {
    const seen: Array<string | null> = []
    globalThis.fetch = ((_input: string | URL | Request, init?: RequestInit) => {
      seen.push(new Headers(init?.headers).get('User-Agent'))
      return Promise.resolve(json({ properties: { timeseries: [] } }))
    }) as unknown as typeof fetch

    await metNorwayProvider.fetch(query)
    useRelay('https://relais.klima')
    try {
      await metNorwayProvider.fetch(query)
    } finally {
      useDirectProviders()
    }

    expect(seen[0]).toContain('Klima/')
    // Un navigateur refuserait de poser cet en-tête, et le relais l'a déjà mis.
    expect(seen[1]).toBeNull()
  })
})

describe('Bright Sky', () => {
  test('lit l’observation de la station', async () => {
    stub({
      'api.brightsky.dev': () =>
        json({ weather: { temperature: 17.8, precipitation: 0.2, wind_speed: 14 } }),
    })

    const readings = await brightSkyProvider.fetch(query)
    expect(readings).toHaveLength(1)
    expect(readings[0]!.temperature).toBe(17.8)
    expect(readings[0]!.source.name).toBe('Observation DWD')
  })

  test('aucune station à portée : absence, pas panne', async () => {
    stub({ 'api.brightsky.dev': () => json({}, 404) })
    expect(await brightSkyProvider.fetch(query)).toEqual([])
  })

  test('station sans température : rien à comparer', async () => {
    stub({ 'api.brightsky.dev': () => json({ weather: { temperature: null } }) })
    expect(await brightSkyProvider.fetch(query)).toEqual([])
  })
})

describe('interrogation de tous les fournisseurs', () => {
  test('une panne n’en écarte qu’un', async () => {
    stub({
      'api.open-meteo.com': () =>
        json(openMeteoPayload({ meteofrance_seamless: [10, 18, 19], ecmwf_ifs025: [10, 18.5, 19] })),
      'api.brightsky.dev': () => json({}, 500),
    })

    const outcomes = await fetchAllReadings(query, 'web')
    expect(outcomes).toHaveLength(2)

    const openMeteo = outcomes.find((o) => o.provider.id === 'open-meteo')!
    expect(openMeteo.readings).toHaveLength(2)
    expect(openMeteo.error).toBeUndefined()

    const brightSky = outcomes.find((o) => o.provider.id === 'bright-sky')!
    expect(brightSky.readings).toHaveLength(0)
    expect(brightSky.error).toBeDefined()
  })

  test('le natif interroge un fournisseur de plus que le web', () => {
    expect(providersFor('native').length).toBe(providersFor('web').length + 1)
    expect(PROVIDERS).toHaveLength(3)
  })

  test('les mentions ne sont listées qu’une fois par licence', async () => {
    stub({
      'api.open-meteo.com': () =>
        json(openMeteoPayload({ meteofrance_seamless: [10, 18, 19], ecmwf_ifs025: [10, 18.5, 19] })),
      'api.brightsky.dev': () =>
        json({ weather: { temperature: 17.8, precipitation: 0, wind_speed: 14 } }),
    })

    const outcomes = await fetchAllReadings(query, 'web')
    const attributions = attributionsFor(outcomes.flatMap((o) => o.readings))
    expect(attributions).toHaveLength(2)
    expect(attributions.some((a) => a.includes('Open-Meteo'))).toBe(true)
    expect(attributions.some((a) => a.includes('Bright Sky'))).toBe(true)
  })
})
