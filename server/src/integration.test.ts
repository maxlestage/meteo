/**
 * La chaîne entière, sans rien simuler entre les maillons : le client du cœur
 * partagé appelle le relais par HTTP réel, le relais interroge un fournisseur
 * simulé. Ce que ça vérifie et que les tests unitaires ne peuvent pas voir :
 * la construction des URL, les en-têtes, et le fait que le web gagne bien
 * MET Norway une fois le relais en place.
 */
import { afterAll, beforeAll, describe, expect, test } from 'bun:test'
import { fetchAllReadings, useDirectProviders, useRelay } from '@klima/core'
import { createHandler } from './index'

const upstreamSeen: URL[] = []
const upstreamAgents: Array<string | null> = []

/** Le fournisseur, vu du relais. */
const fakeUpstream = (async (input: URL | RequestInfo, init?: RequestInit) => {
  const url = new URL(String(input))
  upstreamSeen.push(url)
  upstreamAgents.push(new Headers(init?.headers).get('User-Agent'))

  if (url.host.includes('met.no')) {
    return Response.json({
      properties: {
        timeseries: [
          {
            time: new Date().toISOString(),
            data: {
              instant: { details: { air_temperature: 17.1, wind_speed: 4 } },
              next_1_hours: { details: { precipitation_amount: 0 } },
            },
          },
        ],
      },
    })
  }
  if (url.host.includes('brightsky')) {
    return Response.json({ weather: { temperature: 17.4, precipitation: 0, wind_speed: 13 } })
  }
  const time = [new Date().toISOString().slice(0, 13) + ':00']
  const hourly: Record<string, unknown> = { time }
  for (const model of ['meteofrance_seamless', 'ecmwf_ifs025', 'icon_seamless', 'gfs_seamless']) {
    hourly[`temperature_2m_${model}`] = [16.8]
    hourly[`precipitation_${model}`] = [0]
    hourly[`wind_speed_10m_${model}`] = [12]
  }
  return Response.json({ utc_offset_seconds: -new Date().getTimezoneOffset() * 60, hourly })
}) as unknown as typeof fetch

let server: ReturnType<typeof Bun.serve>

beforeAll(() => {
  server = Bun.serve({
    port: 0,
    fetch: createHandler({ fetch: fakeUpstream, openMeteoKey: 'clé-de-test' }),
  })
  useRelay(`http://localhost:${server.port}`)
})

afterAll(() => {
  useDirectProviders()
  server.stop(true)
})

describe('le cœur partagé à travers un vrai relais', () => {
  test('le web obtient les mêmes sources que le natif, MET Norway comprise', async () => {
    const outcomes = await fetchAllReadings({ latitude: 48.4468, longitude: 1.4892 }, 'web')

    expect(outcomes.map((o) => o.provider.id).sort()).toEqual([
      'bright-sky',
      'met-norway',
      'open-meteo',
    ])
    for (const outcome of outcomes) {
      expect(outcome.error).toBeUndefined()
      expect(outcome.readings.length).toBeGreaterThan(0)
    }

    // Six sources lues : quatre modèles Open-Meteo, MET Norway, Bright Sky.
    expect(outcomes.flatMap((o) => o.readings)).toHaveLength(6)
  })

  test('c’est le relais qui s’est nommé auprès de MET Norway, pas le client', () => {
    const index = upstreamSeen.findIndex((url) => url.host.includes('met.no'))
    expect(index).toBeGreaterThanOrEqual(0)
    expect(upstreamAgents[index]).toContain('Klima/')
  })

  test('le fournisseur a reçu la maille, et la clé n’a jamais quitté le relais', async () => {
    const openMeteo = upstreamSeen.find((url) => url.host.includes('open-meteo'))!
    expect(openMeteo.searchParams.get('latitude')).toBe('48.440')
    expect(openMeteo.searchParams.get('apikey')).toBe('clé-de-test')

    const direct = await fetch(
      `http://localhost:${server.port}/v1/open-meteo/forecast?latitude=48.4468&longitude=1.4892`,
    )
    expect(await direct.text()).not.toContain('clé-de-test')
  })

  test('une deuxième lecture de la même parcelle ne repart pas chez le fournisseur', async () => {
    const before = upstreamSeen.length
    await fetchAllReadings({ latitude: 48.4468, longitude: 1.4892 }, 'web')
    expect(upstreamSeen.length).toBe(before)
  })
})
