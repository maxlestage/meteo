import type { AgroSummary } from '../domain/agro'

interface Props {
  summary: AgroSummary
}

type Tone = 'good' | 'warn' | 'bad' | 'neutral'

/** Les cinq indicateurs agronomiques du tableau de bord. */
export function SummaryGrid({ summary }: Props) {
  const { water, soil, disease, frost, gdd, nextSpray } = summary

  return (
    <section className="grid" aria-label="Indicateurs agronomiques">
      <Card
        title="Bilan hydrique 7 j"
        value={`${water.balance > 0 ? '+' : ''}${water.balance} mm`}
        tone={water.status === 'deficit' ? 'bad' : water.status === 'excedent' ? 'warn' : 'good'}
        caption={
          water.irrigationAdvice > 0
            ? `Irrigation conseillée : ${Math.round(water.irrigationAdvice)} mm`
            : `Pluie ${water.precipitation} mm · ET0 ${water.evapotranspiration} mm`
        }
      />

      <Card
        title="État du sol"
        value={soilLabel(soil.state)}
        tone={soil.state === 'sature' ? 'bad' : soil.state === 'sec' ? 'warn' : 'good'}
        caption={`${(soil.moisture * 100).toFixed(1)} % vol. · ${soil.temperature} °C à 6 cm`}
        footer={[
          soil.trafficable ? 'Portance correcte' : 'Risque de tassement',
          soil.sowable ? 'Semis possible' : 'Semis déconseillé',
        ]}
      />

      <Card
        title="Fenêtre de traitement"
        value={nextSpray ? formatRange(nextSpray.start, nextSpray.end) : 'Aucune'}
        tone={nextSpray ? (nextSpray.score >= 80 ? 'good' : 'warn') : 'bad'}
        caption={
          nextSpray ? `Score ${nextSpray.score}/100 sur la plage` : 'Rien d’exploitable sur 7 jours'
        }
      />

      <Card
        title="Pression maladie"
        value={diseaseLabel(disease.level)}
        tone={disease.level === 'elevee' ? 'bad' : disease.level === 'moyenne' ? 'warn' : 'good'}
        caption={`${disease.leafWetnessHours} h d’humectation du feuillage sur 24 h`}
      />

      <Card
        title="Risque de gel"
        value={frost.severity === 'aucun' ? 'Aucun' : frostLabel(frost.severity)}
        tone={frost.severity === 'aucun' ? 'good' : frost.severity === 'faible' ? 'warn' : 'bad'}
        caption={`Mini ${frost.minTemperature} °C${frost.hoarFrost ? ' · gelée blanche probable' : ''}`}
      />

      <Card
        title="Degrés-jours (base 10)"
        value={`${gdd} °C·j`}
        tone="neutral"
        caption="Cumul sur les 7 jours de prévision"
      />
    </section>
  )
}

function Card({
  title,
  value,
  caption,
  tone,
  footer,
}: {
  title: string
  value: string
  caption: string
  tone: Tone
  footer?: string[]
}) {
  return (
    <article className={`card card--${tone}`}>
      <h2 className="card__title">{title}</h2>
      <p className="card__value">{value}</p>
      <p className="card__caption">{caption}</p>
      {footer && (
        <ul className="card__footer">
          {footer.map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
      )}
    </article>
  )
}

const hourFormat = new Intl.DateTimeFormat('fr-FR', { weekday: 'short', hour: '2-digit' })

function formatRange(start: Date, end: Date): string {
  return `${hourFormat.format(start)} → ${end.getHours()} h`
}

function soilLabel(state: 'sature' | 'ressuye' | 'sec'): string {
  return { sature: 'Saturé', ressuye: 'Ressuyé', sec: 'Sec' }[state]
}

function diseaseLabel(level: 'faible' | 'moyenne' | 'elevee'): string {
  return { faible: 'Faible', moyenne: 'Moyenne', elevee: 'Élevée' }[level]
}

function frostLabel(severity: 'faible' | 'modere' | 'severe'): string {
  return { faible: 'Faible', modere: 'Modéré', severe: 'Sévère' }[severity]
}
