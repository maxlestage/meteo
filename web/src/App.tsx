import { DailyList } from './components/DailyList'
import { DetailTile } from './components/DetailTile'
import { Hero } from './components/Hero'
import { HourlyStrip } from './components/HourlyStrip'
import { ParcelleSearch } from './components/ParcelleSearch'
import { ProNote } from './components/ProNote'
import { SprayCard } from './components/SprayCard'
import { AgroThresholds } from '@klima/core'
import { BrandLockup, LanguageSwitcher, useI18n } from '@klima/core/ui'
import { useAgroForecast } from './hooks/useAgroForecast'
import { useParcelle } from './hooks/useParcelle'

export default function App() {
  const { t, f, locale } = useI18n()
  const [parcelle, setParcelle] = useParcelle()
  const { forecast, summary, consensus, loading, error, reload } = useAgroForecast(parcelle)

  const sky = forecast ? skyFor(forecast.current.isDay, forecast.current.weatherCode) : 'night'

  return (
    <div className={`sky sky--${sky}`}>
      <div className="shell">
        <header className="topbar">
          {/*
            Le signe ramène à la vitrine. Sans lui l'application est un
            cul-de-sac : on y arrive par un lien partagé, un favori ou l'icône
            de l'écran d'accueil — sans historique à remonter — et installée
            sur l'écran d'accueil, il n'y a même plus de barre de navigation.
            Le chemin est relatif : l'application est publiée sous /app/ de la
            vitrine.
          */}
          <a className="topbar__home" href="../" aria-label={t('app.home')}>
            <BrandLockup size={26} />
          </a>
          <LanguageSwitcher className="lang" label={t('language.label')} />
        </header>

        <ParcelleSearch current={parcelle} onSelect={setParcelle} />

        {loading && !forecast && <p className="state">{t('app.loading')}</p>}

        {error && (
          <div className="state state--error" role="alert">
            <p>{error}</p>
            <button type="button" className="button" onClick={reload}>
              {t('app.retry')}
            </button>
          </div>
        )}

        {forecast && summary && (
          <>
            <Hero forecast={forecast} />
            <HourlyStrip
              hours={forecast.hourly}
              current={forecast.current}
              timeZone={forecast.timezone}
            />
            <DailyList
              days={forecast.daily}
              currentTemperature={forecast.current.temperature}
              timeZone={forecast.timezone}
            />
            <SprayCard
              hours={forecast.hourly}
              nextSpray={summary.nextSpray}
              timeZone={forecast.timezone}
            />

            <div className="tiles">
              <DetailTile
                label={t('tile.soil')}
                value={t(`soil.${summary.soil.state}`)}
                caption={t('tile.soil.caption', {
                  moisture: f.percent(summary.soil.moisture * 100),
                  temperature: f.unit(summary.soil.temperature, '°C'),
                  state: t(summary.soil.trafficable ? 'soil.trafficable' : 'soil.compaction'),
                })}
                gauge={{
                  position: summary.soil.moisture / 0.5,
                  gradient:
                    'linear-gradient(to right, #d8b36a 0%, #8fc46a 30%, #4aa3d8 70%, #2b5f9e 100%)',
                }}
              />

              <DetailTile
                label={t('tile.water')}
                value={f.signedUnit(summary.water.balance, 'mm')}
                caption={
                  summary.water.irrigationAdvice > 0
                    ? t('tile.water.irrigation', {
                        amount: f.unit(summary.water.irrigationAdvice, 'mm', 0),
                      })
                    : t('tile.water.caption', {
                        rain: f.unit(summary.water.precipitation, 'mm'),
                        et0: f.unit(summary.water.evapotranspiration, 'mm'),
                      })
                }
              />

              <DetailTile
                label={t('tile.wind')}
                value={f.unit(forecast.current.windSpeed, 'km/h', 0)}
                caption={t('tile.wind.caption', {
                  gusts: f.unit(forecast.current.windGusts, 'km/h', 0),
                  limit: f.unit(AgroThresholds.sprayWindMax, 'km/h', 0),
                })}
              />

              <DetailTile
                label={t('tile.frost')}
                value={t(`frost.${summary.frost.severity}`)}
                caption={t('tile.frost.caption', {
                  temperature: f.unit(summary.frost.minTemperature, '°C'),
                  hoarFrost: summary.frost.hoarFrost ? `, ${t('frost.hoarFrost')}` : '',
                })}
              />

              <DetailTile
                label={t('tile.disease')}
                value={t(`disease.${summary.disease.level}`)}
                caption={t('tile.disease.caption', { hours: summary.disease.leafWetnessHours })}
              />

              <DetailTile
                label={t('tile.gdd')}
                value={f.unit(summary.gdd, '°C·j')}
                caption={t('tile.gdd.caption', { base: f.unit(AgroThresholds.gddBase, '°C', 0) })}
              />

              <DetailTile
                label={t('tile.sunrise')}
                value={time(forecast.daily[0]?.sunrise, forecast.timezone, locale)}
                caption={t('tile.sunrise.caption', {
                  time: time(forecast.daily[0]?.sunset, forecast.timezone, locale),
                })}
              />

              {consensus && (
                <DetailTile
                  label={t('consensus.title')}
                  value={t(`consensus.${consensus.agreement}`)}
                  caption={`${t('consensus.detail', {
                    count: consensus.readings.length,
                    spread: f.unit(consensus.temperature.spread, '°C'),
                  })}${consensus.agreeOnRain ? '' : ` · ${t('consensus.rainDisagreement')}`}. ${t(
                    'consensus.median',
                    { value: f.unit(consensus.temperature.median, '°C') },
                  )}.`}
                  gauge={{
                    position: 1 - Math.min(consensus.temperature.spread / 5, 1),
                    gradient: 'linear-gradient(to right, #ef8a5a, #f0c14b, #7ed07a)',
                  }}
                />
              )}

              <DetailTile
                label={t('tile.sowing')}
                value={t(summary.soil.sowable ? 'tile.sowing.yes' : 'tile.sowing.no')}
                caption={t('tile.sowing.caption', {
                  temperature: f.unit(summary.soil.temperature, '°C'),
                })}
              />
            </div>

            <ProNote />

            <footer className="footer">
              <p>{t('app.source')}.</p>
              <button type="button" className="button" onClick={reload}>
                {t('app.refresh')}
              </button>
            </footer>
          </>
        )}
      </div>
    </div>
  )
}

/** Le fond suit le ciel : nuit, journée couverte ou journée dégagée. */
function skyFor(isDay: boolean, code: number): 'night' | 'grey' | 'day' {
  if (!isDay) return 'night'
  return code >= 45 ? 'grey' : 'day'
}

function time(date: Date | null | undefined, timeZone: string, locale: string): string {
  if (!date) return '—'
  return new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit', timeZone }).format(date)
}
