import { FEATURES, PLAN_LIMITS, type Feature } from '@klima/core'
import { useI18n } from '@klima/core/ui'

/**
 * Ce que l'abonnement ajoute, et où.
 *
 * L'application web n'a pas de boutique : elle ne peut pas vendre, et lui
 * greffer des comptes pour encaisser un euro par mois coûterait plus que ça ne
 * rapporterait. Elle dit donc simplement ce qui existe ailleurs.
 *
 * La liste vient de `FEATURES`, dans le cœur partagé : une fonction ajoutée au
 * palier payant apparaît ici sans qu'on y touche, et ne peut pas être oubliée.
 */

/**
 * Ce que le web donne malgré tout.
 *
 * Le recoupement des instituts se paie dans l'application, mais reste offert
 * ici : c'est l'argument qui vend le produit, et le montrer à l'œuvre convainc
 * mieux que le décrire. L'annoncer comme payant à quelqu'un qui l'a sous les
 * yeux serait au mieux confus, au pire malhonnête.
 *
 * Les trois autres fonctions supposent un appareil qui prévient, se souvient
 * et exporte : le web ne les aurait de toute façon pas.
 */
const OFFERT_SUR_LE_WEB: readonly Feature[] = ['recoupement']

export function ProNote() {
  const { t } = useI18n()
  const missing = FEATURES.filter((feature) => !OFFERT_SUR_LE_WEB.includes(feature))

  return (
    <section className="pro" aria-labelledby="pro-title">
      <h2 id="pro-title">{t('plan.pro')}</h2>
      <p className="pro__lead">{t('pro.lead')}</p>

      <ul className="pro__features">
        {missing.map((feature) => (
          <li key={feature}>{t(`plan.feature.${feature}`)}</li>
        ))}
      </ul>

      <p className="pro__free">
        {t('pro.free', { parcelles: PLAN_LIMITS.libre.parcelles, jours: PLAN_LIMITS.libre.jours })}
      </p>
    </section>
  )
}
