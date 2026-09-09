import { endpoints } from '../endpoints'
import type { Provider, ProviderQuery, SourceReading, WeatherSource } from './types'

/**
 * Bright Sky — relais ouvert des données du Deutscher Wetterdienst.
 *
 * Contrairement aux trois autres sources, celle-ci n'est pas une sortie de
 * modèle mais une **observation de station**, ce qui en fait un point de
 * comparaison d'une autre nature : elle dit ce qu'il fait, pas ce qui est
 * prévu. La couverture suit le réseau du DWD, dense en Allemagne et clairsemée
 * ailleurs : hors de portée d'une station, le fournisseur ne renvoie rien et
 * sort du recoupement.
 */
const SOURCE: WeatherSource = {
  id: 'brightsky-dwd',
  name: 'Observation DWD',
  institution: 'Deutscher Wetterdienst',
  country: 'DE',
  provider: 'bright-sky',
  attribution: 'Bright Sky — données du Deutscher Wetterdienst, licence CC BY 4.0',
}

export const brightSkyProvider: Provider = {
  id: 'bright-sky',
  institution: 'Bright Sky / DWD',
  platforms: ['web', 'native'],
  viaRelay: true,
  attribution: SOURCE.attribution,
  fetch: fetchBrightSky,
}

export const BRIGHT_SKY_SOURCE = SOURCE

interface BrightSkyPayload {
  weather?: {
    temperature?: number | null
    precipitation?: number | null
    wind_speed?: number | null
  }
}

async function fetchBrightSky(query: ProviderQuery, signal?: AbortSignal): Promise<SourceReading[]> {
  const url = new URL(endpoints().brightSky)
  url.searchParams.set('lat', query.latitude.toFixed(4))
  url.searchParams.set('lon', query.longitude.toFixed(4))

  const response = await fetch(url, { signal })
  // 404 : aucune station à portée. Ce n'est pas une panne, c'est une absence.
  if (response.status === 404) return []
  if (!response.ok) throw new Error(`Bright Sky ${response.status}`)

  const payload = (await response.json()) as BrightSkyPayload
  const temperature = payload.weather?.temperature
  if (typeof temperature !== 'number' || !Number.isFinite(temperature)) return []

  return [
    {
      source: SOURCE,
      temperature,
      precipitation: numberOrZero(payload.weather?.precipitation),
      windSpeed: numberOrZero(payload.weather?.wind_speed),
    },
  ]
}

function numberOrZero(value: number | null | undefined): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : 0
}
