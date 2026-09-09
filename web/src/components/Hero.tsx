import type { AgroForecast } from '@klima/core'
import { weatherCondition } from '@klima/core'
import { useI18n } from '@klima/core/ui'

interface Props {
  forecast: AgroForecast
}

/** En-tête : commune, température, temps et amplitude du jour. */
export function Hero({ forecast }: Props) {
  const { t, f } = useI18n()
  const { current, parcelle, daily } = forecast
  const today = daily[0]
  const condition = weatherCondition(current.weatherCode)

  return (
    <header className="hero">
      <h1 className="hero__place">{parcelle.name}</h1>
      <p className="hero__temperature">{f.temperature(current.temperature)}</p>
      <p className="hero__condition">{t(condition.labelKey)}</p>
      {today && (
        <p className="hero__range">
          ↑ {f.temperature(today.temperatureMax)} &nbsp; ↓ {f.temperature(today.temperatureMin)}
        </p>
      )}
    </header>
  )
}
