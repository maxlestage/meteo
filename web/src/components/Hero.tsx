import type { AgroForecast } from '../api/openMeteo'
import { weatherCondition } from '../domain/weather'

interface Props {
  forecast: AgroForecast
}

/** En-tête : commune, température, temps et amplitude du jour. */
export function Hero({ forecast }: Props) {
  const { current, parcelle, daily } = forecast
  const today = daily[0]
  const condition = weatherCondition(current.weatherCode)

  return (
    <header className="hero">
      <h1 className="hero__place">{parcelle.name}</h1>
      <p className="hero__temperature">{Math.round(current.temperature)}°</p>
      <p className="hero__condition">{condition.label}</p>
      {today && (
        <p className="hero__range">
          ↑ {Math.round(today.temperatureMax)}° &nbsp; ↓ {Math.round(today.temperatureMin)}°
        </p>
      )}
    </header>
  )
}
