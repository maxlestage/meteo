import { sprayWindows, type HourlySample, type SprayOpportunity } from '../domain/agro'

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
  const windows = sprayWindows(hours).slice(0, 24)
  const rangeFormat = new Intl.DateTimeFormat('fr-FR', {
    weekday: 'short',
    hour: 'numeric',
    timeZone,
  })
  const endFormat = new Intl.DateTimeFormat('fr-FR', { hour: 'numeric', timeZone })
  const hourFormat = new Intl.DateTimeFormat('fr-FR', { hour: 'numeric', timeZone })

  // Le motif de blocage le plus fréquent des 24 h résume la situation.
  const blocked = windows.filter((w) => w.verdict === 'defavorable' && w.blockers.length > 0)
  const mainBlocker = blocked.length > 0 ? shorten(blocked[0]!.blockers[0]!) : null

  return (
    <section className="card" aria-label="Fenêtre de traitement">
      <h2 className="card__label">Fenêtre de traitement</h2>

      <p className="spray__headline">
        {nextSpray
          ? `${capitalize(rangeFormat.format(nextSpray.start))} → ${endFormat.format(nextSpray.end)}`
          : 'Aucune fenêtre sur 7 jours'}
      </p>
      <p className="spray__caption">
        {nextSpray
          ? `Score ${nextSpray.score}/100 sur la plage`
          : mainBlocker
            ? `Blocage principal : ${mainBlocker}`
            : 'Conditions défavorables'}
      </p>

      <div className="spray__strip">
        {windows.map((window) => (
          <span
            key={window.time.toISOString()}
            className={`spray__hour spray__hour--${window.verdict}`}
            title={`${hourFormat.format(window.time)} — ${window.score}/100${
              window.blockers.length ? ` · ${window.blockers.join(' · ')}` : ''
            }`}
          />
        ))}
      </div>
      <div className="spray__scale">
        <span>Maintenant</span>
        <span>+12 h</span>
        <span>+24 h</span>
      </div>
    </section>
  )
}

function capitalize(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1)
}

/** Retire la valeur chiffrée pour ne garder que le motif. */
function shorten(blocker: string): string {
  return blocker.toLowerCase().replace(/\s*\(.*\)$/, '')
}
