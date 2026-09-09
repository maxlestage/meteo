import type { CurrentSample, HourlySample } from '@klima/core'
import { weatherCondition } from '@klima/core'
import { useI18n, WeatherIcon } from '@klima/core/ui'

interface Props {
  hours: readonly HourlySample[]
  /** Conditions observées, affichées sur la colonne « Maintenant ». */
  current: CurrentSample
  timeZone: string
}

/** Bandeau horaire sur 24 h : heure, temps, probabilité de pluie, température. */
export function HourlyStrip({ hours, current, timeZone }: Props) {
  const { t, f, locale } = useI18n()
  const hourFormat = new Intl.DateTimeFormat(locale, { hour: 'numeric', timeZone })
  const slice = hours.slice(0, 24)

  return (
    <section className="card" aria-label={t('hourly.title')}>
      <h2 className="card__label">{t('hourly.title')}</h2>
      <div className="strip">
        {slice.map((hour, index) => {
          // La première colonne montre le relevé courant, pas la prévision de
          // l'heure déjà entamée.
          const observed = index === 0
          const condition = weatherCondition(observed ? current.weatherCode : hour.weatherCode)
          return (
            <div className="strip__item" key={hour.time.toISOString()}>
              <span className="strip__hour">
                {observed ? t('hourly.now') : hourFormat.format(hour.time)}
              </span>
              <WeatherIcon
                icon={condition.icon}
                isDay={observed ? current.isDay : hour.isDay}
                title={t(condition.labelKey)}
              />
              <span className="strip__rain">
                {hour.precipitationProbability >= 10 ? f.percent(hour.precipitationProbability) : '\u00a0'}
              </span>
              <span className="strip__temp">
                {f.temperature(observed ? current.temperature : hour.temperature)}
              </span>
            </div>
          )
        })}
      </div>
    </section>
  )
}
