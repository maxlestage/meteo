/**
 * Marque de Klima.
 *
 * Un K dont la hampe est droite et les bras cintrés, comme une pousse qui
 * s'ouvre : la lettre du nom et le geste de la plante dans le même signe. Peu
 * de formes, un trait épais — il tient à vingt-deux pixels comme sur une icône
 * d'application.
 *
 * La géométrie est celle des gabarits de /tmp/brand : favicon, icônes web,
 * icône iOS et watchOS, image de partage et ce composant sortent du même
 * dessin, dans le même repère de 1024. Toute retouche se porte des deux côtés.
 */

interface Props {
  size?: number
  title?: string
}

export function BrandMark({ size = 32, title }: Props) {
  return (
    <svg
      viewBox="0 0 1024 1024"
      width={size}
      height={size}
      role={title ? 'img' : 'presentation'}
      aria-label={title}
      aria-hidden={title ? undefined : true}
      className="brand-mark"
    >
      <defs>
        <linearGradient id="klimaTile" x1="0" y1="0" x2="0.4" y2="1">
          <stop offset="0%" stopColor="#4e7d33" />
          <stop offset="100%" stopColor="#22401a" />
        </linearGradient>
      </defs>

      {/* Le carreau : la parcelle sur laquelle la lettre est posée. */}
      <rect width="1024" height="1024" rx="229" fill="url(#klimaTile)" />

      {/* Les deux bras, cintrés. Le bras haut porte la lumière, le bas la feuille. */}
      <path
        d="M396 512 Q558 466 680 254"
        stroke="#f2f6ec"
        strokeWidth="112"
        strokeLinecap="round"
        fill="none"
      />
      <path
        d="M396 512 Q558 558 680 770"
        stroke="#cfe8a2"
        strokeWidth="112"
        strokeLinecap="round"
        fill="none"
      />

      {/* La hampe, dessinée en dernier : elle referme la jonction des bras. */}
      <rect x="288" y="212" width="112" height="600" rx="56" fill="#f2f6ec" />
    </svg>
  )
}

/** Marque et nom, côte à côte. */
export function BrandLockup({ size = 30 }: Props) {
  return (
    <span className="brand">
      <BrandMark size={size} />
      <span className="brand__name">Klima</span>
    </span>
  )
}
