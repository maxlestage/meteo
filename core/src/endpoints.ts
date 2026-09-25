/**
 * Où l'on va chercher la météo.
 *
 * Deux acheminements, et le choix a des conséquences juridiques autant que
 * techniques :
 *
 * - **En direct.** Chaque appareil interroge les fournisseurs lui-même. Simple,
 *   sans infrastructure, mais la facture Open-Meteo suit le nombre
 *   d'utilisateurs, MET Norway reste hors de portée du navigateur, et le plan
 *   gratuit d'Open-Meteo interdit l'usage commercial.
 * - **Par le relais.** Un service interroge une fois pour tout le monde. Il
 *   détient la clé du plan commercial, pose l'en-tête que MET Norway exige, et
 *   mutualise le cache.
 *
 * Le choix se fait une fois au démarrage de l'application. Les fournisseurs
 * lisent l'acheminement courant plutôt que de recevoir une URL en paramètre :
 * il n'y a qu'un réglage, il vaut pour tout le processus, et l'exposer
 * partout n'apporterait rien.
 */

export type Transport = 'direct' | 'relais'

export interface Endpoints {
  transport: Transport
  openMeteoForecast: string
  openMeteoSearch: string
  metNorway: string
  brightSky: string
}

/** Chaque appareil pour soi : les adresses publiques des fournisseurs. */
export const DIRECT_ENDPOINTS: Endpoints = {
  transport: 'direct',
  openMeteoForecast: 'https://api.open-meteo.com/v1/forecast',
  openMeteoSearch: 'https://geocoding-api.open-meteo.com/v1/search',
  metNorway: 'https://api.met.no/weatherapi/locationforecast/2.0/compact',
  brightSky: 'https://api.brightsky.dev/current_weather',
}

/** Les mêmes fournisseurs, vus à travers le relais. */
export function relayEndpoints(origin: string): Endpoints {
  const base = origin.replace(/\/+$/, '')
  return {
    transport: 'relais',
    openMeteoForecast: `${base}/v1/open-meteo/forecast`,
    openMeteoSearch: `${base}/v1/open-meteo/search`,
    metNorway: `${base}/v1/met-norway/compact`,
    brightSky: `${base}/v1/bright-sky/current`,
  }
}

/**
 * La valeur qui dit « le relais, c'est l'hôte qui sert cette page ».
 *
 * Quand le relais sert lui-même l'application, son adresse n'est connue qu'au
 * moment de l'affichage : elle dépend du nom de domaine, qui change d'un
 * hébergement à l'autre et n'existe pas à la construction.
 */
export const MEME_ORIGINE = 'meme-origine'

/**
 * L'adresse du relais, d'après le réglage et l'origine de la page.
 *
 * Rien de configuré : pas de relais, chaque navigateur pour soi. C'est le
 * défaut, et il reste sur le plan gratuit d'Open-Meteo.
 */
export function relayFrom(configure: string | undefined, origine: string): string | null {
  const valeur = (configure ?? '').trim()
  if (!valeur) return null
  return valeur === MEME_ORIGINE ? origine : valeur
}

let current: Endpoints = DIRECT_ENDPOINTS

/** L'acheminement en vigueur. */
export function endpoints(): Endpoints {
  return current
}

/** Fait passer les appels par le relais donné. */
export function useRelay(origin: string): void {
  current = relayEndpoints(origin)
}

/** Revient aux appels directs — le défaut, et ce que font les tests. */
export function useDirectProviders(): void {
  current = DIRECT_ENDPOINTS
}
