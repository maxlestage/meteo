import { attributionsFor, type Consensus } from '@klima/core'
import { useI18n } from '@klima/core/ui'
import { useReveal } from '../hooks/useReveal'

interface Props {
  consensus: Consensus | null
  loading: boolean
}

/**
 * Le recoupement des modèles, montré plutôt qu'affirmé : chaque service porte
 * son point sur l'axe des températures, la médiane est marquée, et l'écart se
 * lit d'un coup d'œil.
 */
export function Sources({ consensus, loading }: Props) {
  const { t, f } = useI18n()
  const reveal = useReveal<HTMLDivElement>()

  return (
    <section className="sources" id="sources">
      <div className="section-head">
        <h2>{t('sources.title')}</h2>
        <p>{t('sources.lead')}</p>
      </div>

      <div ref={reveal.ref} className={`sources__panel ${reveal.className}`}>
        {loading && !consensus && <p className="sources__state">{t('sources.loading')}</p>}
        {!loading && !consensus && <p className="sources__state">{t('sources.unavailable')}</p>}

        {consensus && (
          <>
            <header className="sources__header">
              <div>
                <p className="sources__label">{t('consensus.title')}</p>
                <p className={`sources__verdict sources__verdict--${consensus.agreement}`}>
                  {t(`consensus.${consensus.agreement}`)}
                </p>
              </div>
              <div className="sources__median">
                <p className="sources__label">{t('sources.now')}</p>
                <p className="sources__value">{f.unit(consensus.temperature.median, '°C')}</p>
                <p className="sources__providers">
                  {t('sources.answered', {
                    answered: consensus.providersAnswered,
                    queried: consensus.providersQueried,
                  })}
                </p>
                <p className="sources__spread">
                  {t('consensus.detail', {
                    count: consensus.readings.length,
                    spread: f.unit(consensus.temperature.spread, '°C'),
                  })}
                  {consensus.agreeOnRain ? '' : ` · ${t('consensus.rainDisagreement')}`}
                </p>
              </div>
            </header>

            <ul className="models">
              {consensus.readings.map((reading) => {
                const source = reading.source
                const range = Math.max(consensus.temperature.spread, 0.1)
                const offset = ((reading.temperature - consensus.temperature.min) / range) * 100

                return (
                  <li className="models__row" key={source.id}>
                    <span className="models__flag">{source.country}</span>
                    <span className="models__name">
                      <strong>{source.institution}</strong>
                      <span>{source.name}</span>
                    </span>
                    <span className="models__axis">
                      <span className="models__dot" style={{ left: `${offset}%` }} />
                    </span>
                    <span className="models__value">{f.unit(reading.temperature, '°C')}</span>
                  </li>
                )
              })}
            </ul>
          </>
        )}
      </div>

      <div className="sources__method">
        <h3>{t('sources.method')}</h3>
        <p>{t('sources.methodBody')}</p>
        <p className="sources__note">{t('sources.note')}</p>
        {consensus && (
          <ul className="sources__attributions">
            {attributionsFor(consensus.readings).map((attribution) => (
              <li key={attribution}>{attribution}</li>
            ))}
          </ul>
        )}
      </div>
    </section>
  )
}
