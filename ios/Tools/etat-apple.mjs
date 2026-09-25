/**
 * Ce que le compte Apple contient déjà, et ce qui manque encore.
 *
 * L'archive échoue pour des raisons qui ne sont pas dans le dépôt : un
 * identifiant absent, un enregistrement d'application jamais créé. Chacune
 * coûte six minutes de compilation avant de se manifester, et le message
 * qu'Apple renvoie alors ne nomme presque jamais la cause.
 *
 * Ce script demande à l'API App Store Connect ce qu'elle sait, et le dit en
 * une page. Il ne modifie rien : que des lectures.
 *
 * Il tourne dans un workflow parce que la clé y vit — elle n'a pas à sortir
 * des secrets du dépôt pour qu'on l'interroge.
 */

import { createPrivateKey, sign as signRaw } from 'node:crypto'
import { appendFileSync } from 'node:fs'

const BASE = 'https://api.appstoreconnect.apple.com'

const IDENTIFIANTS = [
  'com.kliima.app',
  'com.kliima.app.widgets',
  'com.kliima.app.watchkitapp',
  'com.kliima.app.watchkitapp.complications',
  'com.kliima.app.tests',
]

/** Le `.p8` se donne tel quel ou encodé : on accepte les deux. */
function lireCle(brut) {
  const texte = (brut ?? '').trim()
  if (!texte) throw new Error('ASC_KEY_P8 est vide')
  if (texte.startsWith('-----BEGIN')) return texte
  const decode = Buffer.from(texte, 'base64').toString('utf8')
  if (!decode.includes('BEGIN PRIVATE KEY')) {
    throw new Error("ASC_KEY_P8 ne contient pas de clé PEM lisible")
  }
  return decode
}

function base64url(buffer) {
  return Buffer.from(buffer).toString('base64')
    .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

/**
 * Un jeton ES256. `ieee-p1363` est le détail qui compte : OpenSSL signe en DER,
 * JOSE veut les deux entiers bruts concaténés. Sans ce réglage, Apple répond
 * 401 sans dire pourquoi.
 */
function jeton({ keyId, issuerId, pem }) {
  const entete = { alg: 'ES256', kid: keyId, typ: 'JWT' }
  const maintenant = Math.floor(Date.now() / 1000)
  const charge = {
    iss: issuerId,
    iat: maintenant,
    exp: maintenant + 900,
    aud: 'appstoreconnect-v1',
  }
  const entree = `${base64url(JSON.stringify(entete))}.${base64url(JSON.stringify(charge))}`
  const signature = signRaw('sha256', Buffer.from(entree), {
    key: createPrivateKey(pem),
    dsaEncoding: 'ieee-p1363',
  })
  return `${entree}.${base64url(signature)}`
}

async function demander(chemin, jwt) {
  const reponse = await fetch(`${BASE}${chemin}`, {
    headers: { Authorization: `Bearer ${jwt}` },
  })
  const texte = await reponse.text()
  let corps
  try { corps = JSON.parse(texte) } catch { corps = { brut: texte.slice(0, 400) } }
  return { statut: reponse.status, corps }
}

function erreurs(corps) {
  if (!corps?.errors?.length) return ''
  return corps.errors.map((e) => `${e.title ?? ''} — ${e.detail ?? ''}`).join(' ; ')
}

const lignes = []
const dire = (ligne = '') => { lignes.push(ligne); console.log(ligne) }

async function main() {
  const pem = lireCle(process.env.ASC_KEY_P8)
  const jwt = jeton({
    keyId: process.env.ASC_KEY_ID,
    issuerId: process.env.ASC_ISSUER_ID,
    pem,
  })

  dire('# État du compte Apple')
  dire()

  // 1. La clé ouvre-t-elle quoi que ce soit ?
  const moi = await demander('/v1/apps?limit=1', jwt)
  if (moi.statut === 401) {
    dire('**La clé est refusée (401).** ' + erreurs(moi.corps))
    dire()
    dire("Vérifier `ASC_KEY_ID`, `ASC_ISSUER_ID` et le contenu de `ASC_KEY_P8`.")
    return
  }
  dire(`La clé répond (${moi.statut}).`)
  dire()

  // 2. L'enregistrement de l'application.
  dire('## L\'application dans App Store Connect')
  dire()
  const app = await demander(
    `/v1/apps?filter[bundleId]=com.kliima.app&fields[apps]=name,bundleId,sku`,
    jwt,
  )
  const fiches = app.corps?.data ?? []
  let appId = null
  if (fiches.length === 0) {
    dire('**Absente.** Aucune application sur `com.kliima.app`.')
    dire()
    dire("C'est un préalable à l'envoi : TestFlight refuse un paquet dont la")
    dire("fiche n'existe pas. Et l'API App Store Connect ne sait pas la créer —")
    dire("elle ne fait que lire les applications. Il faut passer par")
    dire('App Store Connect → Mes apps → +.')
  } else {
    for (const fiche of fiches) {
      appId = fiche.id
      dire(`**${fiche.attributes?.name}** — \`${fiche.attributes?.bundleId}\` (id ${fiche.id})`)
    }
  }
  dire()

  // 3. Les identifiants et leurs capacités.
  dire('## Les identifiants d\'application')
  dire()
  for (const identifiant of IDENTIFIANTS) {
    const r = await demander(
      `/v1/bundleIds?filter[identifier]=${encodeURIComponent(identifiant)}&include=bundleIdCapabilities&limit=1`,
      jwt,
    )
    const trouve = (r.corps?.data ?? [])[0]
    if (!trouve) {
      dire(`- \`${identifiant}\` — **absent**`)
      continue
    }
    const capacites = (r.corps?.included ?? [])
      .filter((i) => i.type === 'bundleIdCapabilities')
      .map((i) => i.attributes?.capabilityType)
      .filter(Boolean)
      .sort()
    const groupes = capacites.includes('APP_GROUPS') ? 'App Groups activé' : 'sans App Groups'
    dire(`- \`${identifiant}\` — présent, ${groupes}${capacites.length ? ` (${capacites.join(', ')})` : ''}`)
  }
  dire()
  dire("La capacité *App Groups* peut être activée par l'API ; **le rattachement")
  dire("d'un groupe précis, non** — `/v1/appGroups` n'existe pas dans l'API")
  dire('publique. C\'est ce rattachement qui manque à la signature.')
  dire()

  // 4. L'abonnement.
  if (appId) {
    dire('## L\'abonnement')
    dire()
    const groupes = await demander(
      `/v1/apps/${appId}/subscriptionGroups?include=subscriptions&limit=10`,
      jwt,
    )
    const abonnements = (groupes.corps?.included ?? [])
      .filter((i) => i.type === 'subscriptions')
    if (abonnements.length === 0) {
      dire('**Aucun.** `com.kliima.app.pro.mensuel` reste à créer ; sans lui,')
      dire("l'écran d'achat s'affiche sans prix.")
    } else {
      for (const a of abonnements) {
        dire(`- \`${a.attributes?.productId}\` — ${a.attributes?.name} (${a.attributes?.state})`)
      }
    }
    dire()
  }
}

main()
  .then(() => {
    // Le même texte dans le résumé de l'exécution : c'est là qu'on le relit
    // sans dérouler un journal.
    const resume = process.env.GITHUB_STEP_SUMMARY
    if (resume) appendFileSync(resume, lignes.join('\n') + '\n')
  })
  .catch((erreur) => {
    console.error(`::error::${erreur.message}`)
    process.exit(1)
  })
