import { useEffect, useRef, useState } from 'react'
import { searchParcelles, type Parcelle } from '@klima/core'

interface Props {
  current: Parcelle
  onSelect: (parcelle: Parcelle) => void
}

/** Petit sélecteur de commune, pour essayer la section sur sa propre parcelle. */
export function CommuneSearch({ current, onSelect }: Props) {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<Parcelle[]>([])
  const container = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (query.trim().length < 2) {
      setResults([])
      return
    }
    const controller = new AbortController()
    const timer = setTimeout(() => {
      searchParcelles(query, controller.signal)
        .then(setResults)
        .catch(() => {
          if (!controller.signal.aborted) setResults([])
        })
    }, 250)

    return () => {
      clearTimeout(timer)
      controller.abort()
    }
  }, [query])

  useEffect(() => {
    const close = (event: MouseEvent) => {
      if (!container.current?.contains(event.target as Node)) setResults([])
    }
    document.addEventListener('mousedown', close)
    return () => document.removeEventListener('mousedown', close)
  }, [])

  return (
    <div className="commune" ref={container}>
      <input
        type="search"
        className="commune__input"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder={`${current.name} — changer de commune`}
        aria-label="Rechercher une commune"
      />
      {results.length > 0 && (
        <ul className="commune__results">
          {results.map((result) => (
            <li key={`${result.latitude},${result.longitude}`}>
              <button
                type="button"
                onClick={() => {
                  onSelect(result)
                  setQuery('')
                  setResults([])
                }}
              >
                <span>{result.name}</span>
                <span className="commune__admin">
                  {[result.admin, result.country].filter(Boolean).join(', ')}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
