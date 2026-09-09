/**
 * Le registre : les conditions à l'heure d'un traitement.
 *
 * Miroir Swift : `ios/Klima/Models/Register.swift`.
 *
 * Tenir un registre des traitements phytosanitaires est une obligation. Klima
 * **n'est pas ce registre** et ne prétend pas l'être : il fournit la partie
 * pénible à reconstituer après coup — les conditions météo relevées à l'heure
 * de l'application. Le reste (produit, dose, culture, opérateur) appartient à
 * l'exploitant, et le module se contente de le recopier s'il le fournit.
 *
 * Ce que la réglementation regarde à l'heure d'une application : le vent, ses
 * rafales, la température et l'hygrométrie. On y ajoute la pluie de l'heure et
 * le verdict de Klima, qui n'a aucune valeur réglementaire mais dit ce que
 * l'outil aurait conseillé.
 */
import { evaluateSprayHour, type HourlySample, type SprayVerdict } from './agro'

export interface TreatmentRecord {
  /** Heure de l'application, dans le fuseau de la parcelle. */
  at: Date
  parcelle: string
  /** Renseigné par l'exploitant : Klima ne l'invente pas. */
  product?: string
  temperature: number
  relativeHumidity: number
  windSpeed: number
  windGusts: number
  precipitation: number
  /** Ce que Klima aurait conseillé — indicatif, sans valeur réglementaire. */
  verdict: SprayVerdict
  score: number
}

/**
 * Relève les conditions de l'heure qui contient `at`.
 *
 * Renvoie `null` si la série ne couvre pas ce moment : mieux vaut une ligne
 * absente qu'une ligne inventée dans un document qu'on pourra vous opposer.
 */
export function recordAt(
  hours: readonly HourlySample[],
  at: Date,
  parcelle: string,
  product?: string,
): TreatmentRecord | null {
  const slot = Math.floor(at.getTime() / 3_600_000) * 3_600_000
  const index = hours.findIndex((hour) => Math.floor(hour.time.getTime() / 3_600_000) === slot / 3_600_000)
  if (index < 0) return null

  const hour = hours[index]!
  const window = evaluateSprayHour(hours, index)

  return {
    at: hour.time,
    parcelle,
    ...(product === undefined ? {} : { product }),
    temperature: hour.temperature,
    relativeHumidity: hour.relativeHumidity,
    windSpeed: hour.windSpeed,
    windGusts: hour.windGusts,
    precipitation: hour.precipitation,
    verdict: window.verdict,
    score: window.score,
  }
}

/** Les colonnes du document, dans l'ordre. Les en-têtes sont des clés. */
export const REGISTER_COLUMNS = [
  'date',
  'heure',
  'parcelle',
  'produit',
  'temperature',
  'humidite',
  'vent',
  'rafales',
  'pluie',
  'verdict',
  'score',
] as const
export type RegisterColumn = (typeof REGISTER_COLUMNS)[number]

export interface CsvOptions {
  /**
   * Séparateur de colonnes. Le point-virgule par défaut : Excel en langue
   * française lit un fichier à virgules comme une seule colonne.
   */
  delimiter?: string
  /** Séparateur décimal. La virgule par défaut, pour la même raison. */
  decimal?: string
  /** En-têtes déjà traduits, dans l'ordre de `REGISTER_COLUMNS`. */
  headers?: readonly string[]
}

function escape(value: string, delimiter: string): string {
  // Un guillemet se double, et tout champ qui contient un séparateur, un
  // guillemet ou un saut de ligne se met entre guillemets. Sans ça, une
  // parcelle nommée « Le Clos ; bas » casserait la colonne suivante.
  const needsQuotes = value.includes(delimiter) || value.includes('"') || /[\r\n]/.test(value)
  const escaped = value.replace(/"/g, '""')
  return needsQuotes ? `"${escaped}"` : escaped
}

function two(value: number): string {
  return String(value).padStart(2, '0')
}

/**
 * Met les relevés en CSV.
 *
 * Le fichier commence par une marque d'ordre des octets : sans elle, Excel
 * lit l'UTF-8 comme du Latin-1 et « évapotranspiration » perd ses accents.
 */
export function toCsv(records: readonly TreatmentRecord[], options: CsvOptions = {}): string {
  const delimiter = options.delimiter ?? ';'
  const decimal = options.decimal ?? ','
  const headers = options.headers ?? REGISTER_COLUMNS

  const number = (value: number, digits = 1) =>
    value.toFixed(digits).replace('.', decimal)

  const lines = [headers.map((header) => escape(String(header), delimiter)).join(delimiter)]

  for (const record of records) {
    const date = record.at
    lines.push(
      [
        `${date.getFullYear()}-${two(date.getMonth() + 1)}-${two(date.getDate())}`,
        `${two(date.getHours())}:${two(date.getMinutes())}`,
        escape(record.parcelle, delimiter),
        escape(record.product ?? '', delimiter),
        number(record.temperature),
        number(record.relativeHumidity, 0),
        number(record.windSpeed),
        number(record.windGusts),
        number(record.precipitation),
        record.verdict,
        String(record.score),
      ].join(delimiter),
    )
  }

  return `﻿${lines.join('\r\n')}\r\n`
}
