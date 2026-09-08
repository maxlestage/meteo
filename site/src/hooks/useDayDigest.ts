import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  AgroApiError,
  dayDigest,
  fetchAgroForecast,
  type AgroForecast,
  type DayDigest,
  type Parcelle,
} from '@klima/core'

interface State {
  forecast: AgroForecast | null
  loading: boolean
  error: string | null
}

/**
 * Charge la prévision d'une parcelle et n'en garde que la journée en cours :
 * le site ne montre pas la semaine, c'est le rôle de l'application.
 */
export function useDayDigest(parcelle: Parcelle): State & {
  digest: DayDigest | null
  reload: () => void
} {
  const [state, setState] = useState<State>({ forecast: null, loading: true, error: null })
  const [nonce, setNonce] = useState(0)

  useEffect(() => {
    const controller = new AbortController()
    setState((s) => ({ ...s, loading: true, error: null }))

    // Deux jours suffisent : aujourd'hui, et la nuit qui déborde sur demain.
    fetchAgroForecast(parcelle, 2, controller.signal)
      .then((forecast) => setState({ forecast, loading: false, error: null }))
      .catch((error: unknown) => {
        if (controller.signal.aborted) return
        setState({
          forecast: null,
          loading: false,
          error:
            error instanceof AgroApiError ? error.message : 'Impossible de charger la météo du jour.',
        })
      })

    return () => controller.abort()
  }, [parcelle, nonce])

  const digest = useMemo(() => (state.forecast ? dayDigest(state.forecast) : null), [state.forecast])
  const reload = useCallback(() => setNonce((n) => n + 1), [])

  return { ...state, digest, reload }
}
