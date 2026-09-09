import { useCallback, useEffect, useRef, useState } from 'react'
import { parcelleFromParams, sameParcelle, urlForParcelle } from '../parcelleUrl'
import type { Parcelle } from '../openMeteo'

/**
 * La parcelle consultée, tenue dans l'adresse.
 *
 * Sans cela, choisir une commune ne laisse aucune trace : le bouton retour du
 * navigateur ne défait rien, recharger la page perd le choix, et envoyer
 * l'adresse à quelqu'un lui montre une autre parcelle que la sienne. Trois
 * défauts pour une seule cause — l'état n'était pas dans l'URL.
 *
 * L'ordre de préférence au démarrage : ce que dit l'adresse — elle est
 * explicite, et c'est elle qu'on a partagée —, puis la dernière parcelle
 * consultée si on la mémorise, puis celle par défaut.
 */
export interface UrlParcelleOptions {
  /** Clé de stockage local. Absente : rien n'est mémorisé d'une visite à l'autre. */
  storageKey?: string
}

export function useParcelleInUrl(
  fallback: Parcelle,
  options: UrlParcelleOptions = {},
): [Parcelle, (parcelle: Parcelle) => void] {
  const { storageKey } = options
  const [parcelle, setParcelle] = useState<Parcelle>(
    () => readUrl() ?? readStored(storageKey) ?? fallback,
  )

  // La valeur courante, lisible depuis un rappel stable. On ne compare pas
  // dans un `setState` : React rejoue les mises à jour en mode strict, et
  // l'historique se retrouverait avec deux entrées pour un seul choix.
  const current = useRef(parcelle)

  useEffect(() => {
    current.current = parcelle
    if (!storageKey) return
    try {
      localStorage.setItem(storageKey, JSON.stringify(parcelle))
    } catch {
      // Navigation privée ou stockage plein : on garde la parcelle en mémoire.
    }
  }, [parcelle, storageKey])

  // Le bouton retour du navigateur remonte ici : l'adresse a changé sans que
  // l'application le sache, on se remet sur ce qu'elle dit.
  useEffect(() => {
    const onPop = () => setParcelle(readUrl() ?? fallback)
    window.addEventListener('popstate', onPop)
    return () => window.removeEventListener('popstate', onPop)
    // `fallback` est une constante de module chez les deux appelants.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const select = useCallback((next: Parcelle) => {
    // Rechoisir la même commune n'a pas à créer une étape d'historique :
    // sinon il faudrait appuyer trois fois sur retour pour rien.
    if (sameParcelle(current.current, next)) return
    window.history.pushState(null, '', urlForParcelle(window.location.href, next))
    current.current = next
    setParcelle(next)
  }, [])

  return [parcelle, select]
}

/** La parcelle que désigne l'adresse, si elle en désigne une de valide. */
function readUrl(): Parcelle | null {
  try {
    return parcelleFromParams(new URLSearchParams(window.location.search))
  } catch {
    return null
  }
}

function readStored(key: string | undefined): Parcelle | null {
  if (!key) return null
  try {
    const raw = localStorage.getItem(key)
    if (!raw) return null
    const parsed: unknown = JSON.parse(raw)
    if (
      typeof parsed === 'object' &&
      parsed !== null &&
      typeof (parsed as Parcelle).name === 'string' &&
      typeof (parsed as Parcelle).latitude === 'number' &&
      typeof (parsed as Parcelle).longitude === 'number'
    ) {
      return parsed as Parcelle
    }
    return null
  } catch {
    return null
  }
}
