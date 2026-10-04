/**
 * Ce que le déploiement accorde comme palier.
 *
 * Un abonnement se vend par l'App Store, et StoreKit en tient le registre.
 * Mais pendant l'essai, personne n'achète : les testeurs doivent voir le palier
 * payant sans passer par la boutique, et l'auteur doit pouvoir le rendre à
 * n'importe quel moment sans republier une version.
 *
 * D'où cette variable, et sa place : **sur le serveur, pas dans le binaire**.
 * Une valeur glissée dans une application distribuée est une valeur publiée,
 * qu'on ne peut plus retirer qu'en publiant une nouvelle version ; ici, elle
 * s'enlève en une commande et le palier redescend au prochain démarrage.
 *
 * `KLIMA_PRO` dit laquelle des trois situations on est :
 *
 * | Valeur                    | Ce que le relais répond                     |
 * | ------------------------- | ------------------------------------------- |
 * | absente, vide, `0`, `non` | rien : StoreKit décide seul                 |
 * | `tous`, `1`, `oui`        | le palier payant, à qui demande             |
 * | n'importe quoi d'autre    | le palier payant, à qui présente ce code    |
 *
 * La troisième forme existe parce que la seconde est franche : un relais
 * public qui accorde le palier à qui demande l'accorde à **tout le monde**.
 * C'est exactement ce qu'on veut tant que les seuls clients sont les testeurs
 * qu'on a invités ; passé là, un code vaut mieux.
 */

export type Accord =
  | { readonly type: 'aucun' }
  | { readonly type: 'tous' }
  | { readonly type: 'code'; readonly code: string }

const RIEN = ['', '0', 'non', 'no', 'false']
const TOUT = ['tous', '1', 'oui', 'yes', 'true', 'all']

/** Lit la variable d'environnement. */
export function accordDepuis(valeur: string | undefined): Accord {
  const propre = (valeur ?? '').trim()
  const minuscule = propre.toLowerCase()

  if (RIEN.includes(minuscule)) return { type: 'aucun' }
  if (TOUT.includes(minuscule)) return { type: 'tous' }
  return { type: 'code', code: propre }
}

/** Ce que `/health` peut dire sans rien révéler. */
export function etiquette(accord: Accord): string {
  return accord.type === 'code' ? 'sur code' : accord.type
}

/** Le palier accordé à cette demande. */
export function accorde(accord: Accord, code: string | null): boolean {
  switch (accord.type) {
    case 'aucun':
      return false
    case 'tous':
      return true
    case 'code':
      return code !== null && egal(accord.code, code)
  }
}

/**
 * Comparaison à durée constante.
 *
 * Un `===` ordinaire s'arrête au premier caractère qui diffère : en mesurant le
 * temps de réponse, on devine le code lettre par lettre. Le coût de s'en
 * prémunir est de quelques microsecondes.
 */
function egal(attendu: string, donne: string): boolean {
  let difference = attendu.length ^ donne.length
  for (let i = 0; i < Math.max(attendu.length, donne.length); i += 1) {
    difference |= (attendu.charCodeAt(i) || 0) ^ (donne.charCodeAt(i) || 0)
  }
  return difference === 0
}
