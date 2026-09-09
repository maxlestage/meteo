/**
 * La parcelle consultée, dans l'adresse.
 *
 * Sans cela, choisir une commune ne laisse aucune trace : le bouton retour du
 * navigateur ne défait rien, recharger la page perd le choix, et envoyer
 * l'adresse à quelqu'un lui montre une autre parcelle que la sienne. Trois
 * défauts pour une seule cause — l'état n'était pas dans l'URL.
 *
 * Ce module n'a pas de miroir Swift, et c'est voulu : ce n'est pas une règle
 * agronomique mais une convention de navigation propre au web. L'application
 * iOS n'a pas d'adresse à porter.
 */
import type { Parcelle } from './openMeteo'

/** Quatre décimales : environ onze mètres, bien au-delà du besoin. */
const PRECISION = 4

/** Écrit la parcelle dans des paramètres d'adresse. */
export function parcelleToParams(parcelle: Parcelle): URLSearchParams {
  const params = new URLSearchParams()
  params.set('parcelle', parcelle.name)
  params.set('lat', parcelle.latitude.toFixed(PRECISION))
  params.set('lon', parcelle.longitude.toFixed(PRECISION))
  if (parcelle.admin) params.set('admin', parcelle.admin)
  if (parcelle.country) params.set('pays', parcelle.country)
  return params
}

/**
 * Relit une parcelle depuis une adresse.
 *
 * Renvoie `null` dès qu'un champ manque ou sort des bornes : une adresse
 * bricolée à la main ne doit pas envoyer l'application chercher la météo d'un
 * point qui n'existe pas.
 */
export function parcelleFromParams(params: URLSearchParams): Parcelle | null {
  const name = params.get('parcelle')?.trim()
  const rawLatitude = params.get('lat')
  const rawLongitude = params.get('lon')
  if (!name || rawLatitude === null || rawLongitude === null) return null

  const latitude = Number(rawLatitude)
  const longitude = Number(rawLongitude)
  if (!Number.isFinite(latitude) || !Number.isFinite(longitude)) return null
  if (latitude < -90 || latitude > 90 || longitude < -180 || longitude > 180) return null

  const admin = params.get('admin')?.trim()
  const country = params.get('pays')?.trim()

  return {
    name,
    latitude,
    longitude,
    ...(admin ? { admin } : {}),
    ...(country ? { country } : {}),
  }
}

/** Vrai si les deux adresses désignent la même parcelle. */
export function sameParcelle(a: Parcelle | null, b: Parcelle | null): boolean {
  if (!a || !b) return a === b
  return (
    a.name === b.name &&
    a.latitude.toFixed(PRECISION) === b.latitude.toFixed(PRECISION) &&
    a.longitude.toFixed(PRECISION) === b.longitude.toFixed(PRECISION)
  )
}

/**
 * L'adresse à afficher pour une parcelle, en gardant le reste de l'existante.
 *
 * On conserve le fragment : sur la vitrine, il désigne la section où l'on se
 * trouve, et le perdre au moment de choisir une commune ferait sauter la page
 * en haut.
 */
export function urlForParcelle(current: string, parcelle: Parcelle): string {
  const url = new URL(current)
  const next = parcelleToParams(parcelle)
  for (const key of ['parcelle', 'lat', 'lon', 'admin', 'pays']) url.searchParams.delete(key)
  for (const [key, value] of next) url.searchParams.set(key, value)
  return `${url.pathname}${url.search}${url.hash}`
}
