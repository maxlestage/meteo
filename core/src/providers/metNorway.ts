import { USER_AGENT, type Provider, type ProviderQuery, type SourceReading, type WeatherSource } from './types'

/**
 * Institut météorologique norvégien — `Locationforecast 2.0`.
 *
 * **Fournisseur natif uniquement.** Ses conditions d'utilisation imposent un
 * en-tête `User-Agent` identifiant l'application et un moyen de contact ; un
 * navigateur interdit de le fixer. On l'appelle donc depuis iOS et watchOS, où
 * `URLSession` le permet, et pas depuis le web — plutôt que d'envoyer des
 * requêtes anonymes contre leur volonté.
 */
const SOURCE: WeatherSource = {
  id: 'met-no-locationforecast',
  name: 'Locationforecast',
  institution: 'MET Norway',
  country: 'NO',
  provider: 'met-norway',
  attribution: 'Données de MET Norway (Norwegian Meteorological Institute), licence NLOD / CC BY 4.0',
}

export const metNorwayProvider: Provider = {
  id: 'met-norway',
  institution: 'MET Norway',
  platforms: ['native'],
  attribution: SOURCE.attribution,
  fetch: fetchMetNorway,
}

export const MET_NORWAY_SOURCE = SOURCE

interface MetPayload {
  properties?: {
    timeseries?: Array<{
      time?: string
      data?: {
        instant?: { details?: Record<string, number> }
        next_1_hours?: { details?: Record<string, number> }
      }
    }>
  }
}

async function fetchMetNorway(query: ProviderQuery, signal?: AbortSignal): Promise<SourceReading[]> {
  const url = new URL('https://api.met.no/weatherapi/locationforecast/2.0/compact')
  url.searchParams.set('lat', query.latitude.toFixed(4))
  url.searchParams.set('lon', query.longitude.toFixed(4))

  const response = await fetch(url, { signal, headers: { 'User-Agent': USER_AGENT } })
  if (!response.ok) throw new Error(`MET Norway ${response.status}`)

  const payload = (await response.json()) as MetPayload
  const entry = nearestEntry(payload)
  if (!entry) return []

  const instant = entry.data?.instant?.details ?? {}
  const temperature = instant.air_temperature
  if (typeof temperature !== 'number' || !Number.isFinite(temperature)) return []

  return [
    {
      source: SOURCE,
      temperature,
      precipitation: entry.data?.next_1_hours?.details?.precipitation_amount ?? 0,
      // MET Norway donne le vent en m/s ; Klima raisonne en km/h.
      windSpeed: typeof instant.wind_speed === 'number' ? instant.wind_speed * 3.6 : 0,
    },
  ]
}

/** Échéance la plus proche de maintenant : la série est horaire puis trihoraire. */
function nearestEntry(payload: MetPayload) {
  const series = payload.properties?.timeseries ?? []
  const now = Date.now()
  let best: (typeof series)[number] | undefined
  let bestDistance = Number.POSITIVE_INFINITY

  for (const entry of series) {
    const time = entry.time ? Date.parse(entry.time) : Number.NaN
    if (Number.isNaN(time)) continue
    const distance = Math.abs(time - now)
    if (distance < bestDistance) {
      bestDistance = distance
      best = entry
    }
  }
  return best
}
