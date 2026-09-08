import type { CurrentSample, HourlySample } from '../domain/agro'
import { weatherCondition } from '../domain/weather'
import { WeatherIcon } from './WeatherIcon'

interface Props {
  hours: readonly HourlySample[]
  /** Conditions observées, affichées sur la colonne « Maintenant ». */
  current: CurrentSample
  timeZone: string
}

/** Bandeau horaire sur 24 h : heure, temps, probabilité de pluie, température. */
export function HourlyStrip({ hours, current, timeZone }: Props) {
  const hourFormat = new Intl.DateTimeFormat('fr-FR', { hour: 'numeric', timeZone })
  const slice = hours.slice(0, 24)

  return (
    <section className="card" aria-label="Prévision horaire">
      <h2 className="card__label">Conditions météo</h2>
      <div className="strip">
        {slice.map((hour, index) => {
          // La première colonne montre le relevé courant, pas la prévision de
          // l'heure déjà entamée.
          const observed = index === 0
          const condition = weatherCondition(observed ? current.weatherCode : hour.weatherCode)
          return (
            <div className="strip__item" key={hour.time.toISOString()}>
              <span className="strip__hour">
                {observed ? 'Maint.' : hourFormat.format(hour.time)}
              </span>
              <WeatherIcon
                icon={condition.icon}
                isDay={observed ? current.isDay : hour.isDay}
                title={condition.label}
              />
              <span className="strip__rain">
                {hour.precipitationProbability >= 10
                  ? `${Math.round(hour.precipitationProbability)} %`
                  : '\u00a0'}
              </span>
              <span className="strip__temp">
                {Math.round(observed ? current.temperature : hour.temperature)}°
              </span>
            </div>
          )
        })}
      </div>
    </section>
  )
}
