import { useState } from 'react'
import { AgroThresholds, type Parcelle } from '@klima/core'
import { Features } from './components/Features'
import { PhoneMockup } from './components/PhoneMockup'
import { TodaySection } from './components/TodaySection'
import { useDayDigest } from './hooks/useDayDigest'

/** Plaine céréalière de Beauce, au premier chargement. */
const DEFAULT_PARCELLE: Parcelle = {
  name: 'Chartres',
  latitude: 48.4468,
  longitude: 1.4892,
  admin: 'Eure-et-Loir',
  country: 'France',
}

export default function App() {
  const [parcelle, setParcelle] = useState<Parcelle>(DEFAULT_PARCELLE)
  const { digest, loading, error, reload, forecast } = useDayDigest(parcelle)

  return (
    <>
      <header className="nav">
        <a className="nav__brand" href="#top">
          Klima
        </a>
        <nav>
          <a href="#aujourdhui">Météo du jour</a>
          <a href="#indicateurs">Indicateurs</a>
          <a href="#donnees">Données</a>
        </nav>
      </header>

      <main id="top">
        <section className="hero">
          <div className="hero__text">
            <p className="hero__eyebrow">Application iOS et web</p>
            <h1>La météo qui parle agronomie.</h1>
            <p className="hero__lead">
              Klima lit la météo comme un agronome : humidité et température du sol,
              évapotranspiration de référence, fenêtres de pulvérisation, degrés-jours. Les
              variables agricoles brutes, traduites en décisions pour la parcelle.
            </p>
            <div className="hero__actions">
              <a className="button" href="#aujourdhui">
                Voir la météo du jour
              </a>
              <a className="button button--ghost" href="#indicateurs">
                Ce que Klima calcule
              </a>
            </div>
            <p className="hero__note">
              Données Open-Meteo, sans compte ni clé d'API. Gratuit, sans publicité.
            </p>
          </div>

          <PhoneMockup parcelle={parcelle} digest={digest} current={forecast?.current ?? null} />
        </section>

        <TodaySection
          parcelle={parcelle}
          digest={digest}
          current={forecast?.current ?? null}
          loading={loading}
          error={error}
          onSelect={setParcelle}
          onRetry={reload}
          timeZone={forecast?.timezone ?? 'Europe/Paris'}
        />

        <Features />

        <section className="data" id="donnees">
          <div className="section-head">
            <h2>D'où viennent les chiffres</h2>
          </div>
          <div className="data__grid">
            <article>
              <h3>Un modèle agricole, pas une météo grand public</h3>
              <p>
                Klima n'interroge que les variables agronomiques de l'API Open-Meteo : température
                et humidité du sol par couche, évapotranspiration de référence FAO-56, déficit de
                pression de vapeur — auxquelles s'ajoutent le vent, la pluie et l'hygrométrie
                nécessaires aux fenêtres de traitement.
              </p>
            </article>
            <article>
              <h3>Les mêmes règles sur les deux plateformes</h3>
              <p>
                Les seuils vivent au même endroit pour le web et pour iOS, et les deux suites de
                tests couvrent les mêmes cas. Le conseil rendu au champ est identique, quel que
                soit l'écran par lequel on le lit.
              </p>
            </article>
            <article>
              <h3>Le vent fait loi</h3>
              <p>
                Un vent au-delà de {AgroThresholds.sprayWindMax} km/h, des rafales fortes ou une pluie imminente rendent
                l'heure inexploitable pour un traitement, quel que soit le reste des conditions.
                Klima ne propose jamais une fenêtre hors des clous.
              </p>
            </article>
          </div>
        </section>
      </main>

      <footer className="footer">
        <p>
          Klima — météo agricole. Données <a href="https://open-meteo.com/">Open-Meteo</a>, sans
          clé d'API.
        </p>
        <p className="footer__note">
          Les indicateurs sont des aides à la décision ; ils ne remplacent ni l'observation de la
          parcelle ni la réglementation en vigueur.
        </p>
      </footer>
    </>
  )
}
