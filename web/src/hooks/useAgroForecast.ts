import { useCallback, useEffect, useMemo, useState } from 'react'
import { AgroApiError, fetchAgroForecast, type AgroForecast, type Parcelle } from '@klima/core'
import { useI18n } from '@klima/core/ui'
import { summarize, type AgroSummary } from '@klima/core'

interface State {
  forecast: AgroForecast | null
  loading: boolean
  error: string | null
}

/** Charge la prévision agricole d'une parcelle et en dérive les indicateurs. */
export function useAgroForecast(parcelle: Parcelle | null): State & {
  summary: AgroSummary | null
  reload: () => void
} {
  const { t } = useI18n()
  const [state, setState] = useState<State>({ forecast: null, loading: false, error: null })
  const [nonce, setNonce] = useState(0)

  useEffect(() => {
    if (!parcelle) {
      setState({ forecast: null, loading: false, error: null })
      return
    }

    const controller = new AbortController()
    setState((s) => ({ ...s, loading: true, error: null }))

    fetchAgroForecast(parcelle, 7, controller.signal)
      .then((forecast) => setState({ forecast, loading: false, error: null }))
      .catch((error: unknown) => {
        if (controller.signal.aborted) return
        const message =
          error instanceof AgroApiError ? t(error.messageKey, error.params) : t('app.error')
        setState({ forecast: null, loading: false, error: message })
      })

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
