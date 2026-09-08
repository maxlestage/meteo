import { AgroThresholds, decimal, percent } from '@klima/core'

/**
 * Ce que Klima calcule. Les seuils cités viennent du code : la page ne peut pas
 * annoncer autre chose que ce que l'application applique.
 */
const FEATURES = [
  {
    title: 'Bilan hydrique',
    rule: 'Pluie moins évapotranspiration de référence, cumulées sur sept jours.',
    detail: `Passé un déficit de ${Math.abs(AgroThresholds.irrigationDeficit)} mm sur la période, Klima chiffre l'irrigation à apporter.`,
  },
  {
    title: 'État du sol',
    rule: 'Humidité volumique entre 3 et 9 cm, et température à 6 cm.',
    detail: `Saturé au-delà de ${percent(AgroThresholds.soilTooWet * 100)} vol., sec en deçà de ${percent(
      AgroThresholds.soilTooDry * 100,
    )}. Portance et aptitude au semis en découlent.`,
  },
  {
    title: 'Fenêtre de traitement',
    rule: `Vent entre ${AgroThresholds.sprayWindMin} et ${AgroThresholds.sprayWindMax} km/h, rafales sous ${AgroThresholds.sprayGustMax} km/h, pas de pluie dans les deux heures.`,
    detail: `S'y ajoutent la température (${AgroThresholds.sprayTempMin} à ${AgroThresholds.sprayTempMax} °C), l'hygrométrie et un déficit de pression de vapeur sous ${decimal(AgroThresholds.sprayVpdMax)} kPa.`,
  },
  {
    title: 'Pression maladie',
    rule: `Heures d'humectation du feuillage : hygrométrie au-dessus de ${percent(AgroThresholds.leafWetnessHumidity)}.`,
    detail: `Comptées dans la plage de température favorable au champignon, ${AgroThresholds.diseaseTempMin} à ${AgroThresholds.diseaseTempMax} °C.`,
  },
  {
    title: 'Risque de gel',
    rule: 'Température minimale de la nuit et point de rosée.',
    detail: 'Distingue la gelée blanche du gel sévère, jusqu’à −4 °C et au-delà.',
  },
  {
    title: 'Degrés-jours',
    rule: `Moyenne plafonnée, base ${AgroThresholds.gddBase} °C.`,
    detail: `La journée ne capitalise plus au-delà de ${AgroThresholds.gddCeiling} °C.`,
  },
]

export function Features() {
  return (
    <section className="features" id="indicateurs">
      <div className="section-head">
        <h2>Six indicateurs, une seule question</h2>
        <p>
          Est-ce que je peux y aller aujourd'hui ? Klima part des variables agricoles brutes et
          rend une réponse, pas un tableau de chiffres.
        </p>
      </div>

      <div className="features__grid">
        {FEATURES.map((feature) => (
          <article className="feature" key={feature.title}>
            <h3>{feature.title}</h3>
            <p className="feature__rule">{feature.rule}</p>
            <p className="feature__detail">{feature.detail}</p>
          </article>
        ))}
      </div>
    </section>
  )
}
