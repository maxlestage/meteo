import { useState } from 'react'
import { AgroThresholds, type Parcelle } from '@klima/core'
import { LanguageSwitcher, useI18n } from '@klima/core/ui'
import { Features } from './components/Features'
import { PhoneMockup } from './components/PhoneMockup'
import { TodaySection } from './components/TodaySection'
import { useDayDigest } from './hooks/useDayDigest'

/** Plaine céréalière de Beauce, au premier chargement. */
const DEFAULT_PARCELLE: Parcelle = {
  name: 'Chartres',
  latitude: 48.4468,
  longitude: 1.4892,
  admin: 'Eure-et-Loir',
  country: 'France',
}

export default function App() {
  const { t, f } = useI18n()
  const [parcelle, setParcelle] = useState<Parcelle>(DEFAULT_PARCELLE)
  const { digest, loading, error, reload, forecast } = useDayDigest(parcelle)

  // Le crédit encadre un lien : on coupe la phrase autour de son jeton.
  const [creditBefore = '', creditAfter = ''] = t('footer.credit', { link: '\u0000' }).split('\u0000')
  const credit = { before: creditBefore, after: creditAfter }

  return (
    <>
      <header className="nav">
        <a className="nav__brand" href="#top">
          Klima
        </a>
        <div className="nav__end">
          <nav>
            <a href="#aujourdhui">{t('nav.today')}</a>
            <a href="#indicateurs">{t('nav.indicators')}</a>
            <a href="#donnees">{t('nav.data')}</a>
          </nav>
          <LanguageSwitcher className="lang" label={t('language.label')} />
        </div>
      </header>

      <main id="top">
        <section className="hero">
          <div className="hero__text">
            <p className="hero__eyebrow">{t('hero.eyebrow')}</p>
            <h1>{t('hero.title')}</h1>
            <p className="hero__lead">{t('hero.lead')}</p>
            <div className="hero__actions">
              <a className="button" href="#aujourdhui">
                {t('hero.cta.today')}
              </a>
              <a className="button button--ghost" href="#indicateurs">
                {t('hero.cta.indicators')}
              </a>
            </div>
            <p className="hero__note">{t('hero.note')}</p>
          </div>

          <PhoneMockup parcelle={parcelle} digest={digest} current={forecast?.current ?? null} />
        </section>

        <TodaySection
          parcelle={parcelle}
          digest={digest}
          current={forecast?.current ?? null}
          loading={loading}
          error={error}
          onSelect={setParcelle}
          onRetry={reload}
          timeZone={forecast?.timezone ?? 'Europe/Paris'}
        />

        <Features />

        <section className="data" id="donnees">
          <div className="section-head">
            <h2>{t('data.title')}</h2>
          </div>
          <div className="data__grid">
            <article>
              <h3>{t('data.model.title')}</h3>
              <p>{t('data.model.body')}</p>
            </article>
            <article>
              <h3>{t('data.rules.title')}</h3>
              <p>{t('data.rules.body')}</p>
            </article>
            <article>
              <h3>{t('data.wind.title')}</h3>
              <p>{t('data.wind.body', { limit: f.unit(AgroThresholds.sprayWindMax, 'km/h', 0) })}</p>
            </article>
          </div>
        </section>
      </main>

      <footer className="footer">
        <p>
          {credit.before}
          <a href="https://open-meteo.com/">Open-Meteo</a>
          {credit.after}
        </p>
        <p className="footer__note">{t('footer.note')}</p>
      </footer>
    </>
  )
}
