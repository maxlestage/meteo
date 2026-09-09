import type { Provider, ProviderQuery, SourceReading, WeatherSource } from './types'

/**
 * Open-Meteo redistribue les sorties brutes de plusieurs services nationaux.
 * Une seule requête suffit pour les quatre : le paramètre `models` suffixe
 * chaque colonne de l'identifiant du modèle.
 */
const ATTRIBUTION = 'Open-Meteo — modèles Météo-France, ECMWF, DWD et NOAA'

const MODELS: readonly WeatherSource[] = [
  {
    id: 'meteofrance_seamless',
    name: 'AROME / ARPEGE',
    institution: 'Météo-France',
    country: 'FR',
    provider: 'open-meteo',
    attribution: ATTRIBUTION,
  },
  {
    id: 'ecmwf_ifs025',
    name: 'IFS',
    institution: 'ECMWF',
    country: 'EU',
    provider: 'open-meteo',
    attribution: ATTRIBUTION,
  },
  {
    id: 'icon_seamless',
    name: 'ICON',
    institution: 'Deutscher Wetterdienst',
    country: 'DE',
    provider: 'open-meteo',
    attribution: ATTRIBUTION,
  },
  {
    id: 'gfs_seamless',
    name: 'GFS',
    institution: 'NOAA',
    country: 'US',
    provider: 'open-meteo',
    attribution: ATTRIBUTION,
  },
]

export const openMeteoProvider: Provider = {
  id: 'open-meteo',
  institution: 'Open-Meteo',
  platforms: ['web', 'native'],
  attribution: ATTRIBUTION,
  fetch: fetchOpenMeteo,
}

export const OPEN_METEO_SOURCES = MODELS

async function fetchOpenMeteo(query: ProviderQuery, signal?: AbortSignal): Promise<SourceReading[]> {
  const url = new URL('https://api.open-meteo.com/v1/forecast')
  url.searchParams.set('latitude', query.latitude.toFixed(4))
  url.searchParams.set('longitude', query.longitude.toFixed(4))
  url.searchParams.set('hourly', 'temperature_2m,precipitation,wind_speed_10m')
  url.searchParams.set('models', MODELS.map((model) => model.id).join(','))
  url.searchParams.set('wind_speed_unit', 'kmh')
  url.searchParams.set('timezone', 'auto')
  url.searchParams.set('forecast_days', '1')

  const response = await fetch(url, { signal })
  if (!response.ok) throw new Error(`Open-Meteo ${response.status}`)

  const payload = (await response.json()) as {
    utc_offset_seconds?: number
    hourly?: Record<string, unknown> & { time?: string[] }
  }

  const times = payload.hourly?.time ?? []
  const index = currentHourIndex(times, payload.utc_offset_seconds ?? 0)
  if (index < 0) return []

  const readings: SourceReading[] = []
  for (const source of MODELS) {
    const temperature = column(payload.hourly, `temperature_2m_${source.id}`, index)
    // Une source sans température ne couvre pas la parcelle : on l'écarte
    // plutôt que de la compter pour zéro.
    if (temperature === null) continue

    readings.push({
      source,
      temperature,
      precipitation: column(payload.hourly, `precipitation_${source.id}`, index) ?? 0,
      windSpeed: column(payload.hourly, `wind_speed_10m_${source.id}`, index) ?? 0,
    })
  }
  return readings
}

/** Première heure de la série postérieure ou égale à l'heure en cours. */
function currentHourIndex(times: readonly string[], offsetSeconds: number): number {
  const start = Math.floor(Date.now() / 3_600_000) * 3_600_000
  for (let i = 0; i < times.length; i += 1) {
    const parsed = Date.parse(`${times[i]}Z`)
    if (!Number.isNaN(parsed) && parsed - offsetSeconds * 1000 >= start) return i
  }
  return times.length > 0 ? times.length - 1 : -1
}

function column(block: unknown, key: string, index: number): number | null {
  if (typeof block !== 'object' || block === null) return null
  const raw = (block as Record<string, unknown>)[key]
  if (!Array.isArray(raw)) return null
  const value = raw[index]
  return typeof value === 'number' && Number.isFinite(value) ? value : null
}
