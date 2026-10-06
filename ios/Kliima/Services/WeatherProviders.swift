import Foundation

/// Fournisseurs de prévision interrogés en parallèle.
///
/// Chaque fournisseur est isolé : une panne, un refus ou une absence de
/// couverture en écarte un seul. Le recoupement porte sur ce qui a répondu.
/// Miroir de `core/src/providers/`, à une différence près : `URLSession` peut
/// fixer l'en-tête `User-Agent`, ce qu'un navigateur interdit. MET Norway,
/// dont les conditions l'exigent, n'est donc appelable que d'ici.
enum WeatherProviders {

    /// Identifie l'application auprès des services qui l'exigent.
    static let userAgent = "Kliima/1.0 (météo de ville; https://maxlestage.github.io/meteo/)"

    private static let openMeteoAttribution =
        "Open-Meteo — modèles Météo-France, ECMWF, DWD, NOAA, Met Office, ECCC et JMA"

    static let openMeteoSources: [WeatherSource] = [
        WeatherSource(id: "meteofrance_seamless", name: "AROME / ARPEGE",
                      institution: "Météo-France", country: "FR",
                      provider: "open-meteo", attribution: openMeteoAttribution),
        WeatherSource(id: "ecmwf_ifs025", name: "IFS", institution: "ECMWF", country: "EU",
                      provider: "open-meteo", attribution: openMeteoAttribution),
        WeatherSource(id: "icon_seamless", name: "ICON",
                      institution: "Deutscher Wetterdienst", country: "DE",
                      provider: "open-meteo", attribution: openMeteoAttribution),
        WeatherSource(id: "gfs_seamless", name: "GFS", institution: "NOAA", country: "US",
                      provider: "open-meteo", attribution: openMeteoAttribution),
        WeatherSource(id: "ukmo_seamless", name: "UM", institution: "Met Office", country: "GB",
                      provider: "open-meteo", attribution: openMeteoAttribution),
        WeatherSource(id: "gem_seamless", name: "GEM", institution: "ECCC", country: "CA",
                      provider: "open-meteo", attribution: openMeteoAttribution),
        WeatherSource(id: "jma_seamless", name: "GSM", institution: "JMA", country: "JP",
                      provider: "open-meteo", attribution: openMeteoAttribution),
    ]

    static let metNorwaySource = WeatherSource(
        id: "met-no-locationforecast",
        name: "Locationforecast",
        institution: "MET Norway",
        country: "NO",
        provider: "met-norway",
        attribution: "Données de MET Norway (Norwegian Meteorological Institute), licence NLOD / CC BY 4.0"
    )

    static let brightSkySource = WeatherSource(
        id: "brightsky-dwd",
        name: "Observation DWD",
        institution: "Deutscher Wetterdienst",
        country: "DE",
        provider: "bright-sky",
        attribution: "Bright Sky — données du Deutscher Wetterdienst, licence CC BY 4.0"
    )

    static var allSources: [WeatherSource] {
        openMeteoSources + [metNorwaySource, brightSkySource]
    }

    static func source(_ id: String) -> WeatherSource? {
        allSources.first { $0.id == id }
    }

    /// Mentions à afficher pour les sources effectivement utilisées.
    static func attributions(for readings: [SourceReading]) -> [String] {
        var seen: [String] = []
        for reading in readings where !seen.contains(reading.source.attribution) {
            seen.append(reading.source.attribution)
        }
        return seen
    }
}

// MARK: - Appels

extension WeatherProviders {

    /// Ce qu'un fournisseur a pu dire, et pourquoi il n'a rien dit.
    struct Outcome {
        let provider: String
        let readings: [SourceReading]
        let failed: Bool
    }

    /// Ce que tous les fournisseurs ont dit : de quoi recouper la prévision,
    /// et de quoi dire leur accord.
    struct Ensemble {
        /// L'accord des sources pour l'heure en cours.
        let consensus: Consensus?
        /// Les séries de chaque source, pour la prévision recoupée.
        let series: [SerieSource]
        /// La température d'une station proche, s'il y en a une.
        let observation: Double?
    }

    /// Ce qu'un appel a rapporté : des relevés pour l'accord, des séries pour
    /// la prévision, une mesure de station.
    private struct Apport {
        let outcome: Outcome
        var series: [SerieSource] = []
        var observation: Double?
    }

    /// Interroge les trois fournisseurs en parallèle.
    ///
    /// Open-Meteo répond pour sept modèles d'un coup, sur toute la période : la
    /// même réponse fait leurs séries et leur relevé de l'heure. MET Norway sert
    /// deux fois aussi. Bright Sky, une station, ne vote que pour l'instant.
    static func ensemble(for parcelle: Parcelle, days: Int, session: URLSession = .shared) async -> Ensemble {
        let apports = await withTaskGroup(of: Apport.self) { group -> [Apport] in
            group.addTask {
                do {
                    let data = try await openMeteoEnsemble(parcelle, days: days, session)
                    let readings = (try? JSONDecoder().decode(ModelPayload.self, from: data))
                        .map(openMeteoReadings(from:)) ?? []
                    return Apport(
                        outcome: Outcome(provider: "open-meteo", readings: readings, failed: false),
                        series: decodeEnsemble(data)
                    )
                } catch {
                    return Apport(outcome: Outcome(provider: "open-meteo", readings: [], failed: true))
                }
            }
            group.addTask {
                do {
                    let payload = try await metNorwayPayload(parcelle, session)
                    return Apport(
                        outcome: Outcome(provider: "met-norway", readings: metNorwayReadings(from: payload), failed: false),
                        series: metNorwaySerie(from: payload).map { [$0] } ?? []
                    )
                } catch {
                    return Apport(outcome: Outcome(provider: "met-norway", readings: [], failed: true))
                }
            }
            group.addTask {
                let outcome = await run("bright-sky") { try await brightSky(parcelle, session) }
                return Apport(outcome: outcome, observation: outcome.readings.first?.temperature)
            }

            var collected: [Apport] = []
            for await apport in group { collected.append(apport) }
            return collected
        }

        // Toujours dans le même ordre, quel que soit celui des réponses.
        let ordre = ["open-meteo", "met-norway", "bright-sky"]
        let tries = apports.sorted {
            (ordre.firstIndex(of: $0.outcome.provider) ?? 9) < (ordre.firstIndex(of: $1.outcome.provider) ?? 9)
        }
        let outcomes = tries.map(\.outcome)
        return Ensemble(
            consensus: ModelConsensus.consensus(
                outcomes.flatMap(\.readings),
                answered: outcomes.filter { !$0.readings.isEmpty }.count,
                queried: outcomes.count
            ),
            series: tries.flatMap(\.series),
            observation: tries.compactMap(\.observation).first
        )
    }

    /// Le seul accord des sources, pour l'heure en cours.
    static func consensus(for parcelle: Parcelle, session: URLSession = .shared) async -> Consensus? {
        await ensemble(for: parcelle, days: 1, session: session).consensus
    }

    private static func run(
        _ provider: String,
        _ work: () async throws -> [SourceReading]
    ) async -> Outcome {
        do {
            return Outcome(provider: provider, readings: try await work(), failed: false)
        } catch {
            return Outcome(provider: provider, readings: [], failed: true)
        }
    }

    private static func json(_ url: URL, _ session: URLSession, userAgent: Bool = false) async throws -> Data {
        var request = URLRequest(url: url)
        if userAgent { request.setValue(WeatherProviders.userAgent, forHTTPHeaderField: "User-Agent") }
        let (data, response) = try await session.data(for: request)
        if let http = response as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
            throw AgroWeatherError.badStatus(http.statusCode)
        }
        return data
    }

    // MARK: Open-Meteo

    static func openMeteo(_ parcelle: Parcelle, _ session: URLSession) async throws -> [SourceReading] {
        var components = URLComponents(string: "https://api.open-meteo.com/v1/forecast")!
        components.queryItems = [
            URLQueryItem(name: "latitude", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "longitude", value: String(format: "%.4f", parcelle.longitude)),
            URLQueryItem(name: "hourly", value: "temperature_2m,precipitation,wind_speed_10m"),
            URLQueryItem(name: "models", value: openMeteoSources.map(\.id).joined(separator: ",")),
            URLQueryItem(name: "wind_speed_unit", value: "kmh"),
            URLQueryItem(name: "timezone", value: "auto"),
            URLQueryItem(name: "forecast_days", value: "1"),
        ]

        let data = try await json(components.url!, session)
        let payload = try JSONDecoder().decode(ModelPayload.self, from: data)
        return openMeteoReadings(from: payload)
    }

    /// Avec plusieurs modèles, Open-Meteo suffixe chaque colonne de
    /// l'identifiant du modèle. Une source sans température ne couvre pas la
    /// parcelle : on l'écarte plutôt que de la compter pour zéro.
    static func openMeteoReadings(from payload: ModelPayload) -> [SourceReading] {
        guard let index = currentHourIndex(payload) else { return [] }

        return openMeteoSources.compactMap { source in
            guard let temperature = payload.value("temperature_2m_\(source.id)", at: index) else {
                return nil
            }
            return SourceReading(
                source: source,
                temperature: temperature,
                precipitation: payload.value("precipitation_\(source.id)", at: index) ?? 0,
                windSpeed: payload.value("wind_speed_10m_\(source.id)", at: index) ?? 0
            )
        }
    }

    private static func currentHourIndex(_ payload: ModelPayload) -> Int? {
        let zone = TimeZone(identifier: payload.timezone ?? "") ?? .current
        let formatter = DateFormatter.openMeteo(format: "yyyy-MM-dd'T'HH:mm", zone: zone)
        let start = (Date().timeIntervalSince1970 / 3600).rounded(.down) * 3600

        for (index, stamp) in payload.hourly.time.enumerated() {
            guard let date = formatter.date(from: stamp) else { continue }
            if date.timeIntervalSince1970 >= start { return index }
        }
        return payload.hourly.time.isEmpty ? nil : payload.hourly.time.count - 1
    }

    /// Les sept modèles, heure par heure et jour par jour, sur `days` jours :
    /// les variables de `klima-api/src/ensemble.rs`.
    static let ensembleHeures = [
        "temperature_2m", "apparent_temperature", "precipitation",
        "precipitation_probability", "weather_code", "wind_speed_10m", "wind_gusts_10m",
    ]
    static let ensembleJours = [
        "temperature_2m_max", "temperature_2m_min", "precipitation_sum",
        "precipitation_probability_max", "wind_gusts_10m_max", "weather_code",
    ]

    static func openMeteoEnsemble(_ parcelle: Parcelle, days: Int, _ session: URLSession) async throws -> Data {
        var components = URLComponents(string: "https://api.open-meteo.com/v1/forecast")!
        components.queryItems = [
            URLQueryItem(name: "latitude", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "longitude", value: String(format: "%.4f", parcelle.longitude)),
            URLQueryItem(name: "hourly", value: ensembleHeures.joined(separator: ",")),
            URLQueryItem(name: "daily", value: ensembleJours.joined(separator: ",")),
            URLQueryItem(name: "models", value: openMeteoSources.map(\.id).joined(separator: ",")),
            URLQueryItem(name: "wind_speed_unit", value: "kmh"),
            URLQueryItem(name: "timezone", value: "auto"),
            URLQueryItem(name: "forecast_days", value: String(days)),
        ]
        return try await json(components.url!, session)
    }

    /// Une série par modèle qui couvre le point : un modèle dont toutes les
    /// températures sont nulles est écarté, pas compté pour zéro.
    static func decodeEnsemble(_ data: Data) -> [SerieSource] {
        guard let payload = try? JSONDecoder().decode(EnsemblePayload.self, from: data) else { return [] }
        let zone = TimeZone(identifier: payload.timezone ?? "") ?? .current
        let heureFormat = DateFormatter.openMeteo(format: "yyyy-MM-dd'T'HH:mm", zone: zone)
        let jourFormat = DateFormatter.openMeteo(format: "yyyy-MM-dd", zone: zone)
        let heures = payload.hourly
        let jours = payload.daily

        return openMeteoSources.compactMap { source in
            let m = source.id
            func h(_ variable: String, _ i: Int) -> Double? { heures?.columns["\(variable)_\(m)"].flatMap { i < $0.count ? $0[i] : nil } }
            func j(_ variable: String, _ i: Int) -> Double? { jours?.columns["\(variable)_\(m)"].flatMap { i < $0.count ? $0[i] : nil } }

            let serieHeures: [HeureSource] = (heures?.time ?? []).enumerated().compactMap { i, stamp in
                guard let time = heureFormat.date(from: stamp) else { return nil }
                return HeureSource(
                    time: time,
                    temperature: h("temperature_2m", i),
                    ressenti: h("apparent_temperature", i),
                    precipitation: h("precipitation", i),
                    probabilite: h("precipitation_probability", i),
                    vent: h("wind_speed_10m", i),
                    rafales: h("wind_gusts_10m", i),
                    code: h("weather_code", i).map { Int($0) }
                )
            }
            guard serieHeures.contains(where: { $0.temperature != nil }) else { return nil }

            let serieJours: [JourSource] = (jours?.time ?? []).enumerated().compactMap { i, stamp in
                guard let date = jourFormat.date(from: stamp) else { return nil }
                return JourSource(
                    date: date,
                    minimum: j("temperature_2m_min", i),
                    maximum: j("temperature_2m_max", i),
                    cumul: j("precipitation_sum", i),
                    probabilite: j("precipitation_probability_max", i),
                    rafales: j("wind_gusts_10m_max", i),
                    code: j("weather_code", i).map { Int($0) }
                )
            }
            return SerieSource(sourceId: m, heures: serieHeures, jours: serieJours)
        }
    }

    // MARK: MET Norway

    static func metNorwayPayload(_ parcelle: Parcelle, _ session: URLSession) async throws -> MetPayload {
        var components = URLComponents(string: "https://api.met.no/weatherapi/locationforecast/2.0/compact")!
        components.queryItems = [
            URLQueryItem(name: "lat", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "lon", value: String(format: "%.4f", parcelle.longitude)),
        ]
        // Leurs conditions imposent de s'identifier ; `URLSession` le permet.
        return try MetPayload.decode(try await json(components.url!, session, userAgent: true))
    }

    /// La série de MET Norway : seuls les pas qui portent une pluie sur l'heure
    /// entrent — plus loin, la série passe à six heures. Le vent passe en km/h.
    static func metNorwaySerie(from payload: MetPayload) -> SerieSource? {
        let heures: [HeureSource] = (payload.properties?.timeseries ?? []).compactMap { entree in
            guard
                let time = entree.time,
                let pluie = entree.data?.next_1_hours?.details?["precipitation_amount"]
            else { return nil }
            let instant = entree.data?.instant?.details
            return HeureSource(
                time: time,
                temperature: instant?["air_temperature"].flatMap { $0.isFinite ? $0 : nil },
                precipitation: pluie,
                vent: instant?["wind_speed"].map { $0 * 3.6 },
                code: entree.data?.next_1_hours?.summary?.symbol_code.flatMap(codeDuSymbole)
            )
        }
        guard heures.contains(where: { $0.temperature != nil }) else { return nil }
        return SerieSource(sourceId: metNorwaySource.id, heures: heures)
    }

    /// Le code météo d'un symbole de MET Norway — la table de `klima-api`.
    static func codeDuSymbole(_ symbole: String) -> Int? {
        guard let nom = symbole.split(separator: "_").first.map(String.init) else { return nil }
        if nom.contains("thunder") { return 95 }
        switch nom {
        case "clearsky": return 0
        case "fair": return 1
        case "partlycloudy": return 2
        case "cloudy": return 3
        case "fog": return 45
        case "lightrain": return 61
        case "rain": return 63
        case "heavyrain": return 65
        case "lightrainshowers": return 80
        case "rainshowers": return 81
        case "heavyrainshowers": return 82
        case "lightsleet", "sleet", "heavysleet", "lightsleetshowers", "sleetshowers", "heavysleetshowers": return 66
        case "lightsnow": return 71
        case "snow": return 73
        case "heavysnow": return 75
        case "lightsnowshowers", "snowshowers": return 85
        case "heavysnowshowers": return 86
        default: return nil
        }
    }

    static func metNorway(_ parcelle: Parcelle, _ session: URLSession) async throws -> [SourceReading] {
        var components = URLComponents(string: "https://api.met.no/weatherapi/locationforecast/2.0/compact")!
        components.queryItems = [
            URLQueryItem(name: "lat", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "lon", value: String(format: "%.4f", parcelle.longitude)),
        ]

        // Leurs conditions imposent de s'identifier ; `URLSession` le permet.
        let data = try await json(components.url!, session, userAgent: true)
        return metNorwayReadings(from: try MetPayload.decode(data))
    }

    static func metNorwayReadings(from payload: MetPayload) -> [SourceReading] {
        guard let entry = payload.nearest(to: Date()),
              let temperature = entry.data?.instant?.details?["air_temperature"],
              temperature.isFinite
        else { return [] }

        let windMetresPerSecond = entry.data?.instant?.details?["wind_speed"] ?? 0
        return [
            SourceReading(
                source: metNorwaySource,
                temperature: temperature,
                precipitation: entry.data?.next_1_hours?.details?["precipitation_amount"] ?? 0,
                // MET Norway donne le vent en m/s ; Kliima raisonne en km/h.
                windSpeed: windMetresPerSecond * 3.6
            ),
        ]
    }

    // MARK: Bright Sky

    static func brightSky(_ parcelle: Parcelle, _ session: URLSession) async throws -> [SourceReading] {
        var components = URLComponents(string: "https://api.brightsky.dev/current_weather")!
        components.queryItems = [
            URLQueryItem(name: "lat", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "lon", value: String(format: "%.4f", parcelle.longitude)),
        ]

        var request = URLRequest(url: components.url!)
        request.setValue(userAgent, forHTTPHeaderField: "User-Agent")
        let (data, response) = try await session.data(for: request)

        // 404 : aucune station à portée. Ce n'est pas une panne, c'est une absence.
        if let http = response as? HTTPURLResponse {
            if http.statusCode == 404 { return [] }
            guard (200..<300).contains(http.statusCode) else {
                throw AgroWeatherError.badStatus(http.statusCode)
            }
        }
        return brightSkyReadings(from: try JSONDecoder().decode(BrightSkyPayload.self, from: data))
    }

    static func brightSkyReadings(from payload: BrightSkyPayload) -> [SourceReading] {
        guard let temperature = payload.weather?.temperature, temperature.isFinite else { return [] }
        return [
            SourceReading(
                source: brightSkySource,
                temperature: temperature,
                precipitation: payload.weather?.precipitation ?? 0,
                windSpeed: payload.weather?.wind_speed ?? 0
            ),
        ]
    }
}

// MARK: - Réponses

/// Réponse de MET Norway : une série d'échéances, dont on prend la plus proche.
struct MetPayload: Decodable {

    /// Les horodatages sont en ISO 8601 : sans cette stratégie, le décodage de
    /// `Date` échoue et emporte toute la réponse.
    static func decode(_ data: Data) throws -> MetPayload {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return try decoder.decode(MetPayload.self, from: data)
    }

    struct Entry: Decodable {
        struct Data: Decodable {
            struct Summary: Decodable { let symbol_code: String? }
            struct Details: Decodable {
                let details: [String: Double]?
                let summary: Summary?
            }
            let instant: Details?
            let next_1_hours: Details?
        }
        let time: Date?
        let data: Data?
    }

    struct Properties: Decodable { let timeseries: [Entry]? }
    let properties: Properties?

    /// Échéance la plus proche : la série est horaire puis trihoraire.
    func nearest(to date: Date) -> Entry? {
        properties?.timeseries?
            .filter { $0.time != nil }
            .min { abs($0.time!.timeIntervalSince(date)) < abs($1.time!.timeIntervalSince(date)) }
    }
}

/// Réponse d'Open-Meteo pour plusieurs modèles : des colonnes suffixées, heure
/// par heure et jour par jour.
struct EnsemblePayload: Decodable {
    let timezone: String?
    let hourly: ModelPayload.Block?
    let daily: ModelPayload.Block?
}

/// Réponse de Bright Sky : une observation de station.
struct BrightSkyPayload: Decodable {
    struct Weather: Decodable {
        let temperature: Double?
        let precipitation: Double?
        let wind_speed: Double?
    }
    let weather: Weather?
}
