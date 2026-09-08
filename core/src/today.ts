/**
 * Synthèse de la journée en cours, pour la section « météo du jour » du site
 * de présentation : le temps qu'il fera aujourd'hui, et ce que Klima en déduit
 * pour la parcelle.
 */
import {
  frostRisk,
  nextSprayOpportunity,
  soilCondition,
  sprayWindows,
  type FrostRisk,
  type HourlySample,
  type SoilCondition,
  type SprayOpportunity,
} from './agro'
import type { AgroForecast } from './openMeteo'

export interface DayDigest {
  date: Date
  weatherCode: number
  temperatureMin: number
  temperatureMax: number
  /** Cumul de pluie attendu sur la journée (mm). */
  precipitationSum: number
  precipitationProbabilityMax: number
  /** Évapotranspiration de référence du jour (mm). */
  et0Sum: number
  /** Pluie − ET0 sur la seule journée (mm). */
  balance: number
  windGustsMax: number
  sunrise: Date | null
  sunset: Date | null
  /** Heures restantes de la journée, à partir de l'heure en cours. */
  remainingHours: HourlySample[]
  /** Fenêtre de traitement d'ici ce soir, s'il en reste une. */
  spray: SprayOpportunity | null
  soil: SoilCondition
  /** Gel attendu la nuit prochaine — elle déborde sur le lendemain. */
  frost: FrostRisk
}

/**
 * Journée en cours d'une prévision. Renvoie `null` si la série ne couvre pas
 * aujourd'hui, ce qui ne devrait arriver qu'avec une réponse tronquée.
 */
export function dayDigest(forecast: AgroForecast): DayDigest | null {
  const today = forecast.daily[0]
  if (!today) return null

  const key = dayKey(today.date, forecast.timezone)
  const remainingHours = forecast.hourly.filter((h) => dayKey(h.time, forecast.timezone) === key)

  // Le gel se juge sur la nuit qui vient, laquelle déborde sur le lendemain.
  const tonight = forecast.hourly.slice(0, 18)

  return {
    date: today.date,
    weatherCode: today.weatherCode,
    temperatureMin: today.temperatureMin,
    temperatureMax: today.temperatureMax,
    precipitationSum: today.precipitationSum,
    precipitationProbabilityMax: today.precipitationProbabilityMax,
    et0Sum: today.et0Sum,
    balance: Math.round((today.precipitationSum - today.et0Sum) * 10) / 10,
    windGustsMax: today.windGustsMax,
    sunrise: today.sunrise,
    sunset: today.sunset,
    remainingHours,
    spray: nextSprayOpportunity(sprayWindows(remainingHours)),
    soil: soilCondition(remainingHours),
    frost: frostRisk(
      tonight.length > 0 ? Math.min(...tonight.map((h) => h.temperature)) : today.temperatureMin,
      tonight.length > 0 ? Math.min(...tonight.map((h) => h.dewPoint)) : 0,
    ),
  }
}

/** Jour civil d'un instant, dans le fuseau de la parcelle (« 2026-05-12 »). */
function dayKey(date: Date, timeZone: string): string {
  return new Intl.DateTimeFormat('en-CA', {
    timeZone,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(date)
}
