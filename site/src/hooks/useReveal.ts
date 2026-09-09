import { useLayoutEffect, useRef, useState } from 'react'

type Phase = 'plain' | 'hidden' | 'shown'

/**
 * Fait apparaître un élément quand il entre dans la fenêtre.
 *
 * Le contenu part **visible** : il n'est masqué que dans l'instant qui précède
 * le premier affichage, et seulement si l'on sait pouvoir le ramener. Sans
 * `IntersectionObserver`, avec JavaScript en panne ou lorsque le système
 * demande moins de mouvement, la section reste simplement lisible — un texte
 * caché par une animation qui ne se déclenche pas est un texte perdu.
 *
 * Le masquage passe par `useLayoutEffect`, donc avant le premier rendu à
 * l'écran : pas de clignotement.
 */
export function useReveal<T extends HTMLElement>() {
  const ref = useRef<T>(null)
  const [phase, setPhase] = useState<Phase>('plain')

  useLayoutEffect(() => {
    const element = ref.current
    if (!element) return

    const reduced =
      typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches
    if (reduced || typeof IntersectionObserver === 'undefined') return

    setPhase('hidden')
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setPhase('shown')
            observer.disconnect()
          }
        }
      },
      { rootMargin: '0px 0px -12% 0px' },
    )

    observer.observe(element)
    return () => observer.disconnect()
  }, [])

  const className = phase === 'plain' ? '' : phase === 'shown' ? 'reveal is-visible' : 'reveal'
  return { ref, className }
}
