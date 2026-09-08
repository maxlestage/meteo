import { useState } from 'react'
import { sprayWindows, type HourlySample, type SprayWindow } from '../domain/agro'

interface Props {
  hours: readonly HourlySample[]
}

const hourFormat = new Intl.DateTimeFormat('fr-FR', { hour: '2-digit' })
const dayFormat = new Intl.DateTimeFormat('fr-FR', { weekday: 'long', day: 'numeric', month: 'long' })

/** Frise horaire des conditions de pulvérisation sur les 48 prochaines heures. */
export function SprayTimeline({ hours }: Props) {
  const windows = sprayWindows(hours).slice(0, 48)
  const [selected, setSelected] = useState<SprayWindow | null>(null)
  const detail = selected ?? windows[0] ?? null

  if (!detail) return null

  return (
    <section className="panel" aria-label="Conditions de pulvérisation">
      <header className="panel__header">
        <h2>Conditions de pulvérisation — 48 h</h2>
        <p className="panel__legend">
          <span className="dot dot--good" /> favorable
          <span className="dot dot--warn" /> acceptable
          <span className="dot dot--bad" /> défavorable
        </p>
      </header>

      <ol className="timeline">
        {windows.map((window) => (
          <li key={window.time.toISOString()}>
            <button
              type="button"
              className={`timeline__bar timeline__bar--${window.verdict} ${
                detail.time.getTime() === window.time.getTime() ? 'is-selected' : ''
              }`}
              onClick={() => setSelected(window)}
              aria-label={`${dayFormat.format(window.time)} ${hourFormat.format(window.time)} : ${window.verdict}, score ${window.score}`}
            >
              <span
                className="timeline__track"
                style={{ ['--score' as string]: `${Math.max(window.score, 6)}%` }}
              />
              <span className="timeline__hour">{window.time.getHours()}</span>
            </button>
          </li>
        ))}
      </ol>

      <div className="timeline__detail">
        <p className="timeline__detail-title">
          {dayFormat.format(detail.time)} · {hourFormat.format(detail.time)} — score {detail.score}/100
        </p>
        {detail.blockers.length === 0 ? (
          <p className="timeline__ok">Toutes les conditions sont réunies pour traiter.</p>
        ) : (
          <ul className="timeline__blockers">
            {detail.blockers.map((blocker) => (
              <li key={blocker}>{blocker}</li>
            ))}
          </ul>
        )}
      </div>
    </section>
  )
}
