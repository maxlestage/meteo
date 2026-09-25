/**
 * Servir la vitrine et l'application depuis le relais.
 *
 * Le relais tourne déjà quelque part de public : lui faire servir les fichiers
 * construits évite un second hébergement, et l'application se retrouve sur la
 * même origine que ses appels — donc plus de CORS à accorder, et rien à
 * configurer pour qu'elle trouve le relais.
 *
 * C'est un serveur de fichiers minuscule, et il n'a pas besoin d'être plus :
 * quelques dizaines de fichiers construits par Vite, tous connus d'avance.
 */

import { join, resolve, sep, extname } from 'node:path'

const TYPES: Record<string, string> = {
  '.html': 'text/html;charset=utf-8',
  '.js': 'text/javascript;charset=utf-8',
  '.css': 'text/css;charset=utf-8',
  '.json': 'application/json;charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.webp': 'image/webp',
  '.ico': 'image/x-icon',
  '.woff2': 'font/woff2',
  '.woff': 'font/woff',
  '.txt': 'text/plain;charset=utf-8',
  '.webmanifest': 'application/manifest+json',
  '.map': 'application/json;charset=utf-8',
}

/**
 * Le chemin demandé, ramené à un fichier sous la racine — ou rien.
 *
 * Deux pièges, et le second est le sérieux :
 *
 * - Un dossier ne se sert pas : `/` et `/app/` désignent leur `index.html`.
 * - `%2e%2e/` remonte l'arborescence aussi bien que `../`. On décode donc
 *   *avant* de normaliser, puis on vérifie que le résultat est toujours sous
 *   la racine. Sans cette vérification finale, un relais public sert ses
 *   propres secrets.
 */
export function resolveWithin(root: string, pathname: string): string | null {
  let decoded: string
  try {
    decoded = decodeURIComponent(pathname)
  } catch {
    return null // pourcentage mal formé : on ne devine pas
  }
  if (decoded.includes('\0')) return null

  // On résout depuis la racine, sans barre de tête : « /../x » devient alors
  // « ../x », qui sort vraiment de la racine, et se fait refuser. En gardant
  // la barre, `resolve` aurait absorbé les « .. » contre « / » et rendu un
  // chemin faussement rassurant.
  const base = resolve(root)
  const cible = resolve(base, decoded.replace(/^\/+/, ''))

  // La comparaison porte le séparateur : sans lui, « …/publicité » passerait
  // pour un enfant de « …/public ».
  if (cible !== base && !cible.startsWith(base + sep)) return null

  return decoded.endsWith('/') || extname(cible) === ''
    ? join(cible, 'index.html')
    : cible
}

/** Le type d'un fichier d'après son extension, `application/octet-stream` sinon. */
export function contentType(chemin: string): string {
  return TYPES[extname(chemin).toLowerCase()] ?? 'application/octet-stream'
}

/**
 * Combien de temps garder le fichier.
 *
 * Vite pose une empreinte dans le nom des fichiers de `assets/` : leur contenu
 * ne changera jamais, ils peuvent donc être gardés un an. Le `index.html`, lui,
 * désigne ces noms — s'il était gardé, une nouvelle version resterait
 * invisible.
 */
export function cacheControl(chemin: string): string {
  return chemin.includes(`${sep}assets${sep}`)
    ? 'public, max-age=31536000, immutable'
    : 'no-cache'
}

export interface FileLike {
  exists(): Promise<boolean>
}

/** Sert un fichier depuis `root`, ou rend `null` si la route n'en désigne aucun. */
export function staticSite(
  root: string,
  ouvrir: (chemin: string) => FileLike,
): (pathname: string) => Promise<Response | null> {
  return async function servir(pathname) {
    const chemin = resolveWithin(root, pathname)
    if (!chemin) return null

    const fichier = ouvrir(chemin)
    if (!(await fichier.exists())) return null

    return new Response(fichier as unknown as BodyInit, {
      headers: {
        'Content-Type': contentType(chemin),
        'Cache-Control': cacheControl(chemin),
      },
    })
  }
}
