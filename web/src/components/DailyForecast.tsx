import { growingDegreeDays, type DailySample } from '../domain/agro'

interface Props {
  days: readonly DailySample[]
}

const dayFormat = new Intl.DateTimeFormat('fr-FR', { weekday: 'short', day: 'numeric', month: 'short' })

/** Tableau des cumuls journaliers, orienté conduite de culture. */
export function DailyForecast({ days }: Props) {
  return (
    <section className="panel" aria-label="Prévision journalière">
      <header className="panel__header">
        <h2>Semaine agronomique</h2>
      </header>

      <div className="table-scroll">
        <table className="table">
          <thead>
            <tr>
              <th scope="col">Jour</th>
              <th scope="col">Min / Max</th>
              <th scope="col">Pluie</th>
              <th scope="col">ET0</th>
              <th scope="col">Bilan</th>
              <th scope="col">Rafales</th>
              <th scope="col">°C·j</th>
            </tr>
          </thead>
          <tbody>
            {days.map((day) => {
              const balance = Math.round((day.precipitationSum - day.et0Sum) * 10) / 10
              return (
                <tr key={day.date.toISOString()}>
                  <th scope="row">{dayFormat.format(day.date)}</th>
                  <td>
                    {Math.round(day.temperatureMin)}° / {Math.round(day.temperatureMax)}°
                  </td>
                  <td>
                    {day.precipitationSum.toFixed(1)} mm
                    <span className="table__muted"> ({Math.round(day.precipitationProbabilityMax)} %)</span>
                  </td>
                  <td>{day.et0Sum.toFixed(1)} mm</td>
                  <td className={balance < 0 ? 'table__negative' : 'table__positive'}>
                    {balance > 0 ? '+' : ''}
                    {balance.toFixed(1)}
                  </td>
                  <td>{Math.round(day.windGustsMax)} km/h</td>
                  <td>{growingDegreeDays(day.temperatureMin, day.temperatureMax)}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </section>
  )
}
