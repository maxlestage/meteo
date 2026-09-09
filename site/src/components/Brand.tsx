/**
 * Marque de Klima.
 *
 * Un disque de ciel, une goutte et une feuille qui n'en font qu'un : la météo
 * et l'agronomie tenues dans le même signe. Le trait est épais et les formes
 * peu nombreuses, pour rester lisible à seize pixels comme sur une icône
 * d'application.
 */

interface Props {
  size?: number
  /** Vrai pour la version claire, posée sur un fond sombre. */
  inverted?: boolean
  title?: string
}

export function BrandMark({ size = 32, inverted = false, title }: Props) {
  const leaf = inverted ? '#ffffff' : '#3f6b2b'
  const drop = inverted ? '#bfe3f7' : '#2f7fb5'

  return (
    <svg
      viewBox="0 0 64 64"
      width={size}
      height={size}
      role={title ? 'img' : 'presentation'}
      aria-label={title}
      aria-hidden={title ? undefined : true}
      className="brand-mark"
    >
      <defs>
        <linearGradient id="klimaSky" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={inverted ? '#4a6d8c' : '#8fc0e4'} />
          <stop offset="100%" stopColor={inverted ? '#22303f' : '#3f6b2b'} />
        </linearGradient>
      </defs>

      {/* Le ciel et la terre dans un même disque. */}
      <circle cx="32" cy="32" r="30" fill="url(#klimaSky)" />

      {/* La goutte, qui tombe du ciel dans la feuille. */}
      <path
        d="M32 12c5.6 6.6 8.6 11.3 8.6 15.2a8.6 8.6 0 0 1-17.2 0C23.4 23.3 26.4 18.6 32 12Z"
        fill={drop}
      />

      {/* La feuille, nervure comprise : le vivant que la météo commande. */}
      <path
        d="M18 46c0-10 8-17 26-18-1 13-8 20-18 20a12 12 0 0 1-8-2Z"
        fill={leaf}
      />
      <path
        d="M24 47c7-5 12-9 18-14"
        fill="none"
        stroke={inverted ? '#1d2836' : '#eef3e8'}
        strokeWidth="2.6"
        strokeLinecap="round"
      />
    </svg>
  )
}

/** Marque et nom, côte à côte. */
export function BrandLockup({ size = 30, inverted = false }: Props) {
  return (
    <span className="brand">
      <BrandMark size={size} inverted={inverted} />
      <span className="brand__name">Klima</span>
    </span>
  )
}
