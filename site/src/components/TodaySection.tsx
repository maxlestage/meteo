import { weatherCondition, type CurrentSample, type DayDigest, type Parcelle } from '@klima/core'
import { useI18n, WeatherIcon } from '@klima/core/ui'
import { CommuneSearch } from './CommuneSearch'

interface Props {
  parcelle: Parcelle
  digest: DayDigest | null
  /** Relevé du moment : c'est lui qui donne la grande température. */
  current: CurrentSample | null
  loading: boolean
  error: string | null
  onSelect: (parcelle: Parcelle) => void
  onRetry: () => void
  timeZone: string
}

/**
 * La météo du jour, et rien d'autre : ce que Klima dit de la journée en cours
 * sur la parcelle choisie.
 */
export function TodaySection({
  parcelle,
  digest,
  current,
  loading,
  error,
  onSelect,
  onRetry,
  timeZone,
}: Props) {
  const { t } = useI18n()

  return (
    <section className="today" id="aujourdhui">
      <div className="section-head">
        <h2>{t('today.title')}</h2>
        <p>{t('today.lead')}</p>
      </div>

      <CommuneSearch current={parcelle} onSelect={onSelect} />

      <div className="today__card">
        {loading && <p className="today__state">{t('today.loading')}</p>}

        {error && (
          <div className="today__state" role="alert">
            <p>{error}</p>
            <button type="button" className="button button--ghost" onClick={onRetry}>
              {t('today.retry')}
            </button>
          </div>
        )}

        {digest && current && !loading && !error && (
          <Content parcelle={parcelle} digest={digest} current={current} timeZone={timeZone} />
        )}
      </div>
    </section>
  )
}

function Content({
  parcelle,
  digest,
  current,
  timeZone,
}: {
  parcelle: Parcelle
  digest: DayDigest
  current: CurrentSample
  timeZone: string
}) {
  const { t, f, locale } = useI18n()
  const condition = weatherCondition(current.weatherCode)

  const time = (date: Date | null) =>
    date
      ? new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit', timeZone }).format(date)
      : '—'
  const hour = (date: Date) =>
    new Intl.DateTimeFormat(locale, { hour: 'numeric', timeZone }).format(date)

  return (
    <>
      <div className="today__hero">
        <div>
          <p className="today__place">{parcelle.name}</p>
          <p className="today__date">
            {new Intl.DateTimeFormat(locale, {
              weekday: 'long',
              day: 'numeric',
              month: 'long',
              timeZone,
            }).format(digest.date)}
          </p>
        </div>

        <div className="today__now">
          <WeatherIcon icon={condition.icon} size={54} title={t(condition.labelKey)} />
          <p className="today__temperature">{f.temperature(current.temperature)}</p>
          <div>
            <p className="today__condition">{t(condition.labelKey)}</p>
            <p className="today__range">
              ↑ {f.temperature(digest.temperatureMax)} &nbsp; ↓ {f.temperature(digest.temperatureMin)}
            </p>
          </div>
        </div>
      </div>

      <dl className="today__facts">
        <Fact
          label={t('today.rain')}
          value={f.unit(digest.precipitationSum, 'mm')}
          detail={t('today.rain.detail', { probability: f.percent(digest.precipitationProbabilityMax) })}
        />
        <Fact
          label={t('today.gusts')}
          value={f.unit(digest.windGustsMax, 'km/h', 0)}
          detail={t('today.gusts.detail')}
        />
        <Fact
          label={t('today.sunrise')}
          value={time(digest.sunrise)}
          detail={t('today.sunrise.detail', { time: time(digest.sunset) })}
        />
        <Fact
          label={t('today.et0')}
          value={f.unit(digest.et0Sum, 'mm')}
          detail={t('today.et0.detail')}
        />
      </dl>

      <div className="today__agro">
        <Advice
          label={t('today.spray')}
          value={
            digest.spray
              ? `${hour(digest.spray.start)} → ${hour(digest.spray.end)}`
              : t('today.spray.none')
          }
          detail={
            digest.spray
              ? t('today.spray.score', { score: digest.spray.score })
              : t('today.spray.blocked')
          }
          tone={digest.spray ? (digest.spray.score >= 80 ? 'good' : 'warn') : 'bad'}
        />
        <Advice
          label={t('today.balance')}
          value={f.signedUnit(digest.balance, 'mm')}
          detail={t(digest.balance < 0 ? 'today.balance.deficit' : 'today.balance.ok')}
          tone={digest.balance < 0 ? 'warn' : 'good'}
        />
        <Advice
          label={t('today.soil')}
          value={t(`soil.${digest.soil.state}`)}
          detail={t('today.soil.detail', {
            moisture: f.percent(digest.soil.moisture * 100),
            state: t(digest.soil.trafficable ? 'soil.trafficable' : 'soil.compaction').toLowerCase(),
          })}
          tone={digest.soil.state === 'sature' ? 'bad' : digest.soil.state === 'sec' ? 'warn' : 'good'}
        />
        <Advice
          label={t('today.frost')}
          value={t(`frost.${digest.frost.severity}`)}
          detail={t('today.frost.detail', {
            temperature: f.unit(digest.frost.minTemperature, '°C'),
            hoarFrost: digest.frost.hoarFrost ? ` · ${t('frost.hoarFrost')}` : '',
          })}
          tone={
            digest.frost.severity === 'aucun'
              ? 'good'
              : digest.frost.severity === 'faible'
                ? 'warn'
                : 'bad'
          }
        />
      </div>
    </>
  )
}

function Fact({ label, value, detail }: { label: string; value: string; detail: string }) {
  return (
    <div className="fact">
      <dt>{label}</dt>
      <dd>
        <span className="fact__value">{value}</span>
        <span className="fact__detail">{detail}</span>
      </dd>
    </div>
  )
}

function Advice({
  label,
  value,
  detail,
  tone,
}: {
  label: string
  value: string
  detail: string
  tone: 'good' | 'warn' | 'bad'
}) {
  return (
    <article className={`advice advice--${tone}`}>
      <h3>{label}</h3>
      <p className="advice__value">{value}</p>
      <p className="advice__detail">{detail}</p>
    </article>
  )
}
