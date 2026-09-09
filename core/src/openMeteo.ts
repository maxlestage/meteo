/**
 * Client de l'API agricole Open-Meteo.
 *
 * On n'interroge que les variables agronomiques : température et humidité du
 * sol, évapotranspiration de référence FAO-56, déficit de pression de vapeur,
 * en plus des paramètres nécessaires au calcul des fenêtres de traitement.
 * L'API est libre d'accès et ne demande aucune clé.
 */
import type { CurrentSample, DailySample, HourlySample } from './agro'
import type { Params } from './i18n'

const FORECAST_URL = 'https://api.open-meteo.com/v1/forecast'
const GEOCODING_URL = 'https://geocoding-api.open-meteo.com/v1/search'

const CURRENT_VARIABLES = [
  'temperature_2m',
  'apparent_temperature',
  'relative_humidity_2m',
  'weather_code',
  'is_day',
  'wind_speed_10m',
  'wind_gusts_10m',
] as const

const HOURLY_VARIABLES = [
  'temperature_2m',
  'weather_code',
  'is_day',
  'precipitation_probability',
  'relative_humidity_2m',
  'dew_point_2m',
  'precipitation',
  'wind_speed_10m',
  'wind_gusts_10m',
  'soil_temperature_6cm',
  'soil_moisture_3_to_9cm',
  'et0_fao_evapotranspiration',
  'vapour_pressure_deficit',
] as const

const DAILY_VARIABLES = [
  'weather_code',
  'sunrise',
  'sunset',
  'temperature_2m_min',
  'temperature_2m_max',
  'precipitation_sum',
  'precipitation_probability_max',
  'et0_fao_evapotranspiration',
  'wind_gusts_10m_max',
] as const

export interface Parcelle {
  name: string
  latitude: number
  longitude: number
  /** Région / département, pour lever l'ambiguïté entre homonymes. */
  admin?: string
  country?: string
}

export interface AgroForecast {
  parcelle: Parcelle
  /** Fuseau retenu par l'API pour cette parcelle. */
  timezone: string
  /** Altitude du point de grille (m). */
  elevation: number
  current: CurrentSample
  hourly: HourlySample[]
  daily: DailySample[]
  fetchedAt: Date
}

/**
 * Panne côté service. L'erreur porte une clé de catalogue, pas une phrase :
 * c'est l'interface qui la formule dans la langue de l'utilisateur.
 */
export class AgroApiError extends Error {
  constructor(
    readonly messageKey: string,
    readonly params?: Params,
    readonly cause?: unknown,
  ) {
    super(messageKey)
    this.name = 'AgroApiError'
  }
}

/** Récupère la prévision agricole d'une parcelle sur `days` jours. */
export async function fetchAgroForecast(
  parcelle: Parcelle,
  days = 7,
  signal?: AbortSignal,
): Promise<AgroForecast> {
  const url = new URL(FORECAST_URL)
  url.searchParams.set('latitude', parcelle.latitude.toFixed(4))
  url.searchParams.set('longitude', parcelle.longitude.toFixed(4))
  url.searchParams.set('current', CURRENT_VARIABLES.join(','))
  url.searchParams.set('hourly', HOURLY_VARIABLES.join(','))
  url.searchParams.set('daily', DAILY_VARIABLES.join(','))
  url.searchParams.set('wind_speed_unit', 'kmh')
  url.searchParams.set('timezone', 'auto')
  url.searchParams.set('forecast_days', String(days))

  const payload = await getJson<ForecastPayload>(url, signal)
  const current = decodeCurrent(payload)

  return {
    parcelle,
    timezone: payload.timezone,
    elevation: payload.elevation,
    current,
    // L'API renvoie la journée entière depuis minuit : on repart de l'heure en
    // cours, pour que « maintenant » soit bien le premier élément des séries.
    hourly: fromCurrentHour(decodeHourly(payload), current.time),
    daily: decodeDaily(payload),
    fetchedAt: new Date(),
  }
}

function fromCurrentHour(hours: HourlySample[], now: Date): HourlySample[] {
  const start = Math.floor(now.getTime() / 3_600_000) * 3_600_000
  const trimmed = hours.filter((hour) => hour.time.getTime() >= start)
  // Si l'heure courante sort de la série, on garde la série telle quelle.
  return trimmed.length > 0 ? trimmed : hours
}

/** Recherche une commune par son nom (géocodage Open-Meteo). */
export async function searchParcelles(query: string, signal?: AbortSignal): Promise<Parcelle[]> {
  const trimmed = query.trim()
  if (trimmed.length < 2) return []

  const url = new URL(GEOCODING_URL)
  url.searchParams.set('name', trimmed)
  url.searchParams.set('count', '8')
  url.searchParams.set('language', 'fr')
  url.searchParams.set('format', 'json')

  const payload = await getJson<GeocodingPayload>(url, signal)
  return (payload.results ?? []).map((r) => ({
    name: r.name,
    latitude: r.latitude,
    longitude: r.longitude,
    admin: r.admin1,
    country: r.country,
  }))
}

async function getJson<T>(url: URL, signal?: AbortSignal): Promise<T> {
  let response: Response
  try {
    response = await fetch(url, { signal })
  } catch (error) {
    if (error instanceof DOMException && error.name === 'AbortError') throw error
    throw new AgroApiError('api.unreachable', undefined, error)
  }

  if (!response.ok) {
    throw new AgroApiError('api.status', { status: response.status })
  }
  return (await response.json()) as T
}

/* -------------------- décodage des réponses -------------------- */

interface ForecastPayload {
  timezone: string
  /** Décalage du fuseau de la parcelle, en secondes. */
  utc_offset_seconds: number
  elevation: number
  current: Record<string, unknown> & { time: string }
  hourly: Record<string, unknown> & { time: string[] }
  daily: Record<string, unknown> & { time: string[] }
}

interface GeocodingPayload {
  results?: Array<{
    name: string
    latitude: number
    longitude: number
    admin1?: string
    country?: string
  }>
}

/**
 * Open-Meteo renvoie des tableaux parallèles indexés par `time`, avec des
 * `null` quand une variable manque sur le point de grille : on les ramène à 0
 * pour garder des séries de longueur homogène.
 */
function column(block: Record<string, unknown>, key: string, length: number): number[] {
  const raw = block[key]
  if (!Array.isArray(raw)) return new Array<number>(length).fill(0)
  return Array.from({ length }, (_, i) => {
    const value = raw[i]
    return typeof value === 'number' && Number.isFinite(value) ? value : 0
  })
}

/**
 * Open-Meteo renvoie des horodatages en heure locale de la parcelle, sans
 * décalage (« 2026-05-12T21:00 »). On les lit en UTC puis on retire le décalage
 * du fuseau : l'instant obtenu est absolu, et le formatage avec
 * `timeZone: forecast.timezone` réaffiche bien l'heure locale de la parcelle.
 */
function parseStamp(stamp: string, offsetSeconds: number): Date | null {
  const withTime = stamp.length === 10 ? `${stamp}T00:00` : stamp
  const parsed = Date.parse(`${withTime}Z`)
  return Number.isNaN(parsed) ? null : new Date(parsed - offsetSeconds * 1000)
}

/** Lit une valeur scalaire du bloc `current`, 0 si absente. */
function scalar(block: Record<string, unknown>, key: string): number {
  const value = block[key]
  return typeof value === 'number' && Number.isFinite(value) ? value : 0
}

function decodeCurrent(payload: ForecastPayload): CurrentSample {
  const c = payload.current
  return {
    time: parseStamp(c.time, payload.utc_offset_seconds) ?? new Date(),
    temperature: scalar(c, 'temperature_2m'),
    apparentTemperature: scalar(c, 'apparent_temperature'),
    weatherCode: scalar(c, 'weather_code'),
    isDay: scalar(c, 'is_day') === 1,
    relativeHumidity: scalar(c, 'relative_humidity_2m'),
    windSpeed: scalar(c, 'wind_speed_10m'),
    windGusts: scalar(c, 'wind_gusts_10m'),
  }
}

/** Colonne de dates : les valeurs illisibles deviennent `null`. */
function dateColumn(
  block: Record<string, unknown>,
  key: string,
  length: number,
  offsetSeconds: number,
): (Date | null)[] {
  const raw = block[key]
  if (!Array.isArray(raw)) return new Array<Date | null>(length).fill(null)
  return Array.from({ length }, (_, i) => {
    const value = raw[i]
    return typeof value === 'string' ? parseStamp(value, offsetSeconds) : null
  })
}

function decodeHourly(payload: ForecastPayload): HourlySample[] {
  const times = payload.hourly.time
  const c = (key: string) => column(payload.hourly, key, times.length)

  const temperature = c('temperature_2m')
  const weatherCode = c('weather_code')
  const isDay = c('is_day')
  const rainProbability = c('precipitation_probability')
  const humidity = c('relative_humidity_2m')
  const dewPoint = c('dew_point_2m')
  const precipitation = c('precipitation')
  const windSpeed = c('wind_speed_10m')
  const windGusts = c('wind_gusts_10m')
  const soilTemperature = c('soil_temperature_6cm')
  const soilMoisture = c('soil_moisture_3_to_9cm')
  const et0 = c('et0_fao_evapotranspiration')
  const vpd = c('vapour_pressure_deficit')

  return times.flatMap((stamp, i) => {
    const time = parseStamp(stamp, payload.utc_offset_seconds)
    if (!time) return []
    return [{
    time,
    weatherCode: weatherCode[i]!,
    isDay: isDay[i] !== 0,
    precipitationProbability: rainProbability[i]!,
    temperature: temperature[i]!,
    relativeHumidity: humidity[i]!,
    dewPoint: dewPoint[i]!,
    precipitation: precipitation[i]!,
    windSpeed: windSpeed[i]!,
    windGusts: windGusts[i]!,
    soilTemperature6cm: soilTemperature[i]!,
    soilMoisture3to9cm: soilMoisture[i]!,
    et0: et0[i]!,
    vapourPressureDeficit: vpd[i]!,
    }]
  })
}

function decodeDaily(payload: ForecastPayload): DailySample[] {
  const times = payload.daily.time
  const c = (key: string) => column(payload.daily, key, times.length)

  const weatherCode = c('weather_code')
  const sunrise = dateColumn(payload.daily, 'sunrise', times.length, payload.utc_offset_seconds)
  const sunset = dateColumn(payload.daily, 'sunset', times.length, payload.utc_offset_seconds)
  const tMin = c('temperature_2m_min')
  const tMax = c('temperature_2m_max')
  const rain = c('precipitation_sum')
  const rainProbability = c('precipitation_probability_max')
  const et0 = c('et0_fao_evapotranspiration')
  const gusts = c('wind_gusts_10m_max')

  return times.flatMap((stamp, i) => {
    const date = parseStamp(stamp, payload.utc_offset_seconds)
    if (!date) return []
    return [{
    date,
    weatherCode: weatherCode[i]!,
    temperatureMin: tMin[i]!,
    temperatureMax: tMax[i]!,
    precipitationSum: rain[i]!,
    precipitationProbabilityMax: rainProbability[i]!,
    et0Sum: et0[i]!,
    windGustsMax: gusts[i]!,
    sunrise: sunrise[i] ?? null,
    sunset: sunset[i] ?? null,
    }]
  })
}
