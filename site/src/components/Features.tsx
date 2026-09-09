import { AgroThresholds } from '@klima/core'
import { useI18n } from '@klima/core/ui'

/**
 * Ce que Klima calcule. Les seuils cités viennent du code et sont mis en forme
 * dans la langue courante : la page ne peut pas annoncer autre chose que ce que
 * l'application applique.
 */
export function Features() {
  const { t, f } = useI18n()
  const kmh = (value: number) => f.unit(value, 'km/h', 0)
  const celsius = (value: number) => f.unit(value, '°C', 0)

  const features = [
    {
      key: 'water',
      detail: { deficit: f.unit(Math.abs(AgroThresholds.irrigationDeficit), 'mm', 0) },
    },
    {
      key: 'soil',
      detail: {
        wet: f.percent(AgroThresholds.soilTooWet * 100),
        dry: f.percent(AgroThresholds.soilTooDry * 100),
      },
    },
    {
      key: 'spray',
      rule: {
        min: kmh(AgroThresholds.sprayWindMin),
        max: kmh(AgroThresholds.sprayWindMax),
        gusts: kmh(AgroThresholds.sprayGustMax),
      },
      detail: {
        tempMin: celsius(AgroThresholds.sprayTempMin),
        tempMax: celsius(AgroThresholds.sprayTempMax),
        vpd: f.unit(AgroThresholds.sprayVpdMax, 'kPa'),
      },
    },
    {
      key: 'disease',
      rule: { humidity: f.percent(AgroThresholds.leafWetnessHumidity) },
      detail: {
        min: celsius(AgroThresholds.diseaseTempMin),
        max: celsius(AgroThresholds.diseaseTempMax),
      },
    },
    { key: 'frost' },
    {
      key: 'gdd',
      rule: { base: celsius(AgroThresholds.gddBase) },
      detail: { ceiling: celsius(AgroThresholds.gddCeiling) },
    },
  ] as const

  return (
    <section className="features" id="indicateurs">
      <div className="section-head">
        <h2>{t('features.title')}</h2>
        <p>{t('features.lead')}</p>
      </div>

      <div className="features__grid">
        {features.map((feature) => (
          <article className="feature" key={feature.key}>
            <h3>{t(`feature.${feature.key}.title`)}</h3>
            <p className="feature__rule">
              {t(`feature.${feature.key}.rule`, 'rule' in feature ? feature.rule : undefined)}
            </p>
            <p className="feature__detail">
              {t(`feature.${feature.key}.detail`, 'detail' in feature ? feature.detail : undefined)}
            </p>
          </article>
        ))}
      </div>
    </section>
  )
}
