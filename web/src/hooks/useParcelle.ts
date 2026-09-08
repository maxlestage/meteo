import { useCallback, useEffect, useState } from 'react'
import type { Parcelle } from '../api/openMeteo'

const STORAGE_KEY = 'klima.parcelle'

/** Parcelle par défaut : plaine céréalière de Beauce. */
const DEFAULT_PARCELLE: Parcelle = {
  name: 'Chartres',
  latitude: 48.4468,
  longitude: 1.4892,
  admin: 'Eure-et-Loir',
  country: 'France',
}

/** Parcelle courante, mémorisée d'une visite à l'autre. */
export function useParcelle(): [Parcelle, (parcelle: Parcelle) => void] {
  const [parcelle, setParcelle] = useState<Parcelle>(() => readStored() ?? DEFAULT_PARCELLE)

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(parcelle))
    } catch {
      // Navigation privée ou stockage plein : on garde la parcelle en mémoire.
    }
  }, [parcelle])

  const select = useCallback((next: Parcelle) => setParcelle(next), [])
  return [parcelle, select]
}

function readStored(): Parcelle | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
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
