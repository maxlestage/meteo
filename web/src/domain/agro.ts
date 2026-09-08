/**
 * Cœur agronomique de l'application.
 *
 * Toutes les fonctions de ce module sont pures : elles prennent des mesures
 * météo brutes et renvoient des indicateurs directement exploitables au champ.
 * La même logique est implémentée à l'identique côté iOS
 * (ios/MeteoAgricole/Models/AgroIndicators.swift) — les seuils vivent ici et
 * dans `AgroThresholds` côté Swift, et doivent rester synchronisés.
 */

export interface HourlySample {
  /** Horodatage local de la parcelle. */
  time: Date
  /** Température de l'air à 2 m (°C). */
  temperature: number
  /** Humidité relative à 2 m (%). */
  relativeHumidity: number
  /** Point de rosée à 2 m (°C). */
  dewPoint: number
  /** Précipitations sur l'heure (mm). */
  precipitation: number
  /** Vent moyen à 10 m (km/h). */
  windSpeed: number
  /** Rafales à 10 m (km/h). */
  windGusts: number
  /** Température du sol à 6 cm (°C). */
  soilTemperature6cm: number
  /** Humidité volumique du sol entre 3 et 9 cm (m³/m³). */
  soilMoisture3to9cm: number
  /** Évapotranspiration de référence FAO-56 sur l'heure (mm). */
  et0: number
  /** Déficit de pression de vapeur (kPa). */
  vapourPressureDeficit: number
}

export interface DailySample {
  /** Jour local (minuit heure de la parcelle). */
  date: Date
  temperatureMin: number
  temperatureMax: number
  /** Cumul de pluie du jour (mm). */
  precipitationSum: number
  /** Probabilité de pluie maximale du jour (%). */
  precipitationProbabilityMax: number
  /** Cumul d'ET0 FAO-56 du jour (mm). */
  et0Sum: number
  /** Rafales maximales du jour (km/h). */
  windGustsMax: number
}

/**
 * Seuils agronomiques partagés avec l'application iOS.
 * Les valeurs suivent les recommandations usuelles de pulvérisation
 * (vent < 19 km/h — arrêté du 4 mai 2017 en France) et le modèle FAO-56.
 */
export const AgroThresholds = {
  /** Base de calcul des degrés-jours (maïs, tournesol : 6 à 10 °C). */
  gddBase: 10,
  /** Plafond au-delà duquel la plante ne capitalise plus (°C). */
  gddCeiling: 30,
  /** Vent minimum : en dessous, l'inversion thermique fait dériver le produit. */
  sprayWindMin: 3,
  /** Vent maximum réglementaire pour la pulvérisation (km/h). */
  sprayWindMax: 19,
  sprayGustMax: 25,
  sprayTempMin: 5,
  sprayTempMax: 25,
  sprayHumidityMin: 40,
  /** Au-delà de ce VPD, la gouttelette s'évapore avant d'atteindre la cible. */
  sprayVpdMax: 1.2,
  /** Pluie tolérée sur l'heure et l'heure suivante (mm). */
  sprayRainMax: 0.1,
  /** Humidité relative à partir de laquelle on considère le feuillage mouillé. */
  leafWetnessHumidity: 90,
  diseaseTempMin: 8,
  diseaseTempMax: 30,
  /** Humidité volumique du sol : ressuyage et portance. */
  soilTooWet: 0.35,
  soilIdealMin: 0.15,
  soilTooDry: 0.12,
  /** Déficit hydrique déclenchant une alerte d'irrigation (mm sur 7 jours). */
  irrigationDeficit: -15,
} as const

/* ------------------------------------------------------------------ */
/* Degrés-jours de croissance                                          */
/* ------------------------------------------------------------------ */

/**
 * Degrés-jours d'une journée (méthode « moyenne plafonnée »).
 * GDD = clamp((Tmin + Tmax) / 2, base, plafond) − base
 */
export function growingDegreeDays(
  temperatureMin: number,
  temperatureMax: number,
  base: number = AgroThresholds.gddBase,
  ceiling: number = AgroThresholds.gddCeiling,
): number {
  const mean = (temperatureMin + temperatureMax) / 2
  const capped = Math.min(Math.max(mean, base), ceiling)
  return round(capped - base, 1)
}

/** Cumul de degrés-jours sur une série de journées. */
export function cumulativeGdd(days: readonly DailySample[], base = AgroThresholds.gddBase): number {
  return round(
    days.reduce((sum, d) => sum + growingDegreeDays(d.temperatureMin, d.temperatureMax, base), 0),
    1,
  )
}

/* ------------------------------------------------------------------ */
/* Bilan hydrique                                                      */
/* ------------------------------------------------------------------ */

export type WaterStatus = 'deficit' | 'equilibre' | 'excedent'

export interface WaterBalance {
  /** Cumul de pluie sur la période (mm). */
  precipitation: number
  /** Cumul d'évapotranspiration de référence sur la période (mm). */
  evapotranspiration: number
  /** Pluie − ET0 (mm). Négatif = la parcelle puise dans sa réserve. */
  balance: number
  status: WaterStatus
  /** Conseil d'irrigation exprimé en mm à apporter (0 si inutile). */
  irrigationAdvice: number
}

/** Bilan hydrique climatique P − ET0 sur la période fournie. */
export function waterBalance(days: readonly DailySample[]): WaterBalance {
  const precipitation = round(days.reduce((s, d) => s + d.precipitationSum, 0), 1)
  const evapotranspiration = round(days.reduce((s, d) => s + d.et0Sum, 0), 1)
  const balance = round(precipitation - evapotranspiration, 1)

  let status: WaterStatus = 'equilibre'
  if (balance <= AgroThresholds.irrigationDeficit) status = 'deficit'
  else if (balance >= 15) status = 'excedent'

  const irrigationAdvice = status === 'deficit' ? Math.abs(balance) : 0
  return { precipitation, evapotranspiration, balance, status, irrigationAdvice }
}

/* ------------------------------------------------------------------ */
/* Fenêtres de pulvérisation                                           */
/* ------------------------------------------------------------------ */

export type SprayVerdict = 'favorable' | 'acceptable' | 'defavorable'

export interface SprayWindow {
  time: Date
  verdict: SprayVerdict
  /** Score 0–100 : 100 = conditions idéales. */
  score: number
  /** Motifs de dégradation, en clair, pour l'affichage. */
  blockers: string[]
}

/**
 * Évalue l'heure `index` de la série pour un traitement phytosanitaire.
 *
 * Deux familles de critères :
 *  - les critères rédhibitoires (vent au-delà de la limite réglementaire,
 *    rafales, pluie imminente) rendent l'heure inexploitable quoi qu'il arrive ;
 *  - les critères de confort (température, hygrométrie, VPD) dégradent le score.
 */
export function evaluateSprayHour(hours: readonly HourlySample[], index: number): SprayWindow {
  const h = hours[index]
  if (!h) throw new RangeError(`Heure hors série : ${index}`)

  const next = hours[index + 1]
  const blockers: string[] = []
  let score = 100
  /** Un critère rédhibitoire interdit le passage, quel que soit le reste. */
  let disqualified = false

  if (h.windSpeed > AgroThresholds.sprayWindMax) {
    blockers.push(`Vent ${Math.round(h.windSpeed)} km/h (max ${AgroThresholds.sprayWindMax})`)
    score -= 45
    disqualified = true
  } else if (h.windSpeed < AgroThresholds.sprayWindMin) {
    blockers.push('Vent trop faible, risque d’inversion thermique')
    score -= 25
  }

  if (h.windGusts > AgroThresholds.sprayGustMax) {
    blockers.push(`Rafales ${Math.round(h.windGusts)} km/h`)
    score -= 20
    disqualified = true
  }

  const rainSoon = h.precipitation + (next?.precipitation ?? 0)
  if (rainSoon > AgroThresholds.sprayRainMax) {
    blockers.push(`Pluie ${round(rainSoon, 1)} mm dans les 2 h`)
    score -= 45
    disqualified = true
  }

  if (h.temperature > AgroThresholds.sprayTempMax) {
    blockers.push(`Température ${Math.round(h.temperature)} °C`)
    score -= 20
  } else if (h.temperature < AgroThresholds.sprayTempMin) {
    blockers.push(`Température ${Math.round(h.temperature)} °C, trop froid`)
    score -= 20
  }

  if (h.relativeHumidity < AgroThresholds.sprayHumidityMin) {
    blockers.push(`Hygrométrie ${Math.round(h.relativeHumidity)} %`)
    score -= 15
  }

  if (h.vapourPressureDeficit > AgroThresholds.sprayVpdMax) {
    blockers.push(`VPD ${round(h.vapourPressureDeficit, 2)} kPa, évaporation des gouttelettes`)
    score -= 15
  }

  score = Math.max(0, Math.min(100, score))
  if (disqualified) score = Math.min(score, 35)

  const verdict: SprayVerdict = disqualified
    ? 'defavorable'
    : score >= 80
      ? 'favorable'
      : score >= 55
        ? 'acceptable'
        : 'defavorable'
  return { time: h.time, verdict, score, blockers }
}

/** Évalue toute la série horaire. */
export function sprayWindows(hours: readonly HourlySample[]): SprayWindow[] {
  return hours.map((_, i) => evaluateSprayHour(hours, i))
}

/** Prochaine plage d'au moins `minLength` heures consécutives exploitables. */
export function nextSprayOpportunity(
  windows: readonly SprayWindow[],
  minLength = 2,
): { start: Date; end: Date; score: number } | null {
  let run: SprayWindow[] = []
  for (const w of windows) {
    if (w.verdict === 'defavorable') {
      run = []
      continue
    }
    run.push(w)
    if (run.length >= minLength) {
      const first = run[0]!
      const last = run[run.length - 1]!
      const mean = run.reduce((s, x) => s + x.score, 0) / run.length
      return {
        start: first.time,
        end: new Date(last.time.getTime() + 3_600_000),
        score: Math.round(mean),
      }
    }
  }
  return null
}

/* ------------------------------------------------------------------ */
/* Gel                                                                 */
/* ------------------------------------------------------------------ */

export type FrostSeverity = 'aucun' | 'faible' | 'modere' | 'severe'

export interface FrostRisk {
  severity: FrostSeverity
  /** Température minimale attendue (°C). */
  minTemperature: number
  /** Vrai si le point de rosée est négatif : gelée blanche probable. */
  hoarFrost: boolean
}

/** Risque de gel d'une nuit à partir de la température mini et du point de rosée. */
export function frostRisk(minTemperature: number, minDewPoint: number): FrostRisk {
  let severity: FrostSeverity = 'aucun'
  if (minTemperature <= -4) severity = 'severe'
  else if (minTemperature <= -2) severity = 'modere'
  else if (minTemperature <= 1) severity = 'faible'

  return {
    severity,
    minTemperature: round(minTemperature, 1),
    hoarFrost: severity !== 'aucun' && minDewPoint <= 0,
  }
}

/* ------------------------------------------------------------------ */
/* Pression maladie                                                    */
/* ------------------------------------------------------------------ */

export type DiseaseLevel = 'faible' | 'moyenne' | 'elevee'

export interface DiseasePressure {
  /** Heures d'humectation du feuillage (HR ≥ 90 % dans la plage de température). */
  leafWetnessHours: number
  level: DiseaseLevel
}

/**
 * Approximation de la pression cryptogamique (mildiou, septoriose) : on compte
 * les heures d'humectation du feuillage dans la plage de température favorable
 * au champignon, à la manière des tables de Mills.
 */
export function diseasePressure(hours: readonly HourlySample[]): DiseasePressure {
  const leafWetnessHours = hours.filter(
    (h) =>
      h.relativeHumidity >= AgroThresholds.leafWetnessHumidity &&
      h.temperature >= AgroThresholds.diseaseTempMin &&
      h.temperature <= AgroThresholds.diseaseTempMax,
  ).length

  const level: DiseaseLevel = leafWetnessHours >= 12 ? 'elevee' : leafWetnessHours >= 6 ? 'moyenne' : 'faible'
  return { leafWetnessHours, level }
}

/* ------------------------------------------------------------------ */
/* Portance et travail du sol                                          */
/* ------------------------------------------------------------------ */

export type SoilState = 'sature' | 'ressuye' | 'sec'

export interface SoilCondition {
  /** Humidité volumique moyenne 3–9 cm (m³/m³). */
  moisture: number
  /** Température moyenne du sol à 6 cm (°C). */
  temperature: number
  state: SoilState
  /** Vrai si le sol porte les engins sans risque de tassement. */
  trafficable: boolean
  /** Vrai si le sol est assez chaud pour lever (semis de printemps). */
  sowable: boolean
}

/** État du sol moyenné sur la série horaire fournie. */
export function soilCondition(hours: readonly HourlySample[]): SoilCondition {
  if (hours.length === 0) {
    return { moisture: 0, temperature: 0, state: 'sec', trafficable: false, sowable: false }
  }
  const moisture = round(mean(hours.map((h) => h.soilMoisture3to9cm)), 3)
  const temperature = round(mean(hours.map((h) => h.soilTemperature6cm)), 1)

  const state: SoilState =
    moisture >= AgroThresholds.soilTooWet ? 'sature' : moisture <= AgroThresholds.soilTooDry ? 'sec' : 'ressuye'

  return {
    moisture,
    temperature,
    state,
    trafficable: state !== 'sature',
    sowable: temperature >= 8 && moisture >= AgroThresholds.soilIdealMin && state !== 'sature',
  }
}

/* ------------------------------------------------------------------ */
/* Synthèse                                                            */
/* ------------------------------------------------------------------ */

export interface AgroSummary {
  water: WaterBalance
  soil: SoilCondition
  disease: DiseasePressure
  frost: FrostRisk
  gdd: number
  nextSpray: { start: Date; end: Date; score: number } | null
}

/** Assemble tous les indicateurs pour le tableau de bord. */
export function summarize(hours: readonly HourlySample[], days: readonly DailySample[]): AgroSummary {
  const next24h = hours.slice(0, 24)
  const tonight = hours.slice(0, 18)
  return {
    water: waterBalance(days),
    soil: soilCondition(next24h),
    disease: diseasePressure(next24h),
    frost: frostRisk(
      tonight.length ? Math.min(...tonight.map((h) => h.temperature)) : (days[0]?.temperatureMin ?? 0),
      tonight.length ? Math.min(...tonight.map((h) => h.dewPoint)) : 0,
    ),
    gdd: cumulativeGdd(days),
    nextSpray: nextSprayOpportunity(sprayWindows(hours)),
  }
}

/* ------------------------------------------------------------------ */

function mean(values: readonly number[]): number {
  return values.reduce((s, v) => s + v, 0) / values.length
}

function round(value: number, decimals: number): number {
  const factor = 10 ** decimals
  return Math.round(value * factor) / factor
}
