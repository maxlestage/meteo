import { LANGUAGE_NAMES, LANGUAGES } from '@klima/core'
import { useI18n } from '@klima/core/ui'

/**
 * Pied de page : ce que le produit couvre, d'où viennent ses chiffres, qui l'a
 * fait, et ce qu'il ne prétend pas remplacer.
 */
export function Footer() {
  const { t, language, setLanguage } = useI18n()
  const year = new Date().getFullYear()

  return (
    <footer className="footer">
      <div className="footer__grid">
        <div className="footer__brand">
          <p className="footer__wordmark">Klima</p>
          <p className="footer__tagline">{t('footer.tagline')}</p>
          <ul className="footer__languages">
            {LANGUAGES.map((candidate) => (
              <li key={candidate}>
                <button
                  type="button"
                  className={candidate === language ? 'is-current' : undefined}
                  aria-current={candidate === language}
                  onClick={() => setLanguage(candidate)}
                >
                  {LANGUAGE_NAMES[candidate]}
                </button>
              </li>
            ))}
          </ul>
        </div>

        <nav className="footer__column" aria-label={t('footer.product')}>
          <h2>{t('footer.product')}</h2>
          <ul>
            <li><a href="#aujourdhui">{t('nav.today')}</a></li>
            <li><a href="#indicateurs">{t('nav.indicators')}</a></li>
            <li><a href="#sources">{t('nav.sources')}</a></li>
            <li><a href="#donnees">{t('nav.data')}</a></li>
          </ul>
        </nav>

        <div className="footer__column">
          <h2>{t('footer.dataTitle')}</h2>
          <ul>
            <li>
              <a href="https://open-meteo.com/" rel="noreferrer noopener" target="_blank">
                Open-Meteo
              </a>
            </li>
            <li>
              <a href="https://api.met.no/" rel="noreferrer noopener" target="_blank">
                MET Norway
              </a>
            </li>
            <li>
              <a href="https://brightsky.dev/" rel="noreferrer noopener" target="_blank">
                Bright Sky / DWD
              </a>
            </li>
            <li>{t('footer.models')}</li>
            <li>{t('footer.method')}</li>
          </ul>
        </div>

        <div className="footer__column footer__column--credits">
          <h2>{t('footer.credits')}</h2>
          <p className="footer__author">{t('footer.author')}</p>
          <p className="footer__role">{t('footer.role')}</p>
        </div>
      </div>

      <div className="footer__bottom">
        <p>{t('footer.rights', { year })}</p>
        <p className="footer__legal">{t('footer.legal')}</p>
      </div>
    </footer>
  )
}
