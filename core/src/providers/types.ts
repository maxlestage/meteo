/**
 * Fournisseurs de prévision.
 *
 * Un fournisseur est un service qu'on interroge ; il peut livrer plusieurs
 * sources (Open-Meteo redistribue quatre modèles nationaux, MET Norway n'en
 * livre qu'un). Ce que le recoupement compare, ce sont les **sources**.
 */

/** Là où un fournisseur peut être appelé sans enfreindre ses conditions. */
export type Platform = 'web' | 'native'

export interface WeatherSource {
  id: string
  /** Nom du modèle ou du produit. */
  name: string
  /** Service qui le produit. */
  institution: string
  /** Code pays ou zone du service. */
  country: string
  /** Fournisseur par lequel on l'obtient. */
  provider: string
  /** Mention que la licence impose d'afficher. */
  attribution: string
}

/** Relevé d'une source pour l'heure en cours. */
export interface SourceReading {
  source: WeatherSource
  temperature: number
  /** Précipitations sur l'heure (mm). */
  precipitation: number
  /** Vent moyen (km/h). */
  windSpeed: number
}

export interface Provider {
  id: string
  institution: string
  /** Plateformes où l'appel est légitime et techniquement possible. */
  platforms: readonly Platform[]
  /** Mention à afficher dès qu'une de ses sources est utilisée. */
  attribution: string
  fetch: (parcelle: ProviderQuery, signal?: AbortSignal) => Promise<SourceReading[]>
}

/** Ce dont un fournisseur a besoin : un point sur la carte. */
export interface ProviderQuery {
  latitude: number
  longitude: number
}

/** Identifie l'application auprès des services qui l'exigent. */
export const USER_AGENT = 'Klima/1.0 (météo agricole; https://maxlestage.github.io/meteo/)'
