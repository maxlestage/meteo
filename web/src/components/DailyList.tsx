import type { DailySample } from '@klima/core'
import { weatherCondition } from '@klima/core'
import { useI18n, WeatherIcon } from '@klima/core/ui'

interface Props {
  days: readonly DailySample[]
  /** Température du moment, repérée sur la barre d'aujourd'hui. */
  currentTemperature: number
  timeZone: string
}

/** Liste des jours, avec la barre d'amplitude thermique de la semaine. */
export function DailyList({ days, currentTemperature, timeZone }: Props) {
  const { t, f, locale } = useI18n()
  const dayFormat = new Intl.DateTimeFormat(locale, { weekday: 'short', timeZone })

  // Toutes les barres se lisent sur la même échelle : celle de la semaine.
  const lows = days.map((d) => d.temperatureMin)
  const highs = days.map((d) => d.temperatureMax)
  const weekLow = Math.min(...lows)
  const weekHigh = Math.max(...highs)
  const span = Math.max(weekHigh - weekLow, 1)

  return (
    <section className="card" aria-label={t('daily.title')}>
      <h2 className="card__label">{t('daily.title')}</h2>
      <ul className="days">
        {days.map((day, index) => {
          const condition = weatherCondition(day.weatherCode)
          const left = ((day.temperatureMin - weekLow) / span) * 100
          const width = Math.max(((day.temperatureMax - day.temperatureMin) / span) * 100, 6)

          return (
            <li className="days__row" key={day.date.toISOString()}>
              <span className="days__name">
                {index === 0 ? t('daily.today') : capitalize(dayFormat.format(day.date))}
              </span>

              <span className="days__weather">
                <WeatherIcon icon={condition.icon} size={24} title={t(condition.labelKey)} />
                <span className="days__rain">
                  {day.precipitationProbabilityMax >= 10 ? f.percent(day.precipitationProbabilityMax) : ''}
                </span>
              </span>

              <span className="days__low">{f.temperature(day.temperatureMin)}</span>
              <span className="days__bar">
                <span className="days__fill" style={{ left: `${left}%`, width: `${width}%` }} />
                {index === 0 && (
                  <span
                    className="days__now"
                    style={{
                      left: `${clamp(((currentTemperature - weekLow) / span) * 100)}%`,
                    }}
                  />
                )}
              </span>
              <span className="days__high">{f.temperature(day.temperatureMax)}</span>
            </li>
          )
        })}
      </ul>
    </section>
  )
}

function clamp(percent: number): number {
  return Math.min(Math.max(percent, 0), 100)
}

function capitalize(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1)
}
