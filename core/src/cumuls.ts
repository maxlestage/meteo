/**
 * Les cumuls depuis une date choisie.
 *
 * Miroir Swift : `ios/Klima/Models/Cumuls.swift`.
 *
 * Un agriculteur ne raisonne pas en « sept derniers jours » mais depuis un
 * événement : le semis, le dernier traitement, la reprise de végétation. Ce
 * module additionne la pluie, l'évapotranspiration et les degrés-jours depuis
 * la date qu'on lui donne.
 *
 * Il dit toujours **ce qu'il a réellement couvert**. Une série de prévision ne
 * remonte pas dans le passé : demander le cumul depuis un semis d'octobre à
 * une série qui commence hier donnerait un chiffre faux, et un chiffre faux
 * dans un outil de décision est pire qu'une absence de chiffre. Le résultat
 * porte donc les bornes effectives et le nombre de jours manquants.
 */
import { AgroThresholds, growingDegreeDays, type DailySample } from './agro'

export interface Cumul {
  /** Date demandée. */
  requestedFrom: Date
  /** Première journée réellement disponible dans la série. */
  from: Date
  /** Dernière journée prise en compte. */
  to: Date
  /** Journées effectivement additionnées. */
  days: number
  /**
   * Journées demandées qui manquent à la série. Zéro quand la couverture est
   * complète ; au-delà, le cumul est un minorant.
   */
  missingDays: number
  /** Cumul de pluie (mm). */
  precipitation: number
  /** Cumul d'évapotranspiration de référence (mm). */
  evapotranspiration: number
  /** Pluie − ET0 (mm). Négatif : la parcelle a puisé dans sa réserve. */
  balance: number
  /** Degrés-jours capitalisés depuis la date. */
  gdd: number
}

/** Vrai si le cumul couvre toute la période demandée. */
export function isComplete(cumul: Cumul): boolean {
  return cumul.missingDays === 0
}

const DAY_MS = 86_400_000

function atMidnight(date: Date): Date {
  const midnight = new Date(date)
  midnight.setHours(0, 0, 0, 0)
  return midnight
}

function round(value: number, digits = 1): number {
  const factor = 10 ** digits
  return Math.round(value * factor) / factor
}

/**
 * Additionne les journées depuis `from` incluse.
 *
 * Renvoie `null` quand aucune journée de la série n'entre dans la période :
 * il n'y a alors rien d'honnête à afficher.
 */
export function accumulate(
  days: readonly DailySample[],
  from: Date,
  base: number = AgroThresholds.gddBase,
): Cumul | null {
  const start = atMidnight(from)
  const kept = days.filter((day) => atMidnight(day.date).getTime() >= start.getTime())
  if (kept.length === 0) return null

  const first = atMidnight(kept[0]!.date)
  const last = atMidnight(kept[kept.length - 1]!.date)

  const precipitation = kept.reduce((sum, day) => sum + day.precipitationSum, 0)
  const evapotranspiration = kept.reduce((sum, day) => sum + day.et0Sum, 0)
  const gdd = kept.reduce(
    (sum, day) => sum + growingDegreeDays(day.temperatureMin, day.temperatureMax, base),
    0,
  )

  return {
    requestedFrom: start,
    from: first,
    to: last,
    days: kept.length,
    // Ce que la série ne couvre pas : les journées entre la demande et son début.
    missingDays: Math.max(0, Math.round((first.getTime() - start.getTime()) / DAY_MS)),
    precipitation: round(precipitation),
    evapotranspiration: round(evapotranspiration),
    balance: round(precipitation - evapotranspiration),
    gdd: round(gdd),
  }
}
