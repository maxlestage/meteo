/**
 * Recoupement de plusieurs modèles de prévision.
 *
 * Une seule source donne une réponse ; plusieurs sources donnent une réponse
 * *et* une idée de sa fiabilité. Quand les modèles s'accordent, on peut
 * annoncer un chiffre sans réserve ; quand ils divergent, il faut le dire
 * plutôt que d'afficher une fausse précision.
 */
import type { ProviderOutcome, SourceReading } from './providers'

/** Degré d'accord entre les sources. */
export type Agreement = 'forte' | 'moyenne' | 'faible'

export interface Spread {
  /** Valeur retenue : la médiane, moins sensible qu'une moyenne à un modèle isolé. */
  median: number
  min: number
  max: number
  /** Écart entre les extrêmes. */
  spread: number
}

export interface Consensus {
  readings: SourceReading[]
  /** Fournisseurs ayant répondu, sur ceux qui ont été interrogés. */
  providersAnswered: number
  providersQueried: number
  temperature: Spread
  precipitation: Spread
  windSpeed: Spread
  /** Vrai si tous les modèles s'accordent sur la présence ou l'absence de pluie. */
  agreeOnRain: boolean
  agreement: Agreement
}

/** Seuils de lecture de l'accord entre modèles. */
export const ConsensusThresholds = {
  /** Écart de température en deçà duquel l'accord est jugé fort (°C). */
  strongTemperatureSpread: 1.5,
  /** Au-delà, l'accord est jugé faible (°C). */
  weakTemperatureSpread: 3,
  /** Pluie considérée comme annoncée à partir de ce cumul horaire (mm). */
  rainThreshold: 0.1,
} as const

/**
 * Recoupe les relevés. Renvoie `null` s'il n'y a rien à comparer : un seul
 * modèle ne fait pas un consensus, et le dire vaut mieux que de le laisser
 * croire.
 */
export function consensus(
  readings: readonly SourceReading[],
  providers: { answered: number; queried: number } = { answered: 0, queried: 0 },
): Consensus | null {
  if (readings.length < 2) return null

  const temperature = spread(readings.map((r) => r.temperature))
  const precipitation = spread(readings.map((r) => r.precipitation))
  const windSpeed = spread(readings.map((r) => r.windSpeed))

  const rainy = readings.map((r) => r.precipitation >= ConsensusThresholds.rainThreshold)
  const agreeOnRain = rainy.every((value) => value === rainy[0])

  const distinctProviders = new Set(readings.map((r) => r.source.provider)).size

  return {
    readings: [...readings],
    providersAnswered: providers.answered || distinctProviders,
    providersQueried: providers.queried || distinctProviders,
    temperature,
    precipitation,
    windSpeed,
    agreeOnRain,
    agreement: agreementFrom(temperature.spread, agreeOnRain),
  }
}

/**
 * Recoupe ce que les fournisseurs ont renvoyé, en gardant trace de ceux qui
 * n'ont rien pu dire : l'interface doit pouvoir annoncer « 5 sources sur 6 ».
 */
export function consensusFromOutcomes(outcomes: readonly ProviderOutcome[]): Consensus | null {
  const readings = outcomes.flatMap((outcome) => outcome.readings)
  const answered = outcomes.filter((outcome) => outcome.readings.length > 0).length
  return consensus(readings, { answered, queried: outcomes.length })
}

/**
 * L'accord se juge d'abord sur la température — la variable la mieux prévue —
 * puis sur le désaccord franc que constitue « il pleut / il ne pleut pas ».
 */
function agreementFrom(temperatureSpread: number, agreeOnRain: boolean): Agreement {
  if (temperatureSpread > ConsensusThresholds.weakTemperatureSpread) return 'faible'
  if (!agreeOnRain) return 'moyenne'
  return temperatureSpread <= ConsensusThresholds.strongTemperatureSpread ? 'forte' : 'moyenne'
}

function spread(values: readonly number[]): Spread {
  const sorted = [...values].sort((a, b) => a - b)
  const min = sorted[0]!
  const max = sorted[sorted.length - 1]!
  return {
    median: round(median(sorted), 1),
    min: round(min, 1),
    max: round(max, 1),
    spread: round(max - min, 1),
  }
}

function median(sorted: readonly number[]): number {
  const middle = Math.floor(sorted.length / 2)
  if (sorted.length % 2 === 1) return sorted[middle]!
  return (sorted[middle - 1]! + sorted[middle]!) / 2
}

function round(value: number, decimals: number): number {
  const factor = 10 ** decimals
  return Math.round(value * factor) / factor
}
