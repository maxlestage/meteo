import { useEffect, useRef, useState } from 'react'
import { searchParcelles, type Parcelle } from '../api/openMeteo'

interface Props {
  current: Parcelle
  onSelect: (parcelle: Parcelle) => void
}

/** Barre de recherche de commune, avec repli sur la géolocalisation du navigateur. */
export function ParcelleSearch({ current, onSelect }: Props) {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<Parcelle[]>([])
  const [locating, setLocating] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  // Recherche différée : on laisse l'utilisateur finir de taper.
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
    const onClickOutside = (event: MouseEvent) => {
      if (!containerRef.current?.contains(event.target as Node)) setResults([])
    }
    document.addEventListener('mousedown', onClickOutside)
    return () => document.removeEventListener('mousedown', onClickOutside)
  }, [])

  const choose = (parcelle: Parcelle) => {
    onSelect(parcelle)
    setQuery('')
    setResults([])
    setError(null)
  }

  const locate = () => {
    if (!navigator.geolocation) {
      setError('Géolocalisation indisponible sur ce navigateur.')
      return
    }
    setLocating(true)
    navigator.geolocation.getCurrentPosition(
      (position) => {
        setLocating(false)
        choose({
          name: 'Ma parcelle',
          latitude: position.coords.latitude,
          longitude: position.coords.longitude,
        })
      },
      () => {
        setLocating(false)
        setError('Position refusée. Recherchez la commune à la main.')
      },
      { timeout: 10_000 },
    )
  }

  return (
    <div className="search" ref={containerRef}>
      <div className="search__row">
        <input
          type="search"
          className="search__input"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={`${current.name} — changer de parcelle`}
          aria-label="Rechercher une commune"
        />
        <button type="button" className="button" onClick={locate} disabled={locating}>
          {locating ? 'Localisation…' : 'Me localiser'}
        </button>
      </div>

      {error && <p className="search__error">{error}</p>}

      {results.length > 0 && (
        <ul className="search__results">
          {results.map((result) => (
            <li key={`${result.latitude},${result.longitude}`}>
              <button type="button" onClick={() => choose(result)}>
                <span>{result.name}</span>
                <span className="search__admin">
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
