import { useState } from 'react'
import type { CurrentSample, HourlySample } from '@klima/core'
import { weatherCondition } from '@klima/core'
import { useI18n, WeatherIcon } from '@klima/core/ui'

interface Props {
  hours: readonly HourlySample[]
  /** Conditions observées, affichées sur la colonne « Maintenant ». */
  current: CurrentSample
  timeZone: string
}

/**
 * Ce qu'on montre sans rien demander : deux rangées sur un téléphone.
 *
 * Les vingt-quatre heures repliées en grille tenaient dans la carte, mais la
 * carte tenait tout l'écran. Une demi-journée répond à la question qu'on se
 * pose en ouvrant l'application ; le reste se déplie.
 */
const APERCU = 12

/** Bandeau horaire sur 24 h : heure, temps, probabilité de pluie, température. */
export function HourlyStrip({ hours, current, timeZone }: Props) {
  const { t, f, locale } = useI18n()
  const [deplie, setDeplie] = useState(false)
  const hourFormat = new Intl.DateTimeFormat(locale, { hour: 'numeric', timeZone })
  const slice = hours.slice(0, deplie ? 24 : APERCU)

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

      {hours.length > APERCU && (
        <button
          type="button"
          className="strip__toggle"
          aria-expanded={deplie}
          onClick={() => setDeplie((ouvert) => !ouvert)}
        >
          {deplie ? t('hourly.fold') : t('hourly.unfold')}
        </button>
      )}
    </section>
  )
}
