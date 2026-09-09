/**
 * Cœur partagé de Klima : les règles agronomiques, la traduction des codes
 * météo et le client de l'API agricole Open-Meteo.
 *
 * L'application web (`web/`) et le site de présentation (`site/`) consomment ce
 * paquet ; l'application iOS en implémente l'équivalent en Swift. Les seuils
 * n'existent donc qu'à deux endroits : `AgroThresholds` ici et son homologue
 * dans `ios/Klima/Models/AgroIndicators.swift`.
 */
export * from './agro'
export * from './weather'
export * from './openMeteo'
export * from './today'
export * from './format'
export * from './i18n'
export * from './messages'
export * from './providers'
export * from './consensus'
