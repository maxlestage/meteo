import { describe, expect, test } from 'bun:test'
import { ForecastCache } from './cache'

/** Horloge qu'on avance à la main : les tests n'attendent jamais. */
function clock(start = 0) {
  let value = start
  return { now: () => value, advance: (ms: number) => { value += ms } }
}

const HOUR = 3_600_000

describe('cache mutualisé', () => {
  test('la deuxième demande ne touche pas le fournisseur', async () => {
    const cache = new ForecastCache({ ttlMs: HOUR, staleMs: 2 * HOUR })
    const fetcher = () => Promise.resolve({ t: 16.4 })

    const first = await cache.serve('cell', fetcher)
    const second = await cache.serve('cell', fetcher)

    expect(first.freshness).toBe('frais')
    expect(second.freshness).toBe('cache')
    expect(second.value).toEqual({ t: 16.4 })
    expect(cache.calls).toBe(1)
  })

  test('cent demandes simultanées ne font qu’un appel', async () => {
    const cache = new ForecastCache({ ttlMs: HOUR, staleMs: 2 * HOUR })
    let started = 0
    const fetcher = async () => {
      started += 1
      await new Promise((resolve) => setTimeout(resolve, 5))
      return { t: 16.4 }
    }

    const served = await Promise.all(
      Array.from({ length: 100 }, () => cache.serve('cell', fetcher)),
    )

    expect(started).toBe(1)
    expect(cache.calls).toBe(1)
    expect(served.every((s) => s.value.t === 16.4)).toBe(true)
  })

  test('deux cellules distinctes sont deux appels', async () => {
    const cache = new ForecastCache({ ttlMs: HOUR, staleMs: 2 * HOUR })
    await cache.serve('a', () => Promise.resolve(1))
    await cache.serve('b', () => Promise.resolve(2))
    expect(cache.calls).toBe(2)
  })

  test('passé le TTL, on refait l’appel', async () => {
    const time = clock()
    const cache = new ForecastCache({ ttlMs: HOUR, staleMs: 2 * HOUR, now: time.now })

    await cache.serve('cell', () => Promise.resolve(1))
    time.advance(HOUR - 1)
    expect((await cache.serve('cell', () => Promise.resolve(2))).value).toBe(1)
    time.advance(2)
    expect((await cache.serve('cell', () => Promise.resolve(2))).value).toBe(2)
    expect(cache.calls).toBe(2)
  })

  test('une journée d’ouvertures répétées coûte le nombre d’heures, pas le nombre d’ouvertures', async () => {
    const time = clock()
    const cache = new ForecastCache({ ttlMs: HOUR, staleMs: 2 * HOUR, now: time.now })

    // Vingt appareils qui consultent la même cellule tous les quarts d'heure.
    for (let quarter = 0; quarter < 24 * 4; quarter += 1) {
      for (let device = 0; device < 20; device += 1) {
        await cache.serve('cell', () => Promise.resolve(quarter))
      }
      time.advance(15 * 60_000)
    }

    // 1 920 consultations, 24 interrogations.
    expect(cache.calls).toBe(24)
  })

  test('si le fournisseur tombe, on sert le périmé plutôt que rien', async () => {
    const time = clock()
    const cache = new ForecastCache({ ttlMs: HOUR, staleMs: 2 * HOUR, now: time.now })

    await cache.serve('cell', () => Promise.resolve('bon'))
    time.advance(HOUR + 1)

    const served = await cache.serve<string>('cell', () => Promise.reject(new Error('panne')))
    expect(served.value).toBe('bon')
    expect(served.freshness).toBe('perime')
    expect(served.ageSeconds).toBe(3600)
  })

  test('passé le dépannage, la panne remonte', async () => {
    const time = clock()
    const cache = new ForecastCache({ ttlMs: HOUR, staleMs: 2 * HOUR, now: time.now })

    await cache.serve('cell', () => Promise.resolve('bon'))
    time.advance(3 * HOUR + 1)

    await expect(cache.serve<string>('cell', () => Promise.reject(new Error('panne')))).rejects.toThrow('panne')
  })

  test('une panne ne laisse pas la cellule bloquée pour toujours', async () => {
    const cache = new ForecastCache({ ttlMs: HOUR, staleMs: 2 * HOUR })

    await expect(cache.serve<string>('cell', () => Promise.reject(new Error('panne')))).rejects.toThrow()
    // Sans nettoyage du vol en cours, cette demande attendrait la promesse rejetée.
    expect((await cache.serve('cell', () => Promise.resolve('bon'))).value).toBe('bon')
  })

  test('le balayage oublie ce que même le dépannage ne servirait plus', async () => {
    const time = clock()
    const cache = new ForecastCache({ ttlMs: HOUR, staleMs: 2 * HOUR, now: time.now })

    await cache.serve('cell', () => Promise.resolve(1))
    time.advance(2 * HOUR)
    expect(cache.sweep()).toBe(0)
    time.advance(HOUR + 1)
    expect(cache.sweep()).toBe(1)
    expect(cache.size).toBe(0)
  })
})
