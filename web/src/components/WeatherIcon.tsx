import type { ConditionIcon } from '../domain/weather'

interface Props {
  icon: ConditionIcon
  /** Décline le pictogramme en version nuit (lune au lieu du soleil). */
  isDay?: boolean
  size?: number
  title?: string
}

const CLOUD = 'M7.1 18.4h9.8a4.1 4.1 0 0 0 .4-8.2 5.7 5.7 0 0 0-10.9-1.2 4.5 4.5 0 0 0 .7 9.4Z'
const MOON = 'M17.6 14.9A6.6 6.6 0 0 1 9.1 6.4a6.6 6.6 0 1 0 8.5 8.5Z'

/** Pictogramme météo, dans l'esprit des symboles du système. */
export function WeatherIcon({ icon, isDay = true, size = 28, title }: Props) {
  return (
    <svg
      className="wicon"
      viewBox="0 0 24 24"
      width={size}
      height={size}
      role={title ? 'img' : 'presentation'}
      aria-label={title}
      aria-hidden={title ? undefined : true}
    >
      {shapes(icon, isDay)}
    </svg>
  )
}

function shapes(icon: ConditionIcon, isDay: boolean) {
  switch (icon) {
    case 'clear':
      return isDay ? sun(12, 12, 4.2) : <path d={MOON} className="wicon__moon" />
    case 'partly':
      return (
        <>
          {isDay ? sun(8.5, 8, 3.2) : <path d={MOON} className="wicon__moon" transform="translate(-3.5 -3.5) scale(0.72) translate(6 6)" />}
          <path d={CLOUD} className="wicon__cloud" />
        </>
      )
    case 'cloudy':
      return <path d={CLOUD} className="wicon__cloud" />
    case 'fog':
      return (
        <>
          <path d={CLOUD} className="wicon__cloud" />
          {line(5.5, 20.6, 18.5, 20.6, 'wicon__fog')}
          {line(8, 22.8, 16, 22.8, 'wicon__fog')}
        </>
      )
    case 'drizzle':
      return (
        <>
          <path d={CLOUD} className="wicon__cloud" />
          {drop(9.5)}
          {drop(14.5)}
        </>
      )
    case 'rain':
      return (
        <>
          <path d={CLOUD} className="wicon__cloud" />
          {drop(7.5)}
          {drop(12)}
          {drop(16.5)}
        </>
      )
    case 'showers':
      return (
        <>
          {isDay && sun(8.5, 8, 3.2)}
          <path d={CLOUD} className="wicon__cloud" />
          {drop(9.5)}
          {drop(15)}
        </>
      )
    case 'snow':
      return (
        <>
          <path d={CLOUD} className="wicon__cloud" />
          {flake(9)}
          {flake(15)}
        </>
      )
    case 'thunder':
      return (
        <>
          <path d={CLOUD} className="wicon__cloud" />
          <path d="M12.9 19.2h2.6l-4.4 4.6 1-3.2H9.6l3.9-4.3Z" className="wicon__bolt" />
        </>
      )
  }
}

function sun(cx: number, cy: number, r: number) {
  const rays = Array.from({ length: 8 }, (_, i) => {
    const angle = (i * Math.PI) / 4
    const inner = r + 1.4
    const outer = r + 3.2
    return (
      <line
        key={i}
        x1={cx + Math.cos(angle) * inner}
        y1={cy + Math.sin(angle) * inner}
        x2={cx + Math.cos(angle) * outer}
        y2={cy + Math.sin(angle) * outer}
        className="wicon__ray"
      />
    )
  })
  return (
    <>
      <circle cx={cx} cy={cy} r={r} className="wicon__sun" />
      {rays}
    </>
  )
}

function drop(x: number) {
  return <line x1={x} y1={20} x2={x - 1.1} y2={23.2} className="wicon__drop" />
}

function flake(x: number) {
  return <circle cx={x} cy={21.6} r={1.15} className="wicon__flake" />
}

function line(x1: number, y1: number, x2: number, y2: number, className: string) {
  return <line x1={x1} y1={y1} x2={x2} y2={y2} className={className} />
}
