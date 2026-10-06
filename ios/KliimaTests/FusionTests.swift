import XCTest
@testable import Kliima

/// La prévision recoupée : mêmes cas que `rust/klima-core/src/fusion.rs` et
/// `rust/klima-api/src/ensemble.rs`.
final class FusionTests: XCTestCase {

    private let midi = Date(timeIntervalSince1970: 1_778_587_200)

    private func base(_ decalage: Int) -> HourlySample {
        HourlySample(
            time: midi.addingTimeInterval(Double(decalage) * 3600),
            weatherCode: 1, isDay: true, precipitationProbability: 10,
            temperature: 18, apparentTemperature: 17, relativeHumidity: 60, dewPoint: 10,
            precipitation: 0, windSpeed: 10, windGusts: 20, uvIndex: 4
        )
    }

    private func voix(_ decalage: Int, _ temperature: Double, _ pluie: Double, _ code: Int) -> HeureSource {
        HeureSource(
            time: midi.addingTimeInterval(Double(decalage) * 3600),
            temperature: temperature, precipitation: pluie, vent: 12, code: code
        )
    }

    private var courant: CurrentSample {
        CurrentSample(
            time: midi.addingTimeInterval(20 * 60), temperature: 18.4, apparentTemperature: 17.4,
            weatherCode: 1, isDay: true, relativeHumidity: 60, windSpeed: 10, windGusts: 20, pressure: 1015
        )
    }

    func testLaMedianeIgnoreLeModeleQuiSEgare() {
        XCTAssertEqual(Fusion.mediane([19, 19.5, 20, 26]), 19.75)
        XCTAssertEqual(Fusion.mediane([19, 19.5, 20, 26, 12]), 19.5)
        XCTAssertNil(Fusion.mediane([]))
        XCTAssertEqual(Fusion.mediane([.nan, 3]), 3)
    }

    func testLeRisqueMeleLaPartQuiMouilleEtLeRisquePublie() {
        let cumuls = [0.4, 0.2, 0, 1.1, 0, 0.3, 0, 0.5, 0.2]
        XCTAssertEqual(Fusion.risque(cumuls, [40]), 53)
        XCTAssertEqual(Fusion.risque(cumuls, []), 67)
        XCTAssertNil(Fusion.risque([], [40]))
    }

    func testLeTempsRetenuEstCeluiDeLaMajorite() {
        XCTAssertEqual(Fusion.codeMajoritaire([2, 2, 3, 61, 63], [0, 0, 0, 0.4, 1]), 2)
        XCTAssertEqual(Fusion.codeMajoritaire([3, 61, 61, 80], [0, 0.3, 0.2, 0.6]), 61)
        XCTAssertEqual(Fusion.codeMajoritaire([61, 80, 3], [0.3, 0.6, 0]), 80)
        XCTAssertEqual(Fusion.codeMajoritaire([3, 3], [0.4, 0.5]), 3)
        XCTAssertNil(Fusion.codeMajoritaire([], [0.4]))
    }

    func testChaqueHeurePrendLaMedianeDesSources() {
        let series = [
            SerieSource(sourceId: "a", heures: [voix(0, 19, 0, 2), voix(1, 20, 0.6, 61)]),
            SerieSource(sourceId: "b", heures: [voix(0, 20, 0, 3), voix(1, 21, 0.4, 61)]),
            SerieSource(sourceId: "c", heures: [voix(0, 26, 0, 2), voix(1, 20.5, 0, 3)]),
        ]
        let r = Fusion.recouper(heures: [base(0), base(1)], jours: [], courant: courant, series: series, observation: .aucune)

        let h0 = r.heures[0]
        XCTAssertEqual(h0.temperature, 20, "le 26 de c ne tire rien")
        XCTAssertEqual(h0.apparentTemperature, 19, "sans ressenti publié, la base suit l'écart")
        XCTAssertEqual(h0.weatherCode, 2)
        XCTAssertEqual(h0.precipitationProbability, 0)
        XCTAssertEqual(h0.windSpeed, 12)
        XCTAssertEqual(h0.windGusts, 20, "personne n'a donné de rafales : la base")
        XCTAssertEqual(h0.uvIndex, 4, "l'UV reste celui de la base")

        let h1 = r.heures[1]
        XCTAssertEqual(h1.temperature, 20.5)
        XCTAssertEqual(h1.precipitation, 0.4)
        XCTAssertEqual(h1.precipitationProbability, 67, "deux sources sur trois mouillent")
        XCTAssertEqual(h1.weatherCode, 61)
        XCTAssertEqual(r.sources, ["a", "b", "c"])
    }

    func testUneHeureSansVoixGardeLaBase() {
        let r = Fusion.recouper(
            heures: [base(0), base(1)], jours: [], courant: courant,
            series: [SerieSource(sourceId: "a", heures: [voix(0, 19, 0, 2)])], observation: .aucune
        )
        XCTAssertEqual(r.heures[1], base(1))
    }

    func testUneSourceSansTemperatureNeVotePas() {
        let muette = SerieSource(sourceId: "muette", heures: [HeureSource(time: midi)])
        let r = Fusion.recouper(heures: [base(0)], jours: [], courant: courant, series: [muette], observation: .aucune)
        XCTAssertEqual(r.heures[0].temperature, 18)
        XCTAssertTrue(r.sources.isEmpty)
    }

    func testLInstantPresentSuitLHeureRecoupeeEtLaStation() {
        let series = [
            SerieSource(sourceId: "a", heures: [voix(0, 20, 0, 2)]),
            SerieSource(sourceId: "b", heures: [voix(0, 20, 0, 2)]),
        ]
        var r = Fusion.recouper(heures: [base(0)], jours: [], courant: courant, series: series, observation: .aucune)
        XCTAssertEqual(r.courant.temperature, 20.4, accuracy: 1e-9)
        XCTAssertEqual(r.courant.apparentTemperature, 19.4, accuracy: 1e-9)
        XCTAssertEqual(r.courant.weatherCode, 2)

        r = Fusion.recouper(heures: [base(0)], jours: [], courant: courant, series: series, observation: FusionObservation(temperature: 19))
        XCTAssertEqual(r.courant.temperature, 19.7, accuracy: 1e-9)

        r = Fusion.recouper(heures: [base(0)], jours: [], courant: courant, series: [], observation: .aucune)
        XCTAssertEqual(r.courant, courant)
    }

    func testCeQuiTombeNeSeVotePas() {
        XCTAssertEqual(Fusion.codePresent(base: 1, vote: 2, tombe: 80), 80)
        XCTAssertEqual(Fusion.codePresent(base: 61, vote: 2, tombe: nil), 61)
        XCTAssertEqual(Fusion.codePresent(base: 1, vote: 63, tombe: nil), 63)
        XCTAssertEqual(Fusion.codePresent(base: 1, vote: 2, tombe: nil), 2)
        XCTAssertEqual(Fusion.codePresent(base: 1, vote: nil, tombe: nil), 1)
        XCTAssertEqual(Fusion.codePresent(base: 3, vote: 2, tombe: 45), 2, "du brouillard n'est pas ce qui tombe")
    }

    private func avecCode(_ c: CurrentSample, _ code: Int) -> CurrentSample {
        CurrentSample(
            time: c.time, temperature: c.temperature, apparentTemperature: c.apparentTemperature,
            weatherCode: code, isDay: c.isDay, relativeHumidity: c.relativeHumidity,
            windSpeed: c.windSpeed, windGusts: c.windGusts, pressure: c.pressure
        )
    }

    func testLaPluieDeLaBaseNEstPasEffaceeParLaMajorite() {
        let mouille = avecCode(courant, 61)
        let series = [
            SerieSource(sourceId: "a", heures: [voix(0, 18, 0, 2)]),
            SerieSource(sourceId: "b", heures: [voix(0, 18, 0, 3)]),
            SerieSource(sourceId: "c", heures: [voix(0, 18, 0, 2)]),
        ]
        let r = Fusion.recouper(heures: [base(0), base(1)], jours: [], courant: mouille, series: series, observation: .aucune)
        XCTAssertEqual(r.courant.weatherCode, 61)
        let h0 = r.heures[0]
        XCTAssertEqual(h0.weatherCode, 61, "l'heure en cours dit la même chose que l'en-tête")
        XCTAssertEqual(h0.precipitation, 0.1)
        XCTAssertEqual(h0.precipitationProbability, 50)
        XCTAssertEqual(r.heures[1], base(1), "l'heure suivante reste au vote")
    }

    func testUnePluieObserveeSImposeALInstantEtALHeure() {
        let series = [SerieSource(sourceId: "a", heures: [voix(0, 18, 0, 2)])]
        let vue = FusionObservation(temperature: nil, tombe: 80)
        var r = Fusion.recouper(heures: [base(0)], jours: [], courant: courant, series: series, observation: vue)
        XCTAssertEqual(r.courant.weatherCode, 80)
        XCTAssertEqual(r.heures[0].weatherCode, 80)
        XCTAssertEqual(r.heures[0].precipitationProbability, 100, "vue, ce n'est plus un risque")
        r = Fusion.recouper(heures: [base(0)], jours: [], courant: courant, series: [], observation: vue)
        XCTAssertEqual(r.courant.weatherCode, 80)
        XCTAssertEqual(r.courant.temperature, courant.temperature)
        let mouille = avecCode(courant, 61)
        XCTAssertEqual(
            Fusion.recouper(heures: [base(0)], jours: [], courant: mouille, series: [], observation: .aucune).courant.weatherCode,
            61
        )
    }

    func testLesJourneesPrennentAussiLaMediane() {
        let date = midi.addingTimeInterval(-12 * 3600)
        let jour = DailySample(
            date: date, weatherCode: 1, temperatureMin: 10, temperatureMax: 20,
            precipitationSum: 0, precipitationProbabilityMax: 5, windGustsMax: 30, uvIndexMax: 5,
            sunrise: Date(timeIntervalSince1970: 1), sunset: Date(timeIntervalSince1970: 2)
        )
        let j = { (min: Double, max: Double, cumul: Double, code: Int) in
            JourSource(date: date, minimum: min, maximum: max, cumul: cumul, code: code)
        }
        let series = [j(9, 21, 0, 2), j(11, 22, 2, 61), j(10, 23, 1, 61)].enumerated().map {
            SerieSource(sourceId: String($0.offset), heures: [], jours: [$0.element])
        }
        let d = Fusion.recouper(heures: [], jours: [jour], courant: courant, series: series, observation: .aucune).jours[0]
        XCTAssertEqual(d.temperatureMin, 10)
        XCTAssertEqual(d.temperatureMax, 22)
        XCTAssertEqual(d.precipitationSum, 1)
        XCTAssertEqual(d.precipitationProbabilityMax, 67)
        XCTAssertEqual(d.weatherCode, 61)
        XCTAssertEqual(d.uvIndexMax, 5, "UV, lever et coucher : la base")
        XCTAssertEqual(d.sunrise, jour.sunrise)
    }

    // MARK: Les séries des fournisseurs

    func testUneSerieParModeleQuiCouvreLePoint() throws {
        let reponse = Data("""
        {"timezone": "Europe/Paris", "utc_offset_seconds": 7200,
         "hourly": {
           "time": ["2026-05-12T12:00", "2026-05-12T13:00"],
           "temperature_2m_meteofrance_seamless": [19.5, 20.1],
           "precipitation_meteofrance_seamless": [0.0, 0.3],
           "weather_code_meteofrance_seamless": [2, 61],
           "precipitation_probability_meteofrance_seamless": [null, null],
           "temperature_2m_jma_seamless": [15.5, null],
           "wind_gusts_10m_jma_seamless": [null, null],
           "temperature_2m_gem_seamless": [null, null]
         },
         "daily": {
           "time": ["2026-05-12"],
           "temperature_2m_max_meteofrance_seamless": [22.0],
           "temperature_2m_min_meteofrance_seamless": [11.0],
           "precipitation_sum_meteofrance_seamless": [1.2]
         }}
        """.utf8)
        let series = WeatherProviders.decodeEnsemble(reponse)
        XCTAssertEqual(series.map(\.sourceId), ["meteofrance_seamless", "jma_seamless"], "GEM, tout à null, est écarté")

        let mf = series[0]
        var paris = Calendar(identifier: .gregorian)
        paris.timeZone = try XCTUnwrap(TimeZone(identifier: "Europe/Paris"))
        XCTAssertEqual(paris.component(.hour, from: mf.heures[0].time), 12)
        XCTAssertEqual(mf.heures[1].temperature, 20.1)
        XCTAssertEqual(mf.heures[1].code, 61)
        XCTAssertNil(mf.heures[0].probabilite, "un risque nul n'est pas un risque à zéro")
        XCTAssertEqual(mf.jours[0].maximum, 22)
        XCTAssertEqual(mf.jours[0].cumul, 1.2)

        let jma = series[1]
        XCTAssertNil(jma.heures[1].temperature)
        XCTAssertNil(jma.heures[0].rafales)

        XCTAssertTrue(WeatherProviders.decodeEnsemble(Data("pas du json".utf8)).isEmpty)
    }

    func testLesSymbolesDeMetNorwayDeviennentDesCodes() {
        XCTAssertEqual(WeatherProviders.codeDuSymbole("clearsky_day"), 0)
        XCTAssertEqual(WeatherProviders.codeDuSymbole("lightrainshowers_night"), 80)
        XCTAssertEqual(WeatherProviders.codeDuSymbole("heavyrainandthunder"), 95)
        XCTAssertEqual(WeatherProviders.codeDuSymbole("snow"), 73)
        XCTAssertNil(WeatherProviders.codeDuSymbole("inconnu"))
    }

    func testMetNorwayEnKmHEtSeulementLesPasHoraires() throws {
        let corps = Data("""
        {"properties":{"timeseries":[
          {"time":"2026-05-12T10:00:00Z","data":{"instant":{"details":{"air_temperature":18.2,"wind_speed":5.0}},
            "next_1_hours":{"summary":{"symbol_code":"lightrain"},"details":{"precipitation_amount":0.4}}}},
          {"time":"2026-05-15T12:00:00Z","data":{"instant":{"details":{"air_temperature":14.0,"wind_speed":2.0}},
            "next_6_hours":{"summary":{"symbol_code":"cloudy"},"details":{"precipitation_amount":0.0}}}}
        ]}}
        """.utf8)
        let serie = try XCTUnwrap(WeatherProviders.metNorwaySerie(from: try MetPayload.decode(corps)))
        XCTAssertEqual(serie.sourceId, WeatherProviders.metNorwaySource.id)
        XCTAssertEqual(serie.heures.count, 1, "le pas de six heures n'entre pas")
        let h = serie.heures[0]
        XCTAssertEqual(h.time, Date(timeIntervalSince1970: 1_778_580_000), "10 h UTC")
        XCTAssertEqual(h.temperature, 18.2)
        XCTAssertEqual(h.vent, 18)
        XCTAssertEqual(h.code, 61)
        XCTAssertEqual(h.precipitation, 0.4)
    }
}
