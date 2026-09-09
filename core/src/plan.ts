/**
 * Les paliers d'abonnement.
 *
 * Miroir Swift : `ios/Klima/Models/Plan.swift`. Toute règle ajoutée ici se
 * porte là-bas, avec les mêmes cas de test — comme les seuils agronomiques.
 *
 * Un principe gouverne le découpage : **on ne coupe jamais la réponse du
 * jour.** Un agriculteur qui ouvre Klima pour savoir s'il traite cet
 * après-midi doit l'obtenir sans payer. Ce qui se facture, c'est l'échelle
 * (plusieurs parcelles) et l'anticipation (alertes, cumuls, recoupement).
 * Amputer aujourd'hui rendrait le palier libre inutile, donc l'application
 * invendable.
 *
 * Ce module ne connaît ni StoreKit, ni prix, ni boutique : il dit seulement ce
 * qu'un palier ouvre. Le paiement est affaire d'interface, la règle est
 * affaire de domaine — et c'est ce qui permet de la tester des deux côtés.
 */

export const PLANS = ['libre', 'pro'] as const
export type Plan = (typeof PLANS)[number]

/** Ce qu'un palier peut ouvrir. */
export const FEATURES = [
  /** Comparer plusieurs instituts et afficher leur accord. */
  'recoupement',
  /** Être prévenu sans ouvrir l'application. */
  'alertes',
  /** Cumuls de pluie et de degrés-jours depuis une date choisie. */
  'cumuls',
  /** Export des conditions à l'heure d'un traitement. */
  'registre',
] as const
export type Feature = (typeof FEATURES)[number]

export interface PlanLimits {
  /** Nombre de parcelles suivies. `Infinity` quand il n'y a pas de limite. */
  readonly parcelles: number
  /** Jours de prévision consultables. */
  readonly jours: number
  readonly features: readonly Feature[]
}

export const PLAN_LIMITS: Readonly<Record<Plan, PlanLimits>> = {
  libre: {
    parcelles: 1,
    jours: 7,
    features: [],
  },
  pro: {
    parcelles: Number.POSITIVE_INFINITY,
    jours: 7,
    features: [...FEATURES],
  },
}

export function limitsFor(plan: Plan): PlanLimits {
  return PLAN_LIMITS[plan]
}

/** Vrai si le palier ouvre cette fonction. */
export function allows(plan: Plan, feature: Feature): boolean {
  return PLAN_LIMITS[plan].features.includes(feature)
}

/** Vrai si le palier permet d'en suivre une de plus. */
export function canAddParcelle(plan: Plan, current: number): boolean {
  return current < PLAN_LIMITS[plan].parcelles
}

/**
 * Ce qu'il advient des parcelles quand l'abonnement s'arrête.
 *
 * Rien n'est effacé. Un métier saisonnier plus un abonnement mensuel donne un
 * cycle prévisible — résiliation à l'automne, retour au printemps — et un
 * abonné qui devrait ressaisir vingt parcelles ne revient pas. On rend les
 * parcelles excédentaires inaccessibles, jamais absentes : elles réapparaissent
 * telles quelles au réabonnement.
 *
 * L'ordre est celui de la liste : les premières restent lisibles.
 */
export interface ParcelleAccess<T> {
  readable: T[]
  /** Conservées, mais hors du palier courant. */
  locked: T[]
}

export function partitionParcelles<T>(plan: Plan, parcelles: readonly T[]): ParcelleAccess<T> {
  const limit = PLAN_LIMITS[plan].parcelles
  if (!Number.isFinite(limit)) return { readable: [...parcelles], locked: [] }
  return { readable: parcelles.slice(0, limit), locked: parcelles.slice(limit) }
}

/**
 * Pourquoi une fonction est fermée — une clé, pas une phrase : l'interface la
 * traduit, comme partout ailleurs dans le domaine.
 */
export function upgradeReasonKey(feature: Feature): string {
  return `plan.reason.${feature}`
}
