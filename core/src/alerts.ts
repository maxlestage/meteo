/**
 * Les alertes : ce que Klima dit sans qu'on ouvre l'application.
 *
 * Miroir Swift : `ios/Klima/Models/Alerts.swift`. Toute règle ajoutée ici se
 * porte là-bas, avec les mêmes cas de test.
 *
 * Ce module ne notifie rien. Il répond à une seule question — « qu'y a-t-il à
 * dire, maintenant ? » — à partir de la prévision et de ce qu'on a déjà dit.
 * L'acheminement (notification locale, APNs) est affaire de plateforme ; la
 * règle est affaire de domaine, et c'est ce qui permet de la tester des deux
 * côtés sans appareil.
 *
 * Trois principes valent mieux que trente réglages :
 *
 * 1. **Une alerte sans marge de manœuvre est du bruit.** Prévenir qu'une
 *    fenêtre s'ouvre dans dix minutes ne sert à rien : on ne sort pas le
 *    pulvérisateur en dix minutes. D'où un préavis minimum.
 * 2. **On ne réveille personne.** Une alerte calculée la nuit attend le matin.
 *    Si l'événement est passé entre-temps, elle est abandonnée : mieux vaut se
 *    taire que raconter la veille.
 * 3. **On ne répète pas.** Une même nature d'alerte ne repart pas avant un
 *    délai de garde, sauf si ce qu'elle annonce a changé.
 */
import type { AgroSummary, HourlySample, SoilState } from './agro'
import { AgroThresholds } from './agro'
import type { Params } from './i18n'

export const ALERT_KINDS = ['fenetre', 'gel', 'sol', 'pluie'] as const
export type AlertKind = (typeof ALERT_KINDS)[number]

export interface Alert {
  kind: AlertKind
  /** Moment de ce qui est annoncé — pas celui de l'envoi. */
  at: Date
  /** Clés de catalogue : le domaine ne fabrique pas de phrases. */
  titleKey: string
  bodyKey: string
  params: Params
}

/** Ce qu'on sait déjà avoir dit. Sérialisable : il survit aux redémarrages. */
export interface AlertState {
  /** Dernier envoi par nature. */
  lastSent: Partial<Record<AlertKind, Date>>
  /** État du sol au dernier examen, pour repérer le moment où il devient portant. */
  lastSoilState?: SoilState
}

export const EMPTY_ALERT_STATE: AlertState = { lastSent: {} }

export interface AlertOptions {
  now: Date
  /** Heure locale à partir de laquelle on se tait, et heure de reprise. */
  quietFrom: number
  quietTo: number
  /** Délai de garde entre deux alertes de même nature (heures). */
  cooldownHours: number
  /** Préavis minimum avant une fenêtre de traitement (heures). */
  leadHours: number
  /** Préavis maximum : au-delà, il est trop tôt pour en parler. */
  horizonHours: number
}

export const DEFAULT_ALERT_OPTIONS: Omit<AlertOptions, 'now'> = {
  // On se tait de 21 h à 6 h. Le gel de la nuit se dit donc en soirée, quand
  // on peut encore bâcher ou allumer, pas à trois heures du matin.
  quietFrom: 21,
  quietTo: 6,
  cooldownHours: 6,
  leadHours: 2,
  horizonHours: 18,
}

/** Vrai si l'heure locale tombe dans la plage de silence. */
export function isQuiet(hour: number, from: number, to: number): boolean {
  return from <= to ? hour >= from && hour < to : hour >= from || hour < to
}

/**
 * Repousse un envoi à la fin du silence. Renvoie `null` si l'événement annoncé
 * sera passé d'ici là : une alerte en retard est pire que pas d'alerte.
 */
export function deferPastQuietHours(
  send: Date,
  event: Date,
  options: Pick<AlertOptions, 'quietFrom' | 'quietTo'>,
): Date | null {
  if (!isQuiet(send.getHours(), options.quietFrom, options.quietTo)) return send

  const resume = new Date(send)
  if (send.getHours() >= options.quietTo) resume.setDate(resume.getDate() + 1)
  resume.setHours(options.quietTo, 0, 0, 0)

  return resume.getTime() <= event.getTime() ? resume : null
}

function held(kind: AlertKind, state: AlertState, options: AlertOptions): boolean {
  const last = state.lastSent[kind]
  if (!last) return false
  return options.now.getTime() - last.getTime() < options.cooldownHours * 3_600_000
}

/**
 * Ce qu'il y a à dire maintenant.
 *
 * Renvoie les alertes prêtes à partir, dans l'ordre où elles ont été trouvées.
 * Une liste vide est le cas normal : la plupart des heures n'ont rien à
 * annoncer, et c'est ce qui rend les alertes supportables.
 */
export function evaluateAlerts(
  summary: AgroSummary,
  hours: readonly HourlySample[],
  state: AlertState,
  options: AlertOptions,
): Alert[] {
  const alerts: Alert[] = []
  const nowMs = options.now.getTime()

  // 1. La fenêtre de traitement qui s'ouvre — la raison d'être des alertes.
  const spray = summary.nextSpray
  if (spray && !held('fenetre', state, options)) {
    const inHours = (spray.start.getTime() - nowMs) / 3_600_000
    if (inHours >= options.leadHours && inHours <= options.horizonHours) {
      alerts.push({
        kind: 'fenetre',
        at: spray.start,
        titleKey: 'alert.fenetre.title',
        bodyKey: 'alert.fenetre.body',
        params: { score: spray.score },
      })
    }
  }

  // 2. Le gel de la nuit. On le dit tant qu'il reste quelque chose à faire.
  if (summary.frost.severity !== 'aucun' && !held('gel', state, options)) {
    alerts.push({
      kind: 'gel',
      at: firstFreezingHour(hours) ?? options.now,
      titleKey: 'alert.gel.title',
      bodyKey: 'alert.gel.body',
      params: { temperature: summary.frost.minTemperature },
    })
  }

  // 3. Le sol devenu portant : le moment où l'on peut entrer en parcelle.
  //    C'est un changement d'état, pas un état — sans le précédent, on se tait.
  if (
    state.lastSoilState === 'sature' &&
    summary.soil.state === 'ressuye' &&
    !held('sol', state, options)
  ) {
    alerts.push({
      kind: 'sol',
      at: options.now,
      titleKey: 'alert.sol.title',
      bodyKey: 'alert.sol.body',
      params: { moisture: summary.soil.moisture },
    })
  }

  // 4. La pluie qui laverait un traitement fait à l'instant.
  if (spray && !held('pluie', state, options)) {
    const washout = rainAfter(hours, spray.end, 6)
    if (washout) {
      alerts.push({
        kind: 'pluie',
        at: washout.time,
        titleKey: 'alert.pluie.title',
        bodyKey: 'alert.pluie.body',
        params: { rain: washout.precipitation },
      })
    }
  }

  return alerts
}

/** Première heure sous zéro de la série, s'il y en a une. */
function firstFreezingHour(hours: readonly HourlySample[]): Date | null {
  return hours.find((hour) => hour.temperature <= 0)?.time ?? null
}

/** Première pluie significative dans les `within` heures suivant `from`. */
function rainAfter(
  hours: readonly HourlySample[],
  from: Date,
  within: number,
): HourlySample | null {
  const limit = from.getTime() + within * 3_600_000
  return (
    hours.find(
      (hour) =>
        hour.time.getTime() >= from.getTime() &&
        hour.time.getTime() <= limit &&
        hour.precipitation > AgroThresholds.sprayRainMax,
    ) ?? null
  )
}

/** Enregistre ce qui vient d'être dit, pour ne pas le redire. */
export function recordSent(state: AlertState, alerts: readonly Alert[], sentAt: Date): AlertState {
  const lastSent = { ...state.lastSent }
  for (const alert of alerts) lastSent[alert.kind] = sentAt
  return { ...state, lastSent }
}

/** Retient l'état du sol pour repérer le prochain ressuyage. */
export function recordSoil(state: AlertState, soil: SoilState): AlertState {
  return { ...state, lastSoilState: soil }
}
