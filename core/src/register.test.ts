import { describe, expect, test } from 'bun:test'
import type { HourlySample } from './agro'
import { recordAt, REGISTER_COLUMNS, toCsv } from './register'

const START = new Date('2026-04-15T08:00:00')

function hour(offset: number, over: Partial<HourlySample> = {}): HourlySample {
  return {
    time: new Date(START.getTime() + offset * 3_600_000),
    weatherCode: 3,
    isDay: true,
    precipitationProbability: 10,
    temperature: 17.4,
    relativeHumidity: 62,
    dewPoint: 9,
    precipitation: 0,
    windSpeed: 11.2,
    windGusts: 18.5,
    soilTemperature6cm: 12,
    soilMoisture3to9cm: 0.22,
    et0: 0.1,
    vapourPressureDeficit: 0.6,
    ...over,
  }
}

const series = [hour(0), hour(1), hour(2), hour(3)]

describe('relevé d’un traitement', () => {
  test('prend les conditions de l’heure qui contient le moment donné', () => {
    const record = recordAt(series, new Date('2026-04-15T09:37:00'), 'Le Clos')!

    expect(record.at).toEqual(new Date('2026-04-15T09:00:00'))
    expect(record.parcelle).toBe('Le Clos')
    expect(record.windSpeed).toBe(11.2)
    expect(record.relativeHumidity).toBe(62)
  })

  test('porte le verdict de Klima, à titre indicatif', () => {
    const record = recordAt(series, new Date('2026-04-15T09:00:00'), 'Le Clos')!
    expect(['favorable', 'acceptable', 'defavorable']).toContain(record.verdict)
    expect(record.score).toBeGreaterThanOrEqual(0)
  })

  test('le produit vient de l’exploitant, jamais de Klima', () => {
    expect(recordAt(series, new Date('2026-04-15T09:00:00'), 'Le Clos')!.product).toBeUndefined()
    expect(recordAt(series, new Date('2026-04-15T09:00:00'), 'Le Clos', 'Cuivre')!.product)
      .toBe('Cuivre')
  })

  test('hors de la série, pas de ligne plutôt qu’une ligne inventée', () => {
    // Un document qu'on pourra vous opposer ne se remplit pas au jugé.
    expect(recordAt(series, new Date('2026-04-20T09:00:00'), 'Le Clos')).toBeNull()
  })
})

describe('export CSV', () => {
  const record = recordAt(series, new Date('2026-04-15T09:00:00'), 'Le Clos', 'Cuivre')!

  test('sépare par point-virgule et décime à la virgule', () => {
    // Excel en français lit un fichier à virgules comme une seule colonne.
    const csv = toCsv([record])
    const line = csv.split('\r\n')[1]!

    expect(line).toContain(';')
    expect(line).toContain('11,2')
    expect(line).not.toContain('11.2')
  })

  test('commence par la marque d’ordre des octets', () => {
    // Sans elle, Excel lit l'UTF-8 comme du Latin-1.
    expect(toCsv([record]).startsWith('﻿')).toBe(true)
  })

  test('une parcelle qui contient le séparateur ne casse pas la colonne suivante', () => {
    const piege = { ...record, parcelle: 'Le Clos ; bas' }
    const line = toCsv([piege]).split('\r\n')[1]!

    expect(line).toContain('"Le Clos ; bas"')
    // Onze colonnes malgré le point-virgule dans le nom.
    expect(line.split(';').length).toBeGreaterThan(REGISTER_COLUMNS.length)
    expect(line.match(/"/g)).toHaveLength(2)
  })

  test('un guillemet dans un nom se double', () => {
    const piege = { ...record, parcelle: 'Le "Clos"' }
    expect(toCsv([piege])).toContain('"Le ""Clos"""')
  })

  test('l’en-tête suit l’ordre des colonnes et se traduit', () => {
    const csv = toCsv([record], { headers: ['Date', 'Heure', 'Parcelle', 'Produit',
      'Température', 'Humidité', 'Vent', 'Rafales', 'Pluie', 'Avis', 'Score'] })
    expect(csv.split('\r\n')[0]).toBe('﻿Date;Heure;Parcelle;Produit;Température;Humidité;Vent;Rafales;Pluie;Avis;Score')
  })

  test('chaque ligne a autant de champs que de colonnes', () => {
    const line = toCsv([record]).split('\r\n')[1]!
    expect(line.split(';')).toHaveLength(REGISTER_COLUMNS.length)
  })

  test('un export vide garde son en-tête', () => {
    const csv = toCsv([])
    expect(csv.split('\r\n').filter(Boolean)).toHaveLength(1)
  })

  test('on peut demander le point et la virgule pour un tableur anglais', () => {
    const line = toCsv([record], { delimiter: ',', decimal: '.' }).split('\r\n')[1]!
    expect(line).toContain('11.2')
    expect(line.split(',')).toHaveLength(REGISTER_COLUMNS.length)
  })
})
