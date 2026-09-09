/**
 * Illustrations vectorielles, dessinées à la main.
 *
 * Pas de photographies : rien à licencier, rien à charger, et le trait reste
 * net à toutes les tailles. Les animations sont en CSS et se coupent d'elles-
 * mêmes si le système demande moins de mouvement.
 */

interface SkyProps {
  /** Vrai le jour : le soleil remplace la lune. */
  isDay?: boolean
  /** Vrai quand la pluie tombe sur la parcelle. */
  raining?: boolean
}

/**
 * Scène de tête : un bandeau large, dessiné au format où il s'affiche. Le
 * gabarit est volontairement plat (420 × 96) — un carré rogné dans une bande
 * perdrait le soleil et l'horizon.
 */
export function SkyScene({ isDay = true, raining = false }: SkyProps) {
  return (
    <svg
      className="scene"
      viewBox="0 0 420 96"
      role="img"
      aria-label="Parcelle sous un ciel changeant"
      preserveAspectRatio="xMidYMid slice"
    >
      <defs>
        <linearGradient id="skyGradient" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#3c556e" />
          <stop offset="100%" stopColor="#1d2836" />
        </linearGradient>
        <radialGradient id="sunGlow">
          <stop offset="0%" stopColor="#f7c948" stopOpacity="0.45" />
          <stop offset="100%" stopColor="#f7c948" stopOpacity="0" />
        </radialGradient>
        <linearGradient id="fieldGradient" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#6f9448" />
          <stop offset="100%" stopColor="#33491e" />
        </linearGradient>
      </defs>

      <rect width="420" height="96" fill="url(#skyGradient)" />

      {isDay ? (
        <g className="scene__sun">
          <circle cx="352" cy="30" r="13" fill="#f7c948" />
          <circle cx="352" cy="30" r="30" fill="url(#sunGlow)" />
        </g>
      ) : (
        <g className="scene__moon">
          <path d="M360 38a14 14 0 1 1-12-18 11 11 0 0 0 12 18Z" fill="#e8edf2" />
        </g>
      )}

      {/* Trois nuages à des vitesses différentes : le ciel n'est jamais figé. */}
      <g className="scene__cloud scene__cloud--slow" fill="#c6d6e4" opacity="0.3">
        <ellipse cx="88" cy="30" rx="34" ry="10" />
        <ellipse cx="112" cy="25" rx="22" ry="11" />
        <ellipse cx="64" cy="26" rx="18" ry="9" />
      </g>
      <g className="scene__cloud scene__cloud--fast" fill="#c6d6e4" opacity="0.22">
        <ellipse cx="238" cy="19" rx="27" ry="8" />
        <ellipse cx="258" cy="16" rx="17" ry="8" />
      </g>
      <g className="scene__cloud scene__cloud--slower" fill="#c6d6e4" opacity="0.16">
        <ellipse cx="168" cy="44" rx="30" ry="8" />
        <ellipse cx="192" cy="41" rx="19" ry="8" />
      </g>

      {raining && (
        <g className="scene__rain" stroke="#7fd0f5" strokeWidth="1.6" strokeLinecap="round">
          {Array.from({ length: 14 }, (_, i) => (
            <line
              key={i}
              x1={30 + i * 27}
              y1={40}
              x2={27 + i * 27}
              y2={50}
              style={{ animationDelay: `${(i % 7) * 0.18}s` }}
            />
          ))}
        </g>
      )}

      {/* Parcelle : un horizon bombé, puis des sillons en perspective. */}
      <path d="M0 72 Q210 58 420 72 L420 96 L0 96 Z" fill="url(#fieldGradient)" />
      <g stroke="#2f4a1c" strokeWidth="1" opacity="0.55">
        {Array.from({ length: 9 }, (_, i) => (
          <path key={i} d={`M${-40 + i * 60} 96 Q${140 + i * 24} 82 ${180 + i * 30} 70`} fill="none" />
        ))}
      </g>

      {/* Épis de blé : le vent les fait ployer, doucement. */}
      <g className="scene__wheat" stroke="#c9a95a" strokeWidth="1.8" strokeLinecap="round">
        {Array.from({ length: 11 }, (_, i) => (
          <g key={i} style={{ animationDelay: `${(i % 5) * 0.35}s` }}>
            <line x1={22 + i * 38} y1={96} x2={22 + i * 38} y2={80} />
            <ellipse cx={22 + i * 38} cy={77} rx="2.4" ry="7" fill="#d6bb72" stroke="none" />
          </g>
        ))}
      </g>
    </svg>
  )
}

/**
 * Coupe de sol : les couches, la bande 3–9 cm que Klima mesure, et le
 * thermomètre à 6 cm.
 */
export function SoilProfile() {
  return (
    <svg
      className="soil"
      viewBox="0 0 320 200"
      role="img"
      aria-label="Coupe de sol : humidité entre 3 et 9 cm, température à 6 cm"
    >
      <defs>
        <linearGradient id="topSoil" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#7d5f3d" />
          <stop offset="100%" stopColor="#63492c" />
        </linearGradient>
      </defs>

      <rect y="34" width="320" height="166" fill="url(#topSoil)" rx="10" />
      <path d="M0 44 Q80 28 160 40 T320 38 L320 44 Z" fill="#8fbf5f" />

      {/* La bande mesurée, mise en avant. */}
      <rect className="soil__band" x="0" y="66" width="320" height="48" fill="#4aa3d8" opacity="0.28" />
      <line x1="0" y1="66" x2="320" y2="66" stroke="#7fd0f5" strokeWidth="1.5" strokeDasharray="5 4" />
      <line x1="0" y1="114" x2="320" y2="114" stroke="#7fd0f5" strokeWidth="1.5" strokeDasharray="5 4" />
      <text x="12" y="60" className="soil__label">3 cm</text>
      <text x="12" y="130" className="soil__label">9 cm</text>

      {/* Racine qui descend dans la bande. */}
      <path
        className="soil__root"
        d="M168 44 C168 70 158 84 162 104 C166 124 152 138 156 164"
        fill="none"
        stroke="#e8e0cd"
        strokeWidth="2.6"
        strokeLinecap="round"
      />
      <path d="M162 96 C150 100 144 96 138 90" fill="none" stroke="#e8e0cd" strokeWidth="1.8" strokeLinecap="round" />
      <path d="M160 128 C172 132 178 128 184 122" fill="none" stroke="#e8e0cd" strokeWidth="1.8" strokeLinecap="round" />

      {/* Thermomètre à 6 cm. */}
      <g transform="translate(258 60)">
        <rect x="0" y="0" width="12" height="46" rx="6" fill="#efe8d8" />
        <rect className="soil__mercury" x="3.5" y="14" width="5" height="32" rx="2.5" fill="#d2603f" />
        <circle cx="6" cy="52" r="9" fill="#d2603f" />
        <text x="24" y="34" className="soil__label">6 cm</text>
      </g>
    </svg>
  )
}

/** Pictogramme d'une fenêtre de traitement : le vent, la buse, la cible. */
export function SprayScene() {
  return (
    <svg className="spray-scene" viewBox="0 0 320 160" role="img" aria-label="Fenêtre de pulvérisation">
      <g className="spray-scene__wind" stroke="#8fb6d8" strokeWidth="2.4" strokeLinecap="round" fill="none">
        <path d="M20 44 H96 a10 10 0 1 0-10-12" />
        <path d="M14 68 H120 a9 9 0 1 1-9 11" />
        <path d="M28 92 H84" />
      </g>

      <g transform="translate(180 24)">
        <rect x="34" y="6" width="10" height="46" rx="4" fill="#5f6659" />
        <rect x="0" y="46" width="78" height="10" rx="5" fill="#3f6b2b" />
        {Array.from({ length: 5 }, (_, i) => (
          <g key={i} className="spray-scene__jet" style={{ animationDelay: `${i * 0.16}s` }}>
            <path
              d={`M${10 + i * 15} 58 L${4 + i * 15} 96 L${18 + i * 15} 96 Z`}
              fill="#7fd0f5"
              opacity="0.55"
            />
          </g>
        ))}
      </g>

      <path d="M0 132 Q160 118 320 132 L320 160 L0 160 Z" fill="#7ea94f" />
    </svg>
  )
}
