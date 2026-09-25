/**
 * La maille de cache.
 *
 * Deux parcelles voisines partagent la même prévision : les modèles ne
 * distinguent pas deux points séparés de moins de leur propre résolution.
 * On arrondit donc les coordonnées à une maille, et tout ce qui tombe dans
 * la même cellule partage une seule interrogation du fournisseur.
 *
 * La maille vaut 0,02° — environ 2,2 km en latitude, 1,5 km en longitude à la
 * latitude de la France. C'est la résolution des modèles les plus fins qu'on
 * interroge (AROME à 1,3 km, ICON-D2 à 2 km) : arrondir plus grossièrement
 * ferait perdre de la précision à un outil dont c'est justement l'argument.
 *
 * Le coût n'en souffre pas, parce que le cache ne va chercher que les
 * cellules qu'on lui demande : la facture suit le nombre de parcelles
 * distinctes, pas la surface du pays. L'économie vient du temps — une
 * cellule interrogée une fois par heure sert tous ceux qui l'ouvrent.
 *
 * La même maille sert une seconde fois, et c'est la raison pour laquelle elle
 * vit ici plutôt que dans le relais : quand on prend la position de quelqu'un,
 * on l'arrondit avant d'en faire une parcelle. Une parcelle finit dans
 * l'adresse de la page, et une adresse se partage — il n'y a aucune raison d'y
 * écrire à deux mètres près où se tient la personne, alors que la prévision
 * est la même dans tout le carré.
 */

/** Côté de la maille, en degrés. */
export const CELL_DEGREES = 0.02

/** Nombre de décimales à garder après l'arrondi, pour une clé stable. */
const PRECISION = 3

/** Arrondit une coordonnée au centre de sa maille. */
export function snap(value: number): number {
  return Number((Math.round(value / CELL_DEGREES) * CELL_DEGREES).toFixed(PRECISION))
}
