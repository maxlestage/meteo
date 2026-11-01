/**
 * « À tester » : ce que TestFlight montre aux testeurs, rempli d'après Git.
 *
 * Le workflow recevait des notes et n'en faisait rien : `altool` envoie le
 * binaire, pas le texte. Le champ restait vide, et personne ne savait ce que
 * le build contenait. Ce script l'écrit après l'envoi :
 *
 *   - les notes données au lancement, s'il y en a ;
 *   - « Build N · commit abcd1234 » ;
 *   - le message complet du commit construit, sans ses lignes de signature ;
 *   - les commits livrés depuis le dernier build envoyé.
 *
 * Il attend qu'Apple ait enregistré le build (quelques minutes après l'envoi),
 * puis pose le texte sur chacune de ses langues — `fr-FR` s'il n'en a aucune.
 *
 * Variables : ASC_KEY_ID, ASC_ISSUER_ID, ASC_KEY_P8 (la clé App Store Connect),
 * BUILD (le numéro de build), NOTES, GITHUB_TOKEN, GITHUB_REPOSITORY.
 */

import { createPrivateKey, sign as signRaw } from 'node:crypto'
import { execFileSync } from 'node:child_process'

const BASE = 'https://api.appstoreconnect.apple.com'
const BUNDLE = 'com.kliima.app'
/** La limite d'Apple pour « À tester ». */
const LIMITE = 4000
/** Combien de temps attendre qu'Apple connaisse le build. */
const ATTENTE_MS = 20 * 60 * 1000

function lireCle(brut) {
  const texte = (brut ?? '').trim()
  if (!texte) throw new Error('ASC_KEY_P8 est vide')
  if (texte.startsWith('-----BEGIN')) return texte
  const decode = Buffer.from(texte, 'base64').toString('utf8')
  if (!decode.includes('BEGIN PRIVATE KEY')) throw new Error('ASC_KEY_P8 ne contient pas de clé PEM lisible')
  return decode
}

const base64url = (b) => Buffer.from(b).toString('base64').replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')

/** Un jeton ES256 (signature brute, pas DER : sinon Apple répond 401). */
function jeton() {
  const maintenant = Math.floor(Date.now() / 1000)
  const entree = `${base64url(JSON.stringify({ alg: 'ES256', kid: process.env.ASC_KEY_ID, typ: 'JWT' }))}.${base64url(
    JSON.stringify({ iss: process.env.ASC_ISSUER_ID, iat: maintenant, exp: maintenant + 900, aud: 'appstoreconnect-v1' }),
  )}`
  const signature = signRaw('sha256', Buffer.from(entree), {
    key: createPrivateKey(lireCle(process.env.ASC_KEY_P8)),
    dsaEncoding: 'ieee-p1363',
  })
  return `${entree}.${base64url(signature)}`
}

async function asc(methode, chemin, corps) {
  const reponse = await fetch(`${BASE}${chemin}`, {
    method: methode,
    headers: { Authorization: `Bearer ${jeton()}`, 'Content-Type': 'application/json' },
    body: corps ? JSON.stringify(corps) : undefined,
  })
  const texte = await reponse.text()
  if (!reponse.ok) {
    throw new Error(`${methode} ${chemin} → ${reponse.status} ${texte.slice(0, 300)}`)
  }
  return texte ? JSON.parse(texte) : {}
}

const git = (...args) => execFileSync('git', args, { encoding: 'utf8' }).trim()

/** Un message sans ses lignes de signature (co-auteurs, session). */
function sansSignatures(message) {
  return message
    .split('\n')
    .filter((l) => !/^(Co-Authored-By|Co-authored-by|Claude-Session|Signed-off-by):/i.test(l.trim()))
    // La ligne de tirets que GitHub laisse au bas d'un commit fusionné.
    .filter((l) => !/^-{3,}$/.test(l.trim()))
    .join('\n')
    .replace(/\n{3,}/g, '\n\n')
    .trim()
}

/** Le commit du dernier build envoyé avec succès, s'il est connu. */
async function precedent() {
  const { GITHUB_TOKEN: cle, GITHUB_REPOSITORY: depot, GITHUB_RUN_ID: courant } = process.env
  if (!cle || !depot) return null
  const reponse = await fetch(
    `https://api.github.com/repos/${depot}/actions/workflows/testflight.yml/runs?status=success&per_page=5`,
    { headers: { Authorization: `Bearer ${cle}`, Accept: 'application/vnd.github+json' } },
  )
  if (!reponse.ok) return null
  const { workflow_runs: runs = [] } = await reponse.json()
  const run = runs.find((r) => String(r.id) !== String(courant))
  return run?.head_sha ?? null
}

export async function texte() {
  const sha = git('rev-parse', 'HEAD')
  const lignes = []
  const notes = (process.env.NOTES ?? '').trim()
  if (notes) lignes.push(notes, '')
  lignes.push(`Build ${process.env.BUILD} · commit ${sha.slice(0, 8)}`, '')
  lignes.push(sansSignatures(git('log', '-1', '--format=%B', 'HEAD')))

  let depuis = await precedent()
  if (depuis) {
    try {
      depuis = git('merge-base', depuis, 'HEAD')
    } catch {
      depuis = null
    }
  }
  const plage = depuis && depuis !== sha ? [`${depuis}..HEAD`] : ['-15', 'HEAD']
  const sujets = git('log', '--format=%s', ...plage).split('\n').filter(Boolean)
  if (sujets.length > 1) {
    lignes.push('', depuis ? 'Commits depuis le build précédent :' : 'Derniers commits :')
    for (const sujet of sujets) lignes.push(`• ${sujet}`)
  }

  let tout = lignes.join('\n').trim()
  if (tout.length > LIMITE) tout = `${tout.slice(0, LIMITE - 1)}…`
  return tout
}

async function buildDApple(appId) {
  const fin = Date.now() + ATTENTE_MS
  while (Date.now() < fin) {
    const { data = [] } = await asc(
      'GET',
      `/v1/builds?filter[app]=${appId}&filter[version]=${encodeURIComponent(process.env.BUILD)}&limit=1`,
    )
    if (data[0]) return data[0]
    console.log('build pas encore enregistré chez Apple, nouvel essai dans 30 s')
    await new Promise((r) => setTimeout(r, 30_000))
  }
  throw new Error(`build ${process.env.BUILD} introuvable après ${ATTENTE_MS / 60000} minutes`)
}

async function main() {
  const quoi = await texte()
  console.log('--- À tester ---\n' + quoi + '\n----------------')

  const { data: apps = [] } = await asc('GET', `/v1/apps?filter[bundleId]=${BUNDLE}&limit=1`)
  if (!apps[0]) throw new Error(`aucune application ${BUNDLE} chez Apple`)
  const build = await buildDApple(apps[0].id)

  const { data: langues = [] } = await asc('GET', `/v1/builds/${build.id}/betaBuildLocalizations`)
  if (langues.length === 0) {
    await asc('POST', '/v1/betaBuildLocalizations', {
      data: {
        type: 'betaBuildLocalizations',
        attributes: { locale: 'fr-FR', whatsNew: quoi },
        relationships: { build: { data: { type: 'builds', id: build.id } } },
      },
    })
    console.log('« À tester » posé (fr-FR)')
    return
  }
  for (const langue of langues) {
    await asc('PATCH', `/v1/betaBuildLocalizations/${langue.id}`, {
      data: { type: 'betaBuildLocalizations', id: langue.id, attributes: { whatsNew: quoi } },
    })
    console.log(`« À tester » posé (${langue.attributes?.locale})`)
  }
}

if (process.argv[2] === '--apercu') {
  texte().then((t) => console.log(t))
} else {
  main().catch((e) => {
    console.error(`::error::${e.message}`)
    process.exit(1)
  })
}
