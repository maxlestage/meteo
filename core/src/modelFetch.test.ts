import { afterEach, describe, expect, test } from 'bun:test'
import { fetchModelConsensus } from './openMeteo'
import type { Parcelle } from './openMeteo'

const parcelle: Parcelle = { name: 'Chartres', latitude: 48.4468, longitude: 1.4892 }
const realFetch = globalThis.fetch

afterEach(() => {
  globalThis.fetch = realFetch
})

/** Deux heures autour de maintenant, pour que la série couvre l'heure en cours. */
function times(): string[] {
  const pad = (n: number) => String(n).padStart(2, '0')
  const now = new Date()
  return [-1, 0, 1].map((offset) => {
    const d = new Date(now.getTime() + offset * 3_600_000)
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:00`
  })
}

/** Remplace `fetch` le temps d'un test. */
function stubFetch(handler: (input: string) => Response) {
  globalThis.fetch = ((input: string | URL | Request) =>
    Promise.resolve(handler(String(input)))) as unknown as typeof fetch
}

function respond(hourly: Record<string, unknown>) {
  stubFetch(
    () =>
      new Response(
        JSON.stringify({ utc_offset_seconds: -new Date().getTimezoneOffset() * 60, hourly }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      ),
  )
}

describe('recoupement en ligne', () => {
  test('lit les colonnes suffixées par modèle, à l’heure en cours', async () => {
    const time = times()
    respond({
      time,
      temperature_2m_meteofrance_seamless: [10, 18.2, 19],
      precipitation_meteofrance_seamless: [0, 0, 0],
      wind_speed_10m_meteofrance_seamless: [8, 12, 14],
      temperature_2m_ecmwf_ifs025: [10, 18.6, 19],
      precipitation_ecmwf_ifs025: [0, 0, 0],
      wind_speed_10m_ecmwf_ifs025: [8, 13, 14],
      temperature_2m_icon_seamless: [10, 19, 19],
      precipitation_icon_seamless: [0, 0, 0],
      wind_speed_10m_icon_seamless: [8, 11, 14],
      temperature_2m_gfs_seamless: [10, 18.4, 19],
      precipitation_gfs_seamless: [0, 0, 0],
      wind_speed_10m_gfs_seamless: [8, 12, 14],
    })

    const result = await fetchModelConsensus(parcelle)
    expect(result).not.toBeNull()
    expect(result!.readings).toHaveLength(4)
    expect(result!.temperature.min).toBe(18.2)
    expect(result!.temperature.max).toBe(19)
    expect(result!.agreement).toBe('forte')
    expect(result!.readings.map((r) => r.model.institution)).toContain('Météo-France')
  })

  /// Un modèle qui ne couvre pas la parcelle renvoie des `null` : il sort du
  /// recoupement au lieu d'y peser comme un zéro.
  test('écarte un modèle sans valeur plutôt que de compter zéro', async () => {
    const time = times()
    respond({
      time,
      temperature_2m_meteofrance_seamless: [10, 18, 19],
      precipitation_meteofrance_seamless: [0, 0, 0],
      wind_speed_10m_meteofrance_seamless: [8, 12, 14],
      temperature_2m_ecmwf_ifs025: [null, null, null],
      temperature_2m_icon_seamless: [10, 18.5, 19],
      precipitation_icon_seamless: [0, 0, 0],
      wind_speed_10m_icon_seamless: [8, 11, 14],
    })

    const result = await fetchModelConsensus(parcelle)
    expect(result!.readings).toHaveLength(2)
    expect(result!.temperature.min).toBe(18)
    expect(result!.temperature.max).toBe(18.5)
  })

  test('un seul modèle disponible : pas de consensus annoncé', async () => {
    const time = times()
    respond({
      time,
      temperature_2m_meteofrance_seamless: [10, 18, 19],
      precipitation_meteofrance_seamless: [0, 0, 0],
      wind_speed_10m_meteofrance_seamless: [8, 12, 14],
    })

    expect(await fetchModelConsensus(parcelle)).toBeNull()
  })

  test('la requête demande bien les quatre modèles', async () => {
    // Un tableau plutôt qu'une variable : l'affectation dans la fermeture
    // n'est pas suivie par l'analyse de types.
    const requested: string[] = []
    stubFetch((input) => {
      requested.push(input)
      return new Response(JSON.stringify({ utc_offset_seconds: 0, hourly: { time: [] } }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    })

    await fetchModelConsensus(parcelle)
    const url = requested.join(' ')
    expect(url).toContain('models=meteofrance_seamless')
    expect(url).toContain('ecmwf_ifs025')
    expect(url).toContain('icon_seamless')
    expect(url).toContain('gfs_seamless')
  })
})
