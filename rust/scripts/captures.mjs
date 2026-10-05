// Les captures de la galerie du site, refaites quand l'interface change.
//
// Elles montrent l'application web, servie par un relais local, sur un jeu
// de données qui raconte quelque chose — un après-midi parisien, une averse
// vers 17 h, du soleil, des graminées — plutôt que la météo du jour de la
// capture. Les appels aux fournisseurs sont interceptés : rien ne sort.
//
//   cd rust && bash scripts/construire.sh && KLIMA_PUBLIC=../public cargo run -p klima-relay
//   npm i playwright-core && CHROMIUM=/chemin/vers/chrome node scripts/captures.mjs
import { chromium } from 'playwright-core'

// Un après-midi parisien qui raconte quelque chose : sec maintenant, une
// averse vers 17 h, du soleil modéré, des graminées dans l'air.
const OFFSET = 7200
const nowLocal = Date.now() + OFFSET * 1000
const jour = Math.floor(nowLocal / 86400000) * 86400000
// On cale « maintenant » à 14 h 10, heure de la ville, pour que l'histoire
// soit la même quel que soit le moment où l'on capture.
const decalage = jour + 14 * 3600000 + 10 * 60000 - nowLocal
const pad = (n) => String(n).padStart(2, '0')
const stamp = (ms) => { const d = new Date(ms); return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}T${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}` }
const dayStamp = (ms) => stamp(ms).slice(0, 10)

// Le navigateur doit croire qu'il est 14 h 10 à Paris.
const fauxMaintenant = Date.now() + decalage

const N = 72
const h = (i) => jour + i * 3600000
const cur = 14
const pluie = { 17: [1.4, 84], 18: [2.1, 78], 16: [0.0, 46], 19: [0.3, 52] }
const hourly = {
  time: Array.from({ length: N }, (_, i) => stamp(h(i))),
  temperature_2m: Array.from({ length: N }, (_, i) => +(15 + 6 * Math.sin(((i % 24) - 9) / 24 * 2 * Math.PI)).toFixed(1)),
  apparent_temperature: Array.from({ length: N }, (_, i) => +(14 + 6 * Math.sin(((i % 24) - 9) / 24 * 2 * Math.PI)).toFixed(1)),
  weather_code: Array.from({ length: N }, (_, i) => (pluie[i] && pluie[i][0] > 0 ? 61 : i === 16 ? 3 : (i % 24) < 7 || (i % 24) > 20 ? 1 : 2)),
  is_day: Array.from({ length: N }, (_, i) => ((i % 24) >= 8 && (i % 24) <= 19 ? 1 : 0)),
  precipitation_probability: Array.from({ length: N }, (_, i) => (pluie[i] ? pluie[i][1] : i > 20 && i < 30 ? 20 : 8)),
  relative_humidity_2m: Array.from({ length: N }, () => 64),
  dew_point_2m: Array.from({ length: N }, () => 11),
  precipitation: Array.from({ length: N }, (_, i) => (pluie[i] ? pluie[i][0] : 0)),
  wind_speed_10m: Array.from({ length: N }, () => 14),
  wind_gusts_10m: Array.from({ length: N }, (_, i) => (i >= 16 && i <= 18 ? 38 : 24)),
  uv_index: Array.from({ length: N }, (_, i) => { const x = i % 24; return x >= 9 && x <= 18 ? +(5 * Math.sin((x - 8) / 11 * Math.PI)).toFixed(1) : 0 }),
}
const daily = {
  time: Array.from({ length: 7 }, (_, i) => dayStamp(jour + i * 86400000)),
  weather_code: [61, 2, 3, 80, 1, 2, 61],
  sunrise: Array.from({ length: 7 }, (_, i) => stamp(jour + i * 86400000 + 7 * 3600000 + 52 * 60000)),
  sunset: Array.from({ length: 7 }, (_, i) => stamp(jour + i * 86400000 + 19 * 3600000 + 18 * 60000)),
  temperature_2m_min: [10, 11, 9, 12, 8, 10, 11],
  temperature_2m_max: [21, 22, 19, 18, 20, 23, 17],
  precipitation_sum: [3.8, 0, 0.4, 6.2, 0, 0, 2.4],
  precipitation_probability_max: [84, 10, 30, 90, 5, 10, 70],
  wind_gusts_10m_max: [38, 22, 30, 52, 18, 20, 34],
  uv_index_max: [5, 6, 4, 2, 6, 6, 3],
}
const forecast = {
  latitude: 48.86, longitude: 2.34, timezone: 'Europe/Paris', utc_offset_seconds: OFFSET, elevation: 42,
  current: { time: stamp(jour + cur * 3600000 + 10 * 60000), temperature_2m: 19.4, apparent_temperature: 18.6, relative_humidity_2m: 58, weather_code: 2, is_day: 1, wind_speed_10m: 13, wind_gusts_10m: 24, pressure_msl: 1016 },
  hourly, daily,
}
const modeles = { meteofrance_seamless: 19.2, ecmwf_ifs025: 18.6, icon_seamless: 19.8, gfs_seamless: 21.4 }
const multi = { timezone: 'Europe/Paris', utc_offset_seconds: OFFSET, hourly: { time: hourly.time.slice(0, 24) } }
for (const [m, t] of Object.entries(modeles)) {
  multi.hourly[`temperature_2m_${m}`] = Array(24).fill(t)
  multi.hourly[`precipitation_${m}`] = Array(24).fill(m === 'gfs_seamless' ? 0.2 : 0)
  multi.hourly[`wind_speed_10m_${m}`] = Array(24).fill(13)
}
const iso = (ms) => new Date(ms).toISOString().replace('.000', '')
const met = { properties: { timeseries: Array.from({ length: 6 }, (_, i) => ({ time: iso(fauxMaintenant - 3600000 + i * 3600000), data: { instant: { details: { air_temperature: 18.9, wind_speed: 3.8 } }, next_1_hours: { details: { precipitation_amount: 0 } } } })) } }
const bright = { weather: { temperature: 19.6, precipitation: 0, wind_speed: 12 } }
const air = { current: { time: stamp(jour + cur * 3600000), european_aqi: 32, pm2_5: 8.4, pm10: 16, nitrogen_dioxide: 22, ozone: 70, alder_pollen: 0, birch_pollen: 3.2, grass_pollen: 41, mugwort_pollen: 1.2, olive_pollen: 0, ragweed_pollen: 0.4 } }

const browser = await chromium.launch(process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {})
const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, locale: 'fr-FR', deviceScaleFactor: 2, timezoneId: 'Europe/Paris' })
await ctx.addInitScript(`{ const d = ${decalage}; const N = Date.now; Date.now = () => N() + d; const O = Date; globalThis.Date = class extends O { constructor(...a) { a.length ? super(...a) : super(N() + d) } static now() { return N() + d } }; }`)
const page = await ctx.newPage()
await page.route('**/v1/open-meteo/forecast**', (r) => r.fulfill({ json: r.request().url().includes('models=') ? multi : forecast }))
await page.route('**/v1/met-norway/compact**', (r) => r.fulfill({ json: met }))
await page.route('**/v1/bright-sky/current**', (r) => r.fulfill({ json: bright }))
await page.route('**/v1/open-meteo/air-quality**', (r) => r.fulfill({ json: air }))
await page.goto(`${process.env.KLIMA_LOCAL ?? 'http://localhost:8787'}/app/?parcelle=Paris&lat=48.8566&lon=2.3522`, { waitUntil: 'networkidle' })
await page.waitForTimeout(3000)
const sortie = new URL('../klima-site/public/captures', import.meta.url).pathname
const fs = await import('node:fs'); fs.mkdirSync(sortie, { recursive: true })
// 1. L'écran d'accueil : la ville, la température, la pluie qui vient.
await page.screenshot({ path: `${sortie}/accueil.jpg`, type: 'jpeg', quality: 84 })
// 2. Les tuiles : ressenti, UV, air, pollens.
await page.locator('.tiles').scrollIntoViewIfNeeded()
await page.waitForTimeout(600)
const tuiles = await page.locator('.tiles').boundingBox()
await page.screenshot({ path: `${sortie}/tuiles.jpg`, type: 'jpeg', quality: 84, fullPage: true,
  clip: { x: 0, y: tuiles.y + (await page.evaluate(() => window.scrollY)) - 24, width: 390, height: 844 } })
// 3. Ce que dit chaque source.
const sources = page.locator('section.card[aria-label="Ce que dit chaque source"]')
await sources.scrollIntoViewIfNeeded()
await page.waitForTimeout(600)
const box = await sources.boundingBox()
await page.screenshot({ path: `${sortie}/sources.jpg`, type: 'jpeg', quality: 84, fullPage: true,
  clip: { x: 0, y: box.y + (await page.evaluate(() => window.scrollY)) - 160, width: 390, height: 844 } })
await browser.close()
