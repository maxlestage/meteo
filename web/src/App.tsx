import { DailyList } from './components/DailyList'
import { DetailTile } from './components/DetailTile'
import { Hero } from './components/Hero'
import { HourlyStrip } from './components/HourlyStrip'
import { ParcelleSearch } from './components/ParcelleSearch'
import { SprayCard } from './components/SprayCard'
import { AgroThresholds } from './domain/agro'
import { useAgroForecast } from './hooks/useAgroForecast'
import { useParcelle } from './hooks/useParcelle'

export default function App() {
  const [parcelle, setParcelle] = useParcelle()
  const { forecast, summary, loading, error, reload } = useAgroForecast(parcelle)

  const sky = forecast ? skyFor(forecast.current.isDay, forecast.current.weatherCode) : 'night'

  return (
    <div className={`sky sky--${sky}`}>
      <div className="shell">
        <ParcelleSearch current={parcelle} onSelect={setParcelle} />

        {loading && !forecast && <p className="state">Chargement…</p>}

        {error && (
          <div className="state state--error" role="alert">
            <p>{error}</p>
            <button type="button" className="button" onClick={reload}>
              Réessayer
            </button>
          </div>
        )}

        {forecast && summary && (
          <>
            <Hero forecast={forecast} />
            <HourlyStrip
              hours={forecast.hourly}
              current={forecast.current}
              timeZone={forecast.timezone}
            />
            <DailyList
              days={forecast.daily}
              currentTemperature={forecast.current.temperature}
              timeZone={forecast.timezone}
            />
            <SprayCard
              hours={forecast.hourly}
              nextSpray={summary.nextSpray}
              timeZone={forecast.timezone}
            />

            <div className="tiles">
              <DetailTile
                label="Humidité du sol"
                value={summary.soil.state === 'ressuye' ? 'Ressuyé' : summary.soil.state === 'sature' ? 'Saturé' : 'Sec'}
                caption={`${(summary.soil.moisture * 100).toFixed(0)} % vol. · ${summary.soil.temperature} °C à 6 cm. ${
                  summary.soil.trafficable ? 'Le sol porte les engins.' : 'Risque de tassement.'
                }`}
                gauge={{
                  position: summary.soil.moisture / 0.5,
                  gradient:
                    'linear-gradient(to right, #d8b36a 0%, #8fc46a 30%, #4aa3d8 70%, #2b5f9e 100%)',
                }}
              />

              <DetailTile
                label="Bilan hydrique"
                value={`${summary.water.balance > 0 ? '+' : ''}${summary.water.balance} mm`}
                caption={
                  summary.water.irrigationAdvice > 0
                    ? `Irrigation conseillée : ${Math.round(summary.water.irrigationAdvice)} mm sur 7 jours.`
                    : `Pluie ${summary.water.precipitation} mm, ET0 ${summary.water.evapotranspiration} mm sur 7 jours.`
                }
              />

              <DetailTile
                label="Vent"
                value={`${Math.round(forecast.current.windSpeed)} km/h`}
                caption={`Rafales ${Math.round(forecast.current.windGusts)} km/h. Limite de pulvérisation : ${AgroThresholds.sprayWindMax} km/h.`}
              />

              <DetailTile
                label="Risque de gel"
                value={frostLabel(summary.frost.severity)}
                caption={`Mini ${summary.frost.minTemperature} °C cette nuit${
                  summary.frost.hoarFrost ? ', gelée blanche probable' : ''
                }.`}
              />

              <DetailTile
                label="Pression maladie"
                value={diseaseLabel(summary.disease.level)}
                caption={`${summary.disease.leafWetnessHours} h d’humectation du feuillage sur 24 h.`}
              />

              <DetailTile
                label="Degrés-jours"
                value={`${summary.gdd} °C·j`}
                caption={`Cumul sur 7 jours, base ${AgroThresholds.gddBase} °C.`}
              />

              <DetailTile
                label="Lever"
                value={time(forecast.daily[0]?.sunrise, forecast.timezone)}
                caption={`Coucher à ${time(forecast.daily[0]?.sunset, forecast.timezone)}.`}
              />

              <DetailTile
                label="Semis"
                value={summary.soil.sowable ? 'Possible' : 'Déconseillé'}
                caption={`Sol à ${summary.soil.temperature} °C à 6 cm ; il faut 8 °C et un sol ressuyé.`}
              />
            </div>

            <footer className="footer">
              <p>
                Données Open-Meteo — modèle agricole : humidité et température du sol, ET0 FAO-56,
                déficit de pression de vapeur.
              </p>
              <button type="button" className="button" onClick={reload}>
                Actualiser
              </button>
            </footer>
          </>
        )}
      </div>
    </div>
  )
}

/** Le fond suit le ciel : nuit, journée couverte ou journée dégagée. */
function skyFor(isDay: boolean, code: number): 'night' | 'grey' | 'day' {
  if (!isDay) return 'night'
  return code >= 45 ? 'grey' : 'day'
}

function frostLabel(severity: 'aucun' | 'faible' | 'modere' | 'severe'): string {
  return { aucun: 'Aucun', faible: 'Faible', modere: 'Modéré', severe: 'Sévère' }[severity]
}

function diseaseLabel(level: 'faible' | 'moyenne' | 'elevee'): string {
  return { faible: 'Faible', moyenne: 'Moyenne', elevee: 'Élevée' }[level]
}

function time(date: Date | null | undefined, timeZone: string): string {
  if (!date) return '—'
  return new Intl.DateTimeFormat('fr-FR', { hour: '2-digit', minute: '2-digit', timeZone }).format(date)
}
