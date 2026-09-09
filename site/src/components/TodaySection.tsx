import {
  percent,
  signedWithUnit,
  weatherCondition,
  withUnit,
  type CurrentSample,
  type DayDigest,
  type Parcelle,
} from '@klima/core'
import { CommuneSearch } from './CommuneSearch'
import { WeatherIcon } from '@klima/core/ui'

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
  return (
    <section className="today" id="aujourdhui">
      <div className="section-head">
        <h2>La météo du jour</h2>
        <p>
          Essayez sur votre commune. Le site s'en tient à aujourd'hui ; la semaine et le détail
          heure par heure sont dans l'application.
        </p>
      </div>

      <CommuneSearch current={parcelle} onSelect={onSelect} />

      <div className="today__card">
        {loading && <p className="today__state">Chargement de la journée…</p>}

        {error && (
          <div className="today__state" role="alert">
            <p>{error}</p>
            <button type="button" className="button button--ghost" onClick={onRetry}>
              Réessayer
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
  const condition = weatherCondition(current.weatherCode)
  const time = (date: Date | null) =>
    date
      ? new Intl.DateTimeFormat('fr-FR', { hour: '2-digit', minute: '2-digit', timeZone }).format(date)
      : '—'
  const range = (start: Date, end: Date) =>
    `${new Intl.DateTimeFormat('fr-FR', { hour: 'numeric', timeZone }).format(start)} → ${new Intl.DateTimeFormat('fr-FR', { hour: 'numeric', timeZone }).format(end)}`

  return (
    <>
      <div className="today__hero">
        <div>
          <p className="today__place">{parcelle.name}</p>
          <p className="today__date">
            {new Intl.DateTimeFormat('fr-FR', {
              weekday: 'long',
              day: 'numeric',
              month: 'long',
              timeZone,
            }).format(digest.date)}
          </p>
        </div>

        <div className="today__now">
          <WeatherIcon icon={condition.icon} size={54} title={condition.label} />
          <p className="today__temperature">{Math.round(current.temperature)}°</p>
          <div>
            <p className="today__condition">{condition.label}</p>
            <p className="today__range">
              ↑ {Math.round(digest.temperatureMax)}° &nbsp; ↓ {Math.round(digest.temperatureMin)}°
            </p>
          </div>
        </div>
      </div>

      <dl className="today__facts">
        <Fact
          label="Pluie"
          value={withUnit(digest.precipitationSum, 'mm')}
          detail={`${percent(digest.precipitationProbabilityMax)} de probabilité`}
        />
        <Fact
          label="Rafales"
          value={withUnit(digest.windGustsMax, 'km/h', 0)}
          detail="Maximum de la journée"
        />
        <Fact label="Lever" value={time(digest.sunrise)} detail={`Coucher à ${time(digest.sunset)}`} />
        <Fact
          label="ET0"
          value={withUnit(digest.et0Sum, 'mm')}
          detail="Évapotranspiration de référence"
        />
      </dl>

      <div className="today__agro">
        <Advice
          label="Fenêtre de traitement"
          value={digest.spray ? range(digest.spray.start, digest.spray.end) : 'Aucune d’ici ce soir'}
          detail={
            digest.spray
              ? `Score ${digest.spray.score}/100 sur la plage`
              : 'Vent, pluie ou température hors des clous'
          }
          tone={digest.spray ? (digest.spray.score >= 80 ? 'good' : 'warn') : 'bad'}
        />
        <Advice
          label="Bilan du jour"
          value={signedWithUnit(digest.balance, 'mm')}
          detail={
            digest.balance < 0
              ? 'La parcelle puise dans sa réserve'
              : 'La pluie couvre l’évapotranspiration'
          }
          tone={digest.balance < 0 ? 'warn' : 'good'}
        />
        <Advice
          label="État du sol"
          value={soilLabel(digest.soil.state)}
          detail={`${percent(digest.soil.moisture * 100)} vol. · ${
            digest.soil.trafficable ? 'portance correcte' : 'risque de tassement'
          }`}
          tone={digest.soil.state === 'sature' ? 'bad' : digest.soil.state === 'sec' ? 'warn' : 'good'}
        />
        <Advice
          label="Gel cette nuit"
          value={frostLabel(digest.frost.severity)}
          detail={`Mini ${withUnit(digest.frost.minTemperature, '°C')}${
            digest.frost.hoarFrost ? ' · gelée blanche probable' : ''
          }`}
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

function soilLabel(state: 'sature' | 'ressuye' | 'sec'): string {
  return { sature: 'Saturé', ressuye: 'Ressuyé', sec: 'Sec' }[state]
}

function frostLabel(severity: 'aucun' | 'faible' | 'modere' | 'severe'): string {
  return { aucun: 'Aucun', faible: 'Faible', modere: 'Modéré', severe: 'Sévère' }[severity]
}
