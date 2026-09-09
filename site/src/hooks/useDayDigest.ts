import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  AgroApiError,
  dayDigest,
  fetchAgroForecast,
  fetchModelConsensus,
  type AgroForecast,
  type Consensus,
  type DayDigest,
  type Parcelle,
} from '@klima/core'
import { useI18n } from '@klima/core/ui'

interface State {
  forecast: AgroForecast | null
  /** Recoupement des modèles ; absent si la comparaison a échoué. */
  consensus: Consensus | null
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
  const { t } = useI18n()
  const [state, setState] = useState<State>({
    forecast: null,
    consensus: null,
    loading: true,
    error: null,
  })
  const [nonce, setNonce] = useState(0)

  useEffect(() => {
    const controller = new AbortController()
    setState((s) => ({ ...s, loading: true, error: null }))

    // Deux jours suffisent : aujourd'hui, et la nuit qui déborde sur demain.
    fetchAgroForecast(parcelle, 2, controller.signal)
      .then((forecast) => setState((s) => ({ ...s, forecast, loading: false, error: null })))
      .catch((error: unknown) => {
        if (controller.signal.aborted) return
        setState({
          forecast: null,
          consensus: null,
          loading: false,
          error:
            error instanceof AgroApiError
              ? t(error.messageKey, error.params)
              : t('today.error'),
        })
      })

    // Le recoupement est un plus : son échec ne prive de rien.
    fetchModelConsensus(parcelle, controller.signal)
      .then((consensus) => setState((s) => ({ ...s, consensus })))
      .catch(() => setState((s) => ({ ...s, consensus: null })))

    return () => controller.abort()
    // `t` change avec la langue ; le rechargement n'a pas à en dépendre.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [parcelle, nonce])

  const digest = useMemo(() => (state.forecast ? dayDigest(state.forecast) : null), [state.forecast])
  const reload = useCallback(() => setNonce((n) => n + 1), [])

  return { ...state, digest, reload }
}
