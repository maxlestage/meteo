import { DailyForecast } from './components/DailyForecast'
import { ParcelleSearch } from './components/ParcelleSearch'
import { SprayTimeline } from './components/SprayTimeline'
import { SummaryGrid } from './components/SummaryGrid'
import { useAgroForecast } from './hooks/useAgroForecast'
import { useParcelle } from './hooks/useParcelle'

const updatedFormat = new Intl.DateTimeFormat('fr-FR', { hour: '2-digit', minute: '2-digit' })

export default function App() {
  const [parcelle, setParcelle] = useParcelle()
  const { forecast, summary, loading, error, reload } = useAgroForecast(parcelle)

  return (
    <div className="app">
      <header className="app__header">
        <div>
          <p className="app__eyebrow">Météo agricole</p>
          <h1 className="app__title">{parcelle.name}</h1>
          <p className="app__subtitle">
            {[parcelle.admin, parcelle.country].filter(Boolean).join(', ') ||
              `${parcelle.latitude.toFixed(3)}, ${parcelle.longitude.toFixed(3)}`}
            {forecast && ` · ${Math.round(forecast.elevation)} m`}
          </p>
        </div>
        <ParcelleSearch current={parcelle} onSelect={setParcelle} />
      </header>

      {loading && <p className="app__state">Chargement des données agronomiques…</p>}

      {error && (
        <div className="app__error" role="alert">
          <p>{error}</p>
          <button type="button" className="button" onClick={reload}>
            Réessayer
          </button>
        </div>
      )}

      {forecast && summary && (
        <main className="app__main">
          <SummaryGrid summary={summary} />
          <SprayTimeline hours={forecast.hourly} />
          <DailyForecast days={forecast.daily} />
          <footer className="app__footer">
            <p>
              Données Open-Meteo (modèle agricole : humidité et température du sol, ET0 FAO-56, VPD) ·
              mise à jour {updatedFormat.format(forecast.fetchedAt)}
            </p>
            <button type="button" className="button" onClick={reload}>
              Actualiser
            </button>
          </footer>
        </main>
      )}
    </div>
  )
}
