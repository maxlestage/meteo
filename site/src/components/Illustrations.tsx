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

/** Scène de tête : un ciel au-dessus d'une parcelle, du blé, un horizon. */
export function SkyScene({ isDay = true, raining = false }: SkyProps) {
  return (
    <svg
      className="scene"
      viewBox="0 0 420 260"
      role="img"
      aria-label="Parcelle sous un ciel changeant"
      preserveAspectRatio="xMidYMid slice"
    >
      <defs>
        <linearGradient id="skyGradient" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={isDay ? '#7fb2e5' : '#3c556e'} />
          <stop offset="100%" stopColor={isDay ? '#cfe3f3' : '#1d2836'} />
        </linearGradient>
        <linearGradient id="fieldGradient" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#9bbf6a" />
          <stop offset="100%" stopColor="#5d7c3a" />
        </linearGradient>
        <linearGradient id="soilGradient" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#8a6a45" />
          <stop offset="100%" stopColor="#5c4529" />
        </linearGradient>
      </defs>

      <rect width="420" height="260" fill="url(#skyGradient)" />

      {isDay ? (
        <g className="scene__sun">
          <circle cx="330" cy="62" r="24" fill="#f7c948" />
          <circle cx="330" cy="62" r="34" fill="#f7c948" opacity="0.22" />
        </g>
      ) : (
        <g className="scene__moon">
          <path d="M344 76a26 26 0 1 1-22-33 20 20 0 0 0 22 33Z" fill="#e8edf2" />
        </g>
      )}

      {/* Trois nuages à des vitesses différentes : le ciel n'est jamais figé. */}
      <g className="scene__cloud scene__cloud--slow" fill="#ffffff" opacity="0.9">
        <ellipse cx="90" cy="66" rx="38" ry="18" />
        <ellipse cx="118" cy="60" rx="26" ry="20" />
        <ellipse cx="62" cy="60" rx="22" ry="15" />
      </g>
      <g className="scene__cloud scene__cloud--fast" fill="#ffffff" opacity="0.75">
        <ellipse cx="240" cy="42" rx="30" ry="14" />
        <ellipse cx="262" cy="38" rx="20" ry="15" />
      </g>
      <g className="scene__cloud scene__cloud--slower" fill="#ffffff" opacity="0.55">
        <ellipse cx="170" cy="96" rx="34" ry="13" />
        <ellipse cx="196" cy="92" rx="22" ry="14" />
      </g>

      {raining && (
        <g className="scene__rain" stroke="#7fd0f5" strokeWidth="2" strokeLinecap="round">
          {Array.from({ length: 14 }, (_, i) => (
            <line
              key={i}
              x1={30 + i * 27}
              y1={110}
              x2={26 + i * 27}
              y2={124}
              style={{ animationDelay: `${(i % 7) * 0.18}s` }}
            />
          ))}
        </g>
      )}

      {/* Parcelle : sillons en perspective, puis la coupe de sol. */}
      <path d="M0 168 Q210 146 420 168 L420 214 L0 214 Z" fill="url(#fieldGradient)" />
      <g stroke="#4d6b2e" strokeWidth="1.4" opacity="0.6">
        {Array.from({ length: 9 }, (_, i) => (
          <path key={i} d={`M${-40 + i * 60} 214 Q${140 + i * 24} 180 ${180 + i * 30} 166`} fill="none" />
        ))}
      </g>
      <rect y="212" width="420" height="48" fill="url(#soilGradient)" />

      {/* Épis de blé : le vent les fait ployer, doucement. */}
      <g className="scene__wheat" stroke="#e0c169" strokeWidth="2.4" strokeLinecap="round">
        {Array.from({ length: 11 }, (_, i) => (
          <g key={i} style={{ animationDelay: `${(i % 5) * 0.35}s` }}>
            <line x1={22 + i * 38} y1={214} x2={22 + i * 38} y2={182} />
            <ellipse cx={22 + i * 38} cy={176} rx="4.5" ry="9" fill="#e8cf85" stroke="none" />
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
