import { useEffect, useRef } from 'react'
import { locatesOnStart, parcelleFromPosition, type ParcelleOrigin } from '../position'
import type { Parcelle } from '../openMeteo'

/**
 * Cale la page sur la position de la personne, à la première visite seulement.
 *
 * Ne bloque rien : la parcelle par défaut se charge pendant que le navigateur
 * demande l'autorisation, et bascule quand la position arrive. Attendre la
 * réponse laisserait une page vide derrière la boîte de dialogue, pour un
 * geste que personne n'a demandé.
 *
 * Silencieux en cas d'échec, pour la même raison : un refus n'est pas une
 * erreur à afficher. On garde la parcelle par défaut, et la recherche de
 * commune reste là.
 */
export function useStartPosition(
  origin: ParcelleOrigin,
  name: string,
  select: (parcelle: Parcelle) => void,
): void {
  // Le nom et le rappel changent à chaque rendu ; la demande, non. On les lit
  // au moment de répondre plutôt que de relancer l'effet — deux demandes, ce
  // serait deux boîtes de dialogue.
  const nom = useRef(name)
  nom.current = name
  const choisir = useRef(select)
  choisir.current = select

  const demandee = useRef(false)

  useEffect(() => {
    if (demandee.current || !locatesOnStart(origin)) return
    if (typeof navigator === 'undefined' || !navigator.geolocation) return
    demandee.current = true

    navigator.geolocation.getCurrentPosition(
      (position) => choisir.current(parcelleFromPosition(nom.current, position.coords)),
      () => {
        // Refus, position indisponible, délai dépassé : rien à dire.
      },
      { timeout: 10_000, maximumAge: 5 * 60_000 },
    )
  }, [origin])
}
