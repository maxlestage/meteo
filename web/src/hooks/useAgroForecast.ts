import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  AgroApiError,
  fetchAgroForecast,
  fetchModelConsensus,
  type AgroForecast,
  type Consensus,
  type Parcelle,
} from '@klima/core'
import { useI18n } from '@klima/core/ui'
import { summarize, type AgroSummary } from '@klima/core'

interface State {
  forecast: AgroForecast | null
  /** Recoupement des modèles ; absent si la comparaison a échoué. */
  consensus: Consensus | null
  loading: boolean
  error: string | null
}

/** Charge la prévision agricole d'une parcelle et en dérive les indicateurs. */
export function useAgroForecast(parcelle: Parcelle | null): State & {
  summary: AgroSummary | null
  reload: () => void
} {
  const { t } = useI18n()
  const [state, setState] = useState<State>({
    forecast: null,
    consensus: null,
    loading: false,
    error: null,
  })
  const [nonce, setNonce] = useState(0)

  useEffect(() => {
    if (!parcelle) {
      setState({ forecast: null, consensus: null, loading: false, error: null })
      return
    }

    const controller = new AbortController()
    setState((s) => ({ ...s, loading: true, error: null }))

    fetchAgroForecast(parcelle, 7, controller.signal)
      .then((forecast) => setState((s) => ({ ...s, forecast, loading: false, error: null })))
      .catch((error: unknown) => {
        if (controller.signal.aborted) return
        const message =
          error instanceof AgroApiError ? t(error.messageKey, error.params) : t('app.error')
        setState({ forecast: null, consensus: null, loading: false, error: message })
      })

    // Le recoupement est un plus : son échec ne prive de rien.
    fetchModelConsensus(parcelle, controller.signal)
      .then((consensus) => setState((s) => ({ ...s, consensus })))
      .catch(() => setState((s) => ({ ...s, consensus: null })))

    return () => controller.abort()
    // `t` change avec la langue ; le rechargement n'a pas à en dépendre.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [parcelle, nonce])

  const summary = useMemo(
    () => (state.forecast ? summarize(state.forecast.hourly, state.forecast.daily) : null),
    [state.forecast],
  )

  const reload = useCallback(() => setNonce((n) => n + 1), [])
  return { ...state, summary, reload }
}
