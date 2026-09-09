import { signedWithUnit, weatherCondition, type CurrentSample, type DayDigest, type Parcelle } from '@klima/core'
import { WeatherIcon } from '@klima/core/ui'

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
          {current ? `${Math.round(current.temperature)}°` : '—'}
        </p>
        <p className="phone__condition">{condition?.label ?? 'Chargement'}</p>
        <p className="phone__range">
          {digest
            ? `↑ ${Math.round(digest.temperatureMax)}°   ↓ ${Math.round(digest.temperatureMin)}°`
            : ''}
        </p>

        <div className="phone__card">
          <p className="phone__label">Conditions météo</p>
          <div className="phone__hours">
            {(digest?.remainingHours ?? []).slice(0, 5).map((hour) => (
              <div className="phone__hour" key={hour.time.toISOString()}>
                <span>{hour.time.getHours()}</span>
                <WeatherIcon
                  icon={weatherCondition(hour.weatherCode).icon}
                  isDay={hour.isDay}
                  size={18}
                />
                <span>{Math.round(hour.temperature)}°</span>
              </div>
            ))}
          </div>
        </div>

        <div className="phone__tiles">
          <div className="phone__tile">
            <p className="phone__label">Bilan hydrique</p>
            <p className="phone__value">
              {digest ? signedWithUnit(digest.balance, 'mm') : '—'}
            </p>
          </div>
          <div className="phone__tile">
            <p className="phone__label">Traitement</p>
            <p className="phone__value">{digest?.spray ? 'Possible' : 'Non'}</p>
          </div>
        </div>
      </div>
    </div>
  )
}
