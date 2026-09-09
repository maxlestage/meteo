import { LANGUAGES, LANGUAGE_NAMES } from '../i18n'
import { useI18n } from './i18n'

interface Props {
  /** Le style appartient à l'application ; seul le comportement est partagé. */
  className?: string
  label: string
}

/** Sélecteur de langue : trois boutons, celui de la langue courante marqué. */
export function LanguageSwitcher({ className = 'lang', label }: Props) {
  const { language, setLanguage } = useI18n()

  return (
    <div className={className} role="group" aria-label={label}>
      {LANGUAGES.map((candidate) => (
        <button
          key={candidate}
          type="button"
          className={candidate === language ? 'is-current' : undefined}
          aria-current={candidate === language}
          onClick={() => setLanguage(candidate)}
          title={LANGUAGE_NAMES[candidate]}
        >
          {candidate.toUpperCase()}
        </button>
      ))}
    </div>
  )
}
