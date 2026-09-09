import { describeBlocker, sprayWindows, type HourlySample, type SprayOpportunity } from '@klima/core'
import { useI18n } from '@klima/core/ui'

interface Props {
  hours: readonly HourlySample[]
  nextSpray: SprayOpportunity | null
  timeZone: string
}

/**
 * Carte pleine largeur des conditions de pulvérisation : la prochaine fenêtre
 * en clair, puis une frise des 24 prochaines heures.
 */
export function SprayCard({ hours, nextSpray, timeZone }: Props) {
  const { t, f, locale } = useI18n()
  const windows = sprayWindows(hours).slice(0, 24)
  const rangeFormat = new Intl.DateTimeFormat(locale, {
    weekday: 'short',
    hour: 'numeric',
    timeZone,
  })
  const endFormat = new Intl.DateTimeFormat(locale, { hour: 'numeric', timeZone })
  const hourFormat = new Intl.DateTimeFormat(locale, { hour: 'numeric', timeZone })

  // Le premier motif de blocage des 24 h résume la situation.
  const blocked = windows.find((w) => w.verdict === 'defavorable' && w.blockers.length > 0)
  const mainBlocker = blocked ? describeBlocker(blocked.blockers[0]!, t, f) : null

  return (
    <section className="card" aria-label={t('spray.title')}>
      <h2 className="card__label">{t('spray.title')}</h2>

      <p className="spray__headline">
        {nextSpray
          ? `${capitalize(rangeFormat.format(nextSpray.start))} → ${endFormat.format(nextSpray.end)}`
          : t('spray.none')}
      </p>
      <p className="spray__caption">
        {nextSpray
          ? t('spray.score', { score: nextSpray.score })
          : mainBlocker
            ? t('spray.mainBlocker', { blocker: mainBlocker.toLowerCase() })
            : t('spray.unsuitable')}
      </p>

      <div className="spray__strip">
        {windows.map((window) => (
          <span
            key={window.time.toISOString()}
            className={`spray__hour spray__hour--${window.verdict}`}
            title={`${hourFormat.format(window.time)} — ${window.score}/100${
              window.blockers.length
                ? ` · ${window.blockers.map((blocker) => describeBlocker(blocker, t, f)).join(' · ')}`
                : ''
            }`}
          />
        ))}
      </div>
      <div className="spray__scale">
        <span>{t('spray.now')}</span>
        <span>{t('spray.plus12')}</span>
        <span>{t('spray.plus24')}</span>
      </div>
    </section>
  )
}

function capitalize(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1)
}

