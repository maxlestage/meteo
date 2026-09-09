import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react'
import {
  detectLanguage,
  isLanguage,
  LOCALES,
  translator,
  type Catalog,
  type Language,
  type Translate,
} from '../i18n'
import { formats, type Formats } from '../format'

interface I18nValue {
  language: Language
  setLanguage: (language: Language) => void
  /** Traduit une clé, en interpolant ses paramètres. */
  t: Translate
  /** Nombres, unités et pourcentages dans la langue courante. */
  f: Formats
  /** Locale à passer aux API `Intl` (dates, listes). */
  locale: string
}

const I18nContext = createContext<I18nValue | null>(null)

const STORAGE_KEY = 'klima.language'

interface Props {
  /** Catalogues consultés dans l'ordre : le partagé, puis celui de l'application. */
  catalogs: readonly Record<Language, Catalog>[]
  children: ReactNode
}

/**
 * Langue de l'interface : celle choisie à la visite précédente, sinon celle du
 * navigateur, sinon la langue de référence.
 */
export function I18nProvider({ catalogs, children }: Props) {
  const [language, setLanguageState] = useState<Language>(() => stored() ?? fromBrowser())

  useEffect(() => {
    document.documentElement.lang = language
    try {
      localStorage.setItem(STORAGE_KEY, language)
    } catch {
      // Navigation privée : la langue tient le temps de la visite.
    }
  }, [language])

  const setLanguage = useCallback((next: Language) => setLanguageState(next), [])

  const value = useMemo<I18nValue>(
    () => ({
      language,
      setLanguage,
      t: translator(language, ...catalogs),
      f: formats(language),
      locale: LOCALES[language],
    }),
    [language, setLanguage, catalogs],
  )

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>
}

export function useI18n(): I18nValue {
  const value = useContext(I18nContext)
  if (!value) throw new Error('useI18n doit être appelé sous un I18nProvider')
  return value
}

function stored(): Language | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return isLanguage(raw) ? raw : null
  } catch {
    return null
  }
}

function fromBrowser(): Language {
  if (typeof navigator === 'undefined') return detectLanguage([])
  return detectLanguage(navigator.languages ?? [navigator.language])
}
