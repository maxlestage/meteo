/**
 * Marque de Klima.
 *
 * Un K dont la hampe porte une goutte et dont les bras sont taillés en lames
 * de feuille : la lettre du nom, la pluie et le vivant dans un seul signe. Peu
 * de formes, un trait large — il tient sur une icône d'application comme dans
 * un pied de page.
 *
 * La géométrie est celle des gabarits de /tmp/brand : favicon, icônes web,
 * icône iOS et watchOS, aperçus de lien partagé et ce composant sortent du
 * même dessin, dans le même repère de 1024. Toute retouche se porte des deux
 * côtés.
 *
 * Le composant vit dans le paquet partagé : l'application web et le site
 * affichent le même signe, pas deux copies qui divergeront.
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

      {/* La hampe. */}
      <rect x="322" y="276" width="96" height="560" rx="48" fill="#f2f6ec" />

      {/* Les deux bras, taillés en lames : celui du haut prend la lumière. */}
      <path d="M418 556c118-36 214-124 286-244 22 148-70 268-206 316Z" fill="#f2f6ec" />
      <path d="M418 556c118 36 214 124 286 244 22-148-70-268-206-316Z" fill="#b9d98f" />

      {/* La goutte, posée sur la hampe : ce que la météo apporte à la plante. */}
      <circle cx="370" cy="240" r="52" fill="#7fd0f5" />
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
