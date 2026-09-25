/**
 * Les cellules de cache, bâties sur la maille du cœur partagé.
 *
 * La maille et l'arrondi vivent dans `@klima/core` : l'application s'en
 * sert pour ne pas publier la position exacte de quelqu'un, le relais pour
 * ne pas interroger deux fois le même carré. C'est la même maille, et une
 * seule définition.
 */

import { CELL_DEGREES, snap } from '@klima/core'

export { CELL_DEGREES, snap }

export interface Cell {
  latitude: number
  longitude: number
}

/** La cellule dans laquelle tombe un point. */
export function cellFor(latitude: number, longitude: number): Cell {
  return { latitude: snap(latitude), longitude: snap(longitude) }
}

/** Clé de cache d'une cellule, pour un usage donné. */
export function cellKey(usage: string, cell: Cell): string {
  return `${usage}:${cell.latitude},${cell.longitude}`
}
