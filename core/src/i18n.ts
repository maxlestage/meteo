/**
 * Langues de Klima et traduction des libellés partagés.
 *
 * Le domaine ne fabrique jamais de phrase : il renvoie des états et des motifs
 * structurés, que l'interface traduit. Les catalogues d'ici couvrent ce qui est
 * commun à l'application web et au site ; chaque application garde le sien pour
 * ses propres textes. L'équivalent iOS vit dans les catalogues de chaînes du
 * projet Xcode.
 */

export type Language = 'fr' | 'en' | 'es'

export const LANGUAGES: readonly Language[] = ['fr', 'en', 'es']

/** Langue de référence : celle dans laquelle les textes sont écrits d'abord. */
export const REFERENCE_LANGUAGE: Language = 'fr'

export const LANGUAGE_NAMES: Record<Language, string> = {
  fr: 'Français',
  en: 'English',
  es: 'Español',
}

/** Locale à passer aux API `Intl` pour chaque langue. */
export const LOCALES: Record<Language, string> = {
  fr: 'fr-FR',
  en: 'en-GB',
  es: 'es-ES',
}

export type Params = Record<string, string | number>
export type Catalog = Record<string, string>
export type Translate = (key: string, params?: Params) => string

/**
 * Première langue reconnue parmi celles que propose le navigateur
 * (« fr-BE » compte comme « fr »). Faute de correspondance, la référence.
 */
export function detectLanguage(candidates: readonly string[] = []): Language {
  for (const candidate of candidates) {
    const base = candidate.toLowerCase().split('-')[0]
    const match = LANGUAGES.find((language) => language === base)
    if (match) return match
  }
  return REFERENCE_LANGUAGE
}

export function isLanguage(value: unknown): value is Language {
  return typeof value === 'string' && (LANGUAGES as readonly string[]).includes(value)
}

/** Remplace les `{jetons}` d'un modèle par leurs valeurs. */
export function interpolate(template: string, params?: Params): string {
  if (!params) return template
  return template.replace(/\{(\w+)\}/g, (whole, name: string) =>
    name in params ? String(params[name]) : whole,
  )
}

/**
 * Fonction de traduction lisant les catalogues dans l'ordre donné. Une clé
 * absente retombe sur la langue de référence, puis sur la clé elle-même — un
 * texte manquant se voit, il ne disparaît pas.
 */
export function translator(
  language: Language,
  ...catalogs: readonly Record<Language, Catalog>[]
): Translate {
  return (key, params) => {
    for (const catalog of catalogs) {
      const value = catalog[language]?.[key] ?? catalog[REFERENCE_LANGUAGE]?.[key]
      if (value !== undefined) return interpolate(value, params)
    }
    return key
  }
}
