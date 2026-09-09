/**
 * Pièces d'interface partagées entre l'application web et le site de
 * présentation. Séparées du domaine : importer les règles agronomiques ne doit
 * pas entraîner React.
 */
export { WeatherIcon } from './WeatherIcon'
export { I18nProvider, useI18n } from './i18n'
export { LanguageSwitcher } from './LanguageSwitcher'
