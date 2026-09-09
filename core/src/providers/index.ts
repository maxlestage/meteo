import { brightSkyProvider } from './brightSky'
import { metNorwayProvider } from './metNorway'
import { openMeteoProvider } from './openMeteo'
import type { Platform, Provider, ProviderQuery, SourceReading, WeatherSource } from './types'

export * from './types'
export { openMeteoProvider } from './openMeteo'
export { metNorwayProvider } from './metNorway'
export { brightSkyProvider } from './brightSky'

import { OPEN_METEO_SOURCES } from './openMeteo'
import { MET_NORWAY_SOURCE } from './metNorway'
import { BRIGHT_SKY_SOURCE } from './brightSky'

export { OPEN_METEO_SOURCES, MET_NORWAY_SOURCE, BRIGHT_SKY_SOURCE }

export const PROVIDERS: readonly Provider[] = [
  openMeteoProvider,
  metNorwayProvider,
  brightSkyProvider,
]

/** Fournisseurs appelables depuis la plateforme donnée. */
export function providersFor(platform: Platform): Provider[] {
  return PROVIDERS.filter((provider) => provider.platforms.includes(platform))
}

export interface ProviderOutcome {
  provider: Provider
  readings: SourceReading[]
  /** Renseigné quand l'appel a échoué : le fournisseur est écarté, pas l'ensemble. */
  error?: unknown
}

/**
 * Interroge tous les fournisseurs d'une plateforme en parallèle.
 *
 * Chaque fournisseur est isolé : une panne, un refus ou une absence de
 * couverture en écarte un seul. Le recoupement se fait sur ce qui a répondu, et
 * l'interface peut dire combien de sources ont parlé.
 */
export async function fetchAllReadings(
  query: ProviderQuery,
  platform: Platform,
  signal?: AbortSignal,
): Promise<ProviderOutcome[]> {
  const providers = providersFor(platform)
  const results = await Promise.allSettled(
    providers.map((provider) => provider.fetch(query, signal)),
  )

  return results.map((result, index) => {
    const provider = providers[index]!
    return result.status === 'fulfilled'
      ? { provider, readings: result.value }
      : { provider, readings: [], error: result.reason }
  })
}

/** Toutes les sources connues, tous fournisseurs confondus. */
export const ALL_SOURCES: readonly WeatherSource[] = [
  ...OPEN_METEO_SOURCES,
  MET_NORWAY_SOURCE,
  BRIGHT_SKY_SOURCE,
]

export function weatherSource(id: string): WeatherSource | undefined {
  return ALL_SOURCES.find((source) => source.id === id)
}

/** Mentions à afficher pour les sources effectivement utilisées. */
export function attributionsFor(readings: readonly SourceReading[]): string[] {
  return [...new Set(readings.map((reading) => reading.source.attribution))]
}
