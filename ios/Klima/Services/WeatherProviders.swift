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
    static let userAgent = "Klima/1.0 (météo agricole; https://maxlestage.github.io/meteo/)"

    private static let openMeteoAttribution =
        "Open-Meteo — modèles Météo-France, ECMWF, DWD et NOAA"

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

    /// Interroge les trois fournisseurs en parallèle et recoupe leurs réponses.
    static func consensus(for parcelle: Parcelle, session: URLSession = .shared) async -> Consensus? {
        let outcomes = await withTaskGroup(of: Outcome.self) { group -> [Outcome] in
            group.addTask { await run("open-meteo") { try await openMeteo(parcelle, session) } }
            group.addTask { await run("met-norway") { try await metNorway(parcelle, session) } }
            group.addTask { await run("bright-sky") { try await brightSky(parcelle, session) } }

            var collected: [Outcome] = []
            for await outcome in group { collected.append(outcome) }
            return collected
        }

        return ModelConsensus.consensus(
            outcomes.flatMap(\.readings),
            answered: outcomes.filter { !$0.readings.isEmpty }.count,
            queried: outcomes.count
        )
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

    // MARK: MET Norway

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
                // MET Norway donne le vent en m/s ; Klima raisonne en km/h.
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
            struct Details: Decodable { let details: [String: Double]? }
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

/// Réponse de Bright Sky : une observation de station.
struct BrightSkyPayload: Decodable {
    struct Weather: Decodable {
        let temperature: Double?
        let precipitation: Double?
        let wind_speed: Double?
    }
    let weather: Weather?
}
