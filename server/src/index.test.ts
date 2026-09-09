import { describe, expect, test } from 'bun:test'
import { createHandler } from './index'

/** Un `fetch` qui compte ses appels et répond ce qu'on lui dit. */
function upstream(reply: (url: URL) => unknown | Promise<unknown>) {
  const seen: URL[] = []
  const call = (async (input: URL | RequestInfo) => {
    const url = new URL(String(input))
    seen.push(url)
    const body = await reply(url)
    if (body instanceof Response) return body
    return new Response(JSON.stringify(body), {
      headers: { 'Content-Type': 'application/json' },
    })
  }) as unknown as typeof fetch
  return { call, seen }
}

const get = (path: string, headers?: HeadersInit) =>
  new Request(`http://relais${path}`, { headers })

describe('relais', () => {
  test('sert la prévision et dit qu’elle est fraîche', async () => {
    const { call, seen } = upstream(() => ({ timezone: 'Europe/Paris' }))
    const handle = createHandler({ fetch: call })

    const response = await handle(get('/v1/open-meteo/forecast?latitude=48.4468&longitude=1.4892'))

    expect(response.status).toBe(200)
    expect(await response.json()).toEqual({ timezone: 'Europe/Paris' })
    expect(response.headers.get('X-Klima-Cache')).toBe('frais')
    expect(seen).toHaveLength(1)
  })

  test('deux parcelles de la même maille ne font qu’une interrogation', async () => {
    const { call, seen } = upstream(() => ({ ok: true }))
    const handle = createHandler({ fetch: call })

    await handle(get('/v1/open-meteo/forecast?latitude=48.4468&longitude=1.4870'))
    const second = await handle(get('/v1/open-meteo/forecast?latitude=48.4490&longitude=1.4885'))

    expect(seen).toHaveLength(1)
    expect(second.headers.get('X-Klima-Cache')).toBe('cache')
  })

  test('le point transmis au fournisseur est celui de la maille, pas celui demandé', async () => {
    const { call, seen } = upstream(() => ({ ok: true }))
    const handle = createHandler({ fetch: call })

    await handle(get('/v1/open-meteo/forecast?latitude=48.4468&longitude=1.4892'))

    expect(seen[0]!.searchParams.get('latitude')).toBe('48.440')
    expect(seen[0]!.searchParams.get('longitude')).toBe('1.480')
  })

  test('la comparaison des modèles est un cache distinct de la prévision seule', async () => {
    const { call, seen } = upstream(() => ({ ok: true }))
    const handle = createHandler({ fetch: call })

    await handle(get('/v1/open-meteo/forecast?latitude=48.44&longitude=1.48'))
    await handle(get('/v1/open-meteo/forecast?latitude=48.44&longitude=1.48&models=icon_seamless'))

    expect(seen).toHaveLength(2)
  })

  test('MET Norway part avec l’en-tête que ses conditions exigent', async () => {
    let agent: string | null = null
    const call = (async (_input: URL | RequestInfo, init?: RequestInit) => {
      agent = new Headers(init?.headers).get('User-Agent')
      return new Response('{}', { headers: { 'Content-Type': 'application/json' } })
    }) as unknown as typeof fetch

    await createHandler({ fetch: call })(get('/v1/met-norway/compact?lat=48.44&lon=1.48'))

    expect(String(agent)).toContain('Klima/')
    expect(String(agent)).toContain('http')
  })

  test('la clé commerciale part chez Open-Meteo et jamais vers le client', async () => {
    const { call, seen } = upstream(() => ({ ok: true }))
    const handle = createHandler({ fetch: call, openMeteoKey: 'clé-secrète' })

    const response = await handle(get('/v1/open-meteo/forecast?latitude=48.44&longitude=1.48'))

    expect(seen[0]!.host).toBe('customer-api.open-meteo.com')
    expect(seen[0]!.searchParams.get('apikey')).toBe('clé-secrète')
    expect(await response.text()).not.toContain('clé-secrète')
  })

  test('une panne du fournisseur ne fuit ni son message ni son URL', async () => {
    const call = (async () =>
      new Response('erreur sur https://customer-api.open-meteo.com/v1/forecast?apikey=clé-secrète', {
        status: 500,
      })) as unknown as typeof fetch

    const response = await createHandler({ fetch: call, openMeteoKey: 'clé-secrète' })(
      get('/v1/open-meteo/forecast?latitude=48.44&longitude=1.48'),
    )

    expect(response.status).toBe(502)
    expect(await response.text()).not.toContain('clé-secrète')
  })

  test('des coordonnées absurdes sont refusées avant tout appel', async () => {
    const { call, seen } = upstream(() => ({ ok: true }))
    const handle = createHandler({ fetch: call })

    for (const query of ['latitude=95&longitude=1', 'latitude=abc&longitude=1', '']) {
      expect((await handle(get(`/v1/open-meteo/forecast?${query}`))).status).toBe(400)
    }
    expect(seen).toHaveLength(0)
  })

  test('le géocodage se met en cache sur le texte, quelle que soit la casse', async () => {
    const { call, seen } = upstream(() => ({ results: [] }))
    const handle = createHandler({ fetch: call })

    await handle(get('/v1/open-meteo/search?name=Reims'))
    await handle(get('/v1/open-meteo/search?name=reims'))
    await handle(get('/v1/open-meteo/search?name=  REIMS  '))

    expect(seen).toHaveLength(1)
  })

  test('une recherche trop courte ne part pas', async () => {
    const { call, seen } = upstream(() => ({ results: [] }))
    await createHandler({ fetch: call })(get('/v1/open-meteo/search?name=r'))
    expect(seen).toHaveLength(0)
  })

  test('seules les origines nommées obtiennent l’en-tête de partage', async () => {
    const { call } = upstream(() => ({ ok: true }))
    const handle = createHandler({ fetch: call, allowedOrigins: ['https://maxlestage.github.io'] })
    const path = '/v1/open-meteo/forecast?latitude=48.44&longitude=1.48'

    const ami = await handle(get(path, { Origin: 'https://maxlestage.github.io' }))
    const inconnu = await handle(get(path, { Origin: 'https://pirate.example' }))

    expect(ami.headers.get('Access-Control-Allow-Origin')).toBe('https://maxlestage.github.io')
    expect(inconnu.headers.get('Access-Control-Allow-Origin')).toBeNull()
  })

  test('l’état de santé ne révèle pas la clé', async () => {
    const handle = createHandler({ openMeteoKey: 'clé-secrète' })
    const body = await (await handle(get('/health'))).text()
    expect(body).toContain('configurée')
    expect(body).not.toContain('clé-secrète')
  })

  test('un chemin inconnu répond 404 sans rien interroger', async () => {
    const { call, seen } = upstream(() => ({ ok: true }))
    const response = await createHandler({ fetch: call })(get('/v1/autre?latitude=48&longitude=1'))
    expect(response.status).toBe(404)
    expect(seen).toHaveLength(0)
  })
})
