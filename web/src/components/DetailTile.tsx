import type { ReactNode } from 'react'

interface Props {
  label: string
  value: string
  caption?: string
  /** Jauge optionnelle : position 0–1 sur une échelle colorée. */
  gauge?: { position: number; gradient: string }
  children?: ReactNode
}

/** Tuile de détail, dans l'esprit des cartes « Vent » ou « Indice UV ». */
export function DetailTile({ label, value, caption, gauge, children }: Props) {
  return (
    <article className="tile">
      <h3 className="card__label">{label}</h3>
      <p className="tile__value">{value}</p>
      {gauge && (
        <div className="gauge" style={{ background: gauge.gradient }}>
          <span
            className="gauge__cursor"
            style={{ left: `${Math.min(Math.max(gauge.position, 0), 1) * 100}%` }}
          />
        </div>
      )}
      {caption && <p className="tile__caption">{caption}</p>}
      {children}
    </article>
  )
}
