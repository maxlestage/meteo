/**
 * Le relais de Klima.
 *
 * Un seul rôle : interroger les fournisseurs météo une fois pour tout le
 * monde. Ce que ça change, dans l'ordre d'importance :
 *
 * 1. **Le prix tient.** Sans relais, la facture Open-Meteo suit le nombre
 *    d'utilisateurs ; avec lui, elle suit le nombre de parcelles distinctes et
 *    le rythme des modèles. Une cellule coûte vingt-quatre interrogations par
 *    jour, qu'elle soit ouverte par une personne ou par mille.
 * 2. **MET Norway reste sous son plafond.** Leurs conditions plafonnent à
 *    vingt requêtes par seconde *par application*, pas par appareil. Au-delà
 *    de quelques milliers d'installations, l'appel direct depuis l'appareil
 *    dépasse ce plafond ; le relais le ramène à une poignée.
 * 3. **La clé commerciale reste secrète.** Elle vit ici, jamais dans un
 *    binaire distribué.
 *
 * Le relais n'est pas un point de défaillance unique : quand il ne répond
 * pas, les clients retombent sur les fournisseurs qu'ils ont le droit
 * d'appeler seuls (MET Norway en natif, Bright Sky partout). On perd des
 * sources, pas la météo.
 */
import { ForecastCache } from './cache'
import { cellFor, cellKey } from './grid'
import {
  brightSkyCurrent,
  metNorwayCompact,
  openMeteoForecast,
  openMeteoSearch,
  UpstreamError,
  type UpstreamConfig,
} from './upstream'

/**
 * Une heure de fraîcheur. Les modèles ne tournent que quelques fois par jour ;
 * rafraîchir plus souvent ne change pas la réponse et multiplie la facture.
 */
const TTL_MS = 3_600_000

/** Au-delà, une prévision périmée dépanne encore pendant deux heures. */
const STALE_MS = 7_200_000

/** Le géocodage ne bouge pas d'un jour à l'autre. */
const SEARCH_TTL_MS = 86_400_000

export interface ServerOptions extends UpstreamConfig {
  port?: number
  /** Origines autorisées à appeler le relais depuis un navigateur. */
  allowedOrigins?: readonly string[]
  now?: () => number
}

const JSON_HEADERS = { 'Content-Type': 'application/json; charset=utf-8' }

function corsHeaders(origin: string | null, allowed: readonly string[]): Record<string, string> {
  if (!origin) return {}
  // Une étoile suffirait — le relais ne sert que des données publiques et ne
  // lit aucun cookie — mais nommer les origines évite qu'il serve de relais
  // ouvert à n'importe quel site.
  if (!allowed.includes(origin) && !allowed.includes('*')) return {}
  return {
    'Access-Control-Allow-Origin': allowed.includes('*') ? '*' : origin,
    'Vary': 'Origin',
  }
}

/** Lit et valide un point de la requête. */
function pointFrom(params: URLSearchParams): { latitude: number; longitude: number } | null {
  const rawLatitude = params.get('latitude') ?? params.get('lat')
  const rawLongitude = params.get('longitude') ?? params.get('lon')
  // Sans ce garde-fou, `Number(null)` vaut zéro et une requête sans
  // coordonnées irait chercher la météo du golfe de Guinée.
  if (rawLatitude === null || rawLongitude === null) return null

  const latitude = Number(rawLatitude)
  const longitude = Number(rawLongitude)
  if (!Number.isFinite(latitude) || !Number.isFinite(longitude)) return null
  if (latitude < -90 || latitude > 90 || longitude < -180 || longitude > 180) return null
  return { latitude, longitude }
}

export function createHandler(options: ServerOptions = {}) {
  const forecasts = new ForecastCache({ ttlMs: TTL_MS, staleMs: STALE_MS, now: options.now })
  const searches = new ForecastCache({ ttlMs: SEARCH_TTL_MS, staleMs: SEARCH_TTL_MS, now: options.now })
  const allowed = options.allowedOrigins ?? ['https://maxlestage.github.io']

  return async function handle(request: Request): Promise<Response> {
    const url = new URL(request.url)
    const cors = corsHeaders(request.headers.get('Origin'), allowed)

    if (request.method === 'OPTIONS') {
      return new Response(null, {
        status: 204,
        headers: { ...cors, 'Access-Control-Allow-Methods': 'GET, OPTIONS' },
      })
    }

    if (request.method !== 'GET') {
      return new Response('méthode non permise', { status: 405, headers: cors })
    }

    if (url.pathname === '/health') {
      return Response.json(
        {
          statut: 'ok',
          cellules: forecasts.size,
          interrogations: forecasts.calls + searches.calls,
          cleOpenMeteo: options.openMeteoKey ? 'configurée' : 'absente',
        },
        { headers: cors },
      )
    }

    // Le géocodage n'a pas de point : il se met en cache sur le texte cherché.
    if (url.pathname === '/v1/open-meteo/search') {
      const name = (url.searchParams.get('name') ?? '').trim().toLowerCase()
      if (name.length < 2) return Response.json([], { headers: cors })
      return respond(
        () => searches.serve(`search:${name}`, () => openMeteoSearch(options, url.searchParams)),
        cors,
      )
    }

    const point = pointFrom(url.searchParams)
    if (!point) {
      return Response.json({ erreur: 'coordonnées manquantes ou hors bornes' }, {
        status: 400,
        headers: cors,
      })
    }

    const cell = cellFor(point.latitude, point.longitude)

    switch (url.pathname) {
      case '/v1/open-meteo/forecast':
        return respond(
          () =>
            forecasts.serve(cellKey(`om:${url.searchParams.get('models') ?? 'seul'}`, cell), () =>
              openMeteoForecast(options, url.searchParams, cell.latitude, cell.longitude),
            ),
          cors,
        )

      case '/v1/met-norway/compact':
        return respond(
          () =>
            forecasts.serve(cellKey('met', cell), () =>
              metNorwayCompact(options, cell.latitude, cell.longitude),
            ),
          cors,
        )

      case '/v1/bright-sky/current':
        return respond(
          () =>
            forecasts.serve(cellKey('brightsky', cell), () =>
              brightSkyCurrent(options, cell.latitude, cell.longitude),
            ),
          cors,
        )

      default:
        return new Response('inconnu', { status: 404, headers: cors })
    }
  }
}

/** Emballe une lecture de cache en réponse HTTP, en disant d'où elle vient. */
async function respond(
  read: () => Promise<{ value: unknown; freshness: string; ageSeconds: number }>,
  cors: Record<string, string>,
): Promise<Response> {
  try {
    const served = await read()
    return new Response(JSON.stringify(served.value), {
      headers: {
        ...JSON_HEADERS,
        ...cors,
        'X-Klima-Cache': served.freshness,
        'Age': String(served.ageSeconds),
        'Cache-Control': `public, max-age=${Math.round(TTL_MS / 1000)}`,
      },
    })
  } catch (error) {
    // On ne renvoie jamais l'erreur du fournisseur telle quelle : elle peut
    // contenir l'URL, donc la clé.
    const status = error instanceof UpstreamError && error.status === 429 ? 429 : 502
    return Response.json({ erreur: 'fournisseur indisponible' }, { status, headers: cors })
  }
}

if (import.meta.main) {
  const options: ServerOptions = {
    openMeteoKey: process.env.OPEN_METEO_KEY,
    allowedOrigins: process.env.KLIMA_ORIGINS?.split(',').map((o) => o.trim()),
  }
  const port = Number(process.env.PORT ?? 8787)
  Bun.serve({ port, fetch: createHandler(options) })
  console.log(`relais Klima sur :${port} — clé Open-Meteo ${
    options.openMeteoKey ? 'configurée' : 'absente (plan gratuit, usage non commercial)'}`)
}
