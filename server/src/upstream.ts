/**
 * Les interrogations qui partent vraiment.
 *
 * Le relais ne décode rien : il transmet la réponse du fournisseur telle
 * quelle. Les clients gardent donc leur code de décodage, et le jour où un
 * fournisseur ajoute un champ, il n'y a rien à changer ici.
 *
 * Deux choses ne peuvent se faire que de ce côté :
 *
 * - **La clé Open-Meteo.** Le plan gratuit d'Open-Meteo est réservé à un usage
 *   non commercial ; dès que Klima se vend, il faut un plan payant, donc une
 *   clé. Une clé glissée dans une application est une clé publiée : elle vit
 *   ici, et nulle part ailleurs.
 * - **L'en-tête d'identification de MET Norway.** Leurs conditions l'exigent,
 *   et un navigateur n'a pas le droit de le poser. Passer par le relais rend
 *   donc MET Norway accessible aussi à l'application web, ce qu'un appel
 *   direct depuis le navigateur ne permettrait pas.
 */
import { USER_AGENT } from '@klima/core'

export interface UpstreamConfig {
  /** Clé du plan commercial Open-Meteo. Absente en développement. */
  openMeteoKey?: string
  /** Injectable pour les tests : par défaut, `fetch` global. */
  fetch?: typeof fetch
}

/** Hôte à interroger : le client payant quand on a une clé, le public sinon. */
function openMeteoHost(config: UpstreamConfig): string {
  return config.openMeteoKey ? 'https://customer-api.open-meteo.com' : 'https://api.open-meteo.com'
}

export class UpstreamError extends Error {
  constructor(readonly status: number, message: string) {
    super(message)
    this.name = 'UpstreamError'
  }
}

async function getJson(url: URL, config: UpstreamConfig, headers: Record<string, string> = {}): Promise<unknown> {
  const call = config.fetch ?? fetch
  const response = await call(url, { headers: { Accept: 'application/json', ...headers } })
  if (!response.ok) {
    throw new UpstreamError(response.status, `${url.host} a répondu ${response.status}`)
  }
  return response.json()
}

/** Recopie les paramètres de la demande, en remplaçant le point par sa maille. */
function withCell(url: URL, params: URLSearchParams, latitude: number, longitude: number): URL {
  for (const [key, value] of params) {
    if (key !== 'latitude' && key !== 'longitude') url.searchParams.set(key, value)
  }
  url.searchParams.set('latitude', latitude.toFixed(3))
  url.searchParams.set('longitude', longitude.toFixed(3))
  return url
}

export function openMeteoForecast(
  config: UpstreamConfig,
  params: URLSearchParams,
  latitude: number,
  longitude: number,
): Promise<unknown> {
  const url = withCell(new URL('/v1/forecast', openMeteoHost(config)), params, latitude, longitude)
  if (config.openMeteoKey) url.searchParams.set('apikey', config.openMeteoKey)
  return getJson(url, config)
}

export function openMeteoSearch(config: UpstreamConfig, params: URLSearchParams): Promise<unknown> {
  const url = new URL('/v1/search', 'https://geocoding-api.open-meteo.com')
  for (const [key, value] of params) url.searchParams.set(key, value)
  return getJson(url, config)
}

export function metNorwayCompact(
  config: UpstreamConfig,
  latitude: number,
  longitude: number,
): Promise<unknown> {
  const url = new URL('/weatherapi/locationforecast/2.0/compact', 'https://api.met.no')
  url.searchParams.set('lat', latitude.toFixed(3))
  url.searchParams.set('lon', longitude.toFixed(3))
  // Sans cet en-tête, MET Norway refuse la requête — c'est leur condition.
  return getJson(url, config, { 'User-Agent': USER_AGENT })
}

export function brightSkyCurrent(
  config: UpstreamConfig,
  latitude: number,
  longitude: number,
): Promise<unknown> {
  const url = new URL('/current_weather', 'https://api.brightsky.dev')
  url.searchParams.set('lat', latitude.toFixed(3))
  url.searchParams.set('lon', longitude.toFixed(3))
  return getJson(url, config)
}
