import { weatherCondition, type CurrentSample, type DayDigest, type Parcelle } from '@klima/core'
import { useI18n, WeatherIcon } from '@klima/core/ui'

interface Props {
  parcelle: Parcelle
  digest: DayDigest | null
  current: CurrentSample | null
}

/**
 * Aperçu de l'application iOS, alimenté par la commune choisie plus haut :
 * ce que le visiteur voit ici, il le retrouve sur son téléphone.
 */
export function PhoneMockup({ parcelle, digest, current }: Props) {
  const { t, f } = useI18n()
  const condition = current ? weatherCondition(current.weatherCode) : null

  return (
    <div className="phone" aria-hidden="true">
      <div className="phone__screen">
        <div className="phone__status">
          <span>9:41</span>
          <span className="phone__signal" />
        </div>

        <p className="phone__place">{parcelle.name}</p>
        <p className="phone__temperature">
          {current ? f.temperature(current.temperature) : '—'}
        </p>
        <p className="phone__condition">
          {condition ? t(condition.labelKey) : t('phone.loading')}
        </p>
        <p className="phone__range">
          {digest
            ? `↑ ${f.temperature(digest.temperatureMax)}   ↓ ${f.temperature(digest.temperatureMin)}`
            : ''}
        </p>

        <div className="phone__card">
          <p className="phone__label">{t('phone.conditions')}</p>
          <div className="phone__hours">
            {(digest?.remainingHours ?? []).slice(0, 5).map((hour) => (
              <div className="phone__hour" key={hour.time.toISOString()}>
                <span>{hour.time.getHours()}</span>
                <WeatherIcon
                  icon={weatherCondition(hour.weatherCode).icon}
                  isDay={hour.isDay}
                  size={18}
                />
                <span>{f.temperature(hour.temperature)}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="phone__tiles">
          <div className="phone__tile">
            <p className="phone__label">{t('phone.balance')}</p>
            <p className="phone__value">
              {digest ? f.signedUnit(digest.balance, 'mm') : '—'}
            </p>
          </div>
          <div className="phone__tile">
            <p className="phone__label">{t('phone.spray')}</p>
            <p className="phone__value">{t(digest?.spray ? 'phone.spray.yes' : 'phone.spray.no')}</p>
          </div>
        </div>
      </div>
    </div>
  )
}
