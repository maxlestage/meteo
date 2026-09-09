import { useParcelleInUrl } from '@klima/core/ui'
import type { Parcelle } from '@klima/core'

/** Parcelle par défaut : plaine céréalière de Beauce. */
const DEFAULT_PARCELLE: Parcelle = {
  name: 'Chartres',
  latitude: 48.4468,
  longitude: 1.4892,
  admin: 'Eure-et-Loir',
  country: 'France',
}

/**
 * Parcelle courante de l'application, tenue dans l'adresse et mémorisée d'une
 * visite à l'autre. La mécanique est partagée avec la vitrine ; seule la
 * mémorisation lui est propre — c'est un outil qu'on rouvre, pas une page
 * qu'on visite.
 */
export function useParcelle(): [Parcelle, (parcelle: Parcelle) => void] {
  return useParcelleInUrl(DEFAULT_PARCELLE, { storageKey: 'klima.parcelle' })
}
