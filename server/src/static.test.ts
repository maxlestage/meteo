import { describe, expect, test } from 'bun:test'
import { cacheControl, contentType, resolveWithin, staticSite } from './static'

const RACINE = '/var/klima/public'

describe('resolveWithin', () => {

  test('la racine désigne son index', () => {
    expect(resolveWithin(RACINE, '/')).toBe('/var/klima/public/index.html')
  })

  test('un dossier aussi, avec ou sans barre finale', () => {
    expect(resolveWithin(RACINE, '/app/')).toBe('/var/klima/public/app/index.html')
    expect(resolveWithin(RACINE, '/app')).toBe('/var/klima/public/app/index.html')
  })

  test('un fichier reste lui-même', () => {
    expect(resolveWithin(RACINE, '/assets/index-abc123.js'))
      .toBe('/var/klima/public/assets/index-abc123.js')
  })

  // Le cas qui compte : un relais public sert sinon ses propres secrets.
  test('on ne remonte pas au-dessus de la racine', () => {
    expect(resolveWithin(RACINE, '/../../etc/passwd')).toBeNull()
    expect(resolveWithin(RACINE, '/app/../../../../etc/passwd')).toBeNull()
  })

  test('ni en encodant les points', () => {
    expect(resolveWithin(RACINE, '/%2e%2e/%2e%2e/etc/passwd')).toBeNull()
  })

  test('un pourcentage mal formé ne se devine pas', () => {
    expect(resolveWithin(RACINE, '/%zz')).toBeNull()
  })

  test('un octet nul non plus', () => {
    expect(resolveWithin(RACINE, '/index.html%00.png')).toBeNull()
  })

  test('une racine voisine n\'est pas la racine', () => {
    // « /var/klima/publicité » commence par « /var/klima/public » : c'est
    // exactement le genre de préfixe qui passe quand on compare sans séparateur.
    expect(resolveWithin(RACINE, '/../publicité/secret.txt')).toBeNull()
  })
})

describe('contentType', () => {
  test('les types que Vite produit', () => {
    expect(contentType('/x/index.html')).toBe('text/html;charset=utf-8')
    expect(contentType('/x/index-abc.js')).toBe('text/javascript;charset=utf-8')
    expect(contentType('/x/index-abc.css')).toBe('text/css;charset=utf-8')
    expect(contentType('/x/marque.svg')).toBe('image/svg+xml')
    expect(contentType('/x/police.woff2')).toBe('font/woff2')
  })

  test('l\'inconnu reste des octets', () => {
    expect(contentType('/x/chose.quoi')).toBe('application/octet-stream')
  })
})

describe('cacheControl', () => {

  test('les fichiers empreintés se gardent un an', () => {
    expect(cacheControl('/var/klima/public/assets/index-abc123.js'))
      .toBe('public, max-age=31536000, immutable')
  })

  // Si l'index était gardé, il continuerait à désigner les anciens fichiers
  // empreintés : la nouvelle version resterait invisible.
  test('l\'index, jamais', () => {
    expect(cacheControl('/var/klima/public/index.html')).toBe('no-cache')
  })
})

describe('staticSite', () => {

  const faux = (presents: string[]) => (chemin: string) => ({
    exists: async () => presents.includes(chemin),
  })

  test('sert un fichier présent avec son type', async () => {
    const servir = staticSite(RACINE, faux(['/var/klima/public/index.html']))
    const reponse = await servir('/')
    expect(reponse?.status).toBe(200)
    expect(reponse?.headers.get('Content-Type')).toBe('text/html;charset=utf-8')
  })

  test('rend null sur un fichier absent, pour que l\'appelant décide', async () => {
    const servir = staticSite(RACINE, faux([]))
    expect(await servir('/inconnu.png')).toBeNull()
  })

  test('rend null sur une tentative de remontée, sans toucher au disque', async () => {
    let ouvert = false
    const servir = staticSite(RACINE, (c) => {
      ouvert = true
      return { exists: async () => true, chemin: c } as never
    })
    expect(await servir('/../../etc/passwd')).toBeNull()
    expect(ouvert).toBe(false)
  })
})
