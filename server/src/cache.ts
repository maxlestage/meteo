/**
 * Le cache mutualisé.
 *
 * Deux mécanismes, et le second compte autant que le premier :
 *
 * 1. **La péremption.** Une entrée sert pendant `ttlMs`, puis on la refait.
 *    C'est ce qui découple la facture du nombre d'utilisateurs : une cellule
 *    coûte le même nombre d'appels qu'elle soit ouverte par une personne ou
 *    par mille.
 *
 * 2. **La coalescence.** Cent appareils qui réveillent la même cellule à la
 *    même seconde ne doivent produire qu'une interrogation, pas cent. Sans
 *    cela, un pic de trafic passe à travers le cache et arrive entier chez le
 *    fournisseur — exactement ce qu'on voulait éviter. Les demandes qui
 *    arrivent pendant qu'une interrogation est en vol attendent la même
 *    promesse.
 *
 * En cas de panne du fournisseur, une entrée périmée reste préférable à une
 * erreur : la météo d'il y a deux heures reste utilisable, l'absence de météo
 * ne l'est pas. `staleMs` dit jusqu'où on accepte de servir du périmé.
 */

export interface CacheOptions {
  /** Durée pendant laquelle une entrée est servie sans être refaite. */
  ttlMs: number
  /**
   * Au-delà du TTL, durée pendant laquelle une entrée périmée peut encore
   * dépanner si le fournisseur ne répond plus.
   */
  staleMs: number
  /** Horloge injectable, pour que les tests n'aient pas à attendre. */
  now?: () => number
}

export interface Entry<T> {
  value: T
  storedAt: number
}

/** Ce qu'on a servi, et d'où ça vient — l'interface le renvoie en en-tête. */
export type Freshness = 'frais' | 'cache' | 'perime'

export interface Served<T> {
  value: T
  freshness: Freshness
  /** Âge de la donnée servie, en secondes. */
  ageSeconds: number
}

export class ForecastCache {
  private readonly entries = new Map<string, Entry<unknown>>()
  private readonly inFlight = new Map<string, Promise<unknown>>()
  private readonly now: () => number

  /** Compteur d'appels réellement partis chez un fournisseur. */
  private upstreamCalls = 0

  constructor(private readonly options: CacheOptions) {
    this.now = options.now ?? (() => Date.now())
  }

  /** Nombre d'interrogations qui ont atteint le fournisseur depuis le départ. */
  get calls(): number {
    return this.upstreamCalls
  }

  /** Nombre de cellules gardées en mémoire. */
  get size(): number {
    return this.entries.size
  }

  /**
   * Sert la clé demandée : depuis le cache s'il est frais, sinon en
   * interrogeant `fetcher` — une seule fois, même si l'on est plusieurs à
   * arriver ensemble.
   */
  async serve<T>(key: string, fetcher: () => Promise<T>): Promise<Served<T>> {
    const entry = this.entries.get(key) as Entry<T> | undefined
    const age = entry ? this.now() - entry.storedAt : Infinity

    if (entry && age < this.options.ttlMs) {
      return { value: entry.value, freshness: 'cache', ageSeconds: Math.round(age / 1000) }
    }

    const pending = this.inFlight.get(key) as Promise<T> | undefined
    if (pending) {
      // Une interrogation est déjà partie pour cette cellule : on l'attend.
      return { value: await pending, freshness: 'frais', ageSeconds: 0 }
    }

    const flight = (async () => {
      this.upstreamCalls += 1
      const value = await fetcher()
      this.entries.set(key, { value, storedAt: this.now() })
      return value
    })()

    this.inFlight.set(key, flight)

    try {
      return { value: await flight, freshness: 'frais', ageSeconds: 0 }
    } catch (error) {
      // Le fournisseur n'a pas répondu. Une prévision d'il y a deux heures
      // vaut mieux qu'un écran vide ; passé `staleMs`, on renonce.
      if (entry && age < this.options.ttlMs + this.options.staleMs) {
        return { value: entry.value, freshness: 'perime', ageSeconds: Math.round(age / 1000) }
      }
      throw error
    } finally {
      this.inFlight.delete(key)
    }
  }

  /** Oublie les entrées que même le mode dépannage ne servirait plus. */
  sweep(): number {
    const limit = this.options.ttlMs + this.options.staleMs
    let removed = 0
    for (const [key, entry] of this.entries) {
      if (this.now() - entry.storedAt >= limit) {
        this.entries.delete(key)
        removed += 1
      }
    }
    return removed
  }
}
