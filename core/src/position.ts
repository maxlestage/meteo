/**
 * Prendre la position de la personne, plutôt que lui montrer Chartres.
 *
 * Une application météo qui s'ouvre sur une ville qu'on n'a pas choisie
 * demande un geste avant d'être utile. La montre le faisait déjà ; le
 * téléphone et le web, non.
 *
 * Deux règles, et elles tiennent ensemble :
 *
 * - **On ne demande qu'à défaut.** Une adresse partagée désigne une parcelle,
 *   et une parcelle déjà choisie a été choisie : aller chercher la position
 *   par-dessus reviendrait à défaire le geste de quelqu'un. Faute des deux, il
 *   n'y a rien à défaire.
 * - **On arrondit avant d'en faire une parcelle.** La parcelle part dans
 *   l'adresse de la page, et une adresse se partage. On la ramène donc au
 *   centre de sa maille — la prévision y est la même, et le lien ne dit plus
 *   où la personne se tient à deux mètres près.
 */

import { snap } from './grid'
import type { Parcelle } from './openMeteo'

/** D'où vient la parcelle affichée au démarrage. */
export type ParcelleOrigin =
  /** L'adresse la nommait : un lien partagé, un favori. */
  | 'adresse'
  /** La dernière consultée, retrouvée dans la mémoire du navigateur. */
  | 'memoire'
  /** Rien ne la désignait : c'est celle par défaut. */
  | 'defaut'

/**
 * Faut-il aller chercher la position au démarrage ?
 *
 * Seulement quand rien n'a été choisi. Le reste du temps, la parcelle affichée
 * est le résultat d'une décision — celle de la personne, ou celle de qui lui a
 * envoyé le lien.
 */
export function locatesOnStart(origin: ParcelleOrigin): boolean {
  return origin === 'defaut'
}

/** Une position, telle que le navigateur ou CoreLocation la rendent. */
export interface Position {
  latitude: number
  longitude: number
}

/**
 * La parcelle d'une position, arrondie à la maille.
 *
 * Le nom vient de l'interface : le domaine ne fabrique pas de phrases.
 */
export function parcelleFromPosition(name: string, position: Position): Parcelle {
  return {
    name,
    latitude: snap(position.latitude),
    longitude: snap(position.longitude),
  }
}
