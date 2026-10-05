import Foundation

/// Erreurs remontées à l'utilisateur, dans la langue de l'appareil.
enum AgroWeatherError: LocalizedError, Equatable {
    case unreachable
    case badStatus(Int)
    case malformedResponse

    var errorDescription: String? {
        switch self {
        case .unreachable:
            return Localized.text("api.unreachable")
        case let .badStatus(code):
            return Localized.text("api.status", String(code))
        case .malformedResponse:
            return Localized.text("api.malformed")
        }
    }
}

/// Contrat du service, pour pouvoir injecter un double en test et en aperçu.
protocol AgroWeatherProviding {
    func forecast(for parcelle: Parcelle, days: Int) async throws -> AgroForecast
    func search(commune query: String) async throws -> [Parcelle]
    func modelConsensus(for parcelle: Parcelle) async throws -> Consensus?
    func air(for parcelle: Parcelle) async throws -> AirSample?
}

/// Client de l'API de prévision d'Open-Meteo, et de son service de qualité de
/// l'air.
///
/// On n'interroge que ce qu'une ville regarde avant de sortir : température
/// et ressenti, pluie et son risque, vent et rafales, indice UV, pression —
/// et, à part, l'air et les pollens. Les variables sont celles de
/// `klima-api/src/open_meteo.rs` et `air.rs`.
struct AgroWeatherService: AgroWeatherProviding {

    private static let forecastURL = URL(string: "https://api.open-meteo.com/v1/forecast")!
    private static let geocodingURL = URL(string: "https://geocoding-api.open-meteo.com/v1/search")!
    private static let airURL = URL(string: "https://air-quality-api.open-meteo.com/v1/air-quality")!

    /// L'indice européen, les polluants qu'on affiche, puis les six pollens —
    /// dans l'ordre de `klima-api/src/air.rs`.
    static let airVariables =
        ["european_aqi", "pm2_5", "pm10", "nitrogen_dioxide", "ozone"] + Pollen.allCases.map(\.variable)

    private static let currentVariables = [
        "temperature_2m",
        "apparent_temperature",
        "relative_humidity_2m",
        "weather_code",
        "is_day",
        "wind_speed_10m",
        "wind_gusts_10m",
        "pressure_msl",
    ]

    private static let hourlyVariables = [
        "temperature_2m",
        "apparent_temperature",
        "weather_code",
        "is_day",
        "precipitation_probability",
        "relative_humidity_2m",
        "dew_point_2m",
        "precipitation",
        "wind_speed_10m",
        "wind_gusts_10m",
        "uv_index",
    ]

    private static let dailyVariables = [
        "weather_code",
        "sunrise",
        "sunset",
        "temperature_2m_min",
        "temperature_2m_max",
        "precipitation_sum",
        "precipitation_probability_max",
        "wind_gusts_10m_max",
        "uv_index_max",
    ]

    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    /// Récupère la prévision d'une ville sur `days` jours.
    func forecast(for parcelle: Parcelle, days: Int = 7) async throws -> AgroForecast {
        var components = URLComponents(url: Self.forecastURL, resolvingAgainstBaseURL: false)!
        components.queryItems = [
            URLQueryItem(name: "latitude", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "longitude", value: String(format: "%.4f", parcelle.longitude)),
            URLQueryItem(name: "current", value: Self.currentVariables.joined(separator: ",")),
            URLQueryItem(name: "hourly", value: Self.hourlyVariables.joined(separator: ",")),
            URLQueryItem(name: "daily", value: Self.dailyVariables.joined(separator: ",")),
            URLQueryItem(name: "wind_speed_unit", value: "kmh"),
            URLQueryItem(name: "timezone", value: "auto"),
            URLQueryItem(name: "forecast_days", value: String(days)),
        ]

        let payload: ForecastPayload = try await get(components.url!)
        let zone = TimeZone(identifier: payload.timezone) ?? .current
        let current = payload.current.decode(in: zone)

        return AgroForecast(
            parcelle: parcelle,
            timezone: payload.timezone,
            elevation: payload.elevation,
            current: current,
            // L'API renvoie la journée entière depuis minuit : on repart de
            // l'heure en cours, pour que « maintenant » soit bien le premier
            // élément des séries.
            hourly: Self.fromCurrentHour(payload.hourly.decode(in: zone), now: current.time),
            daily: payload.daily.decode(in: zone),
            fetchedAt: Date()
        )
    }

    /// Coupe la série horaire au début de l'heure en cours.
    static func fromCurrentHour(_ hours: [HourlySample], now: Date) -> [HourlySample] {
        let start = (now.timeIntervalSince1970 / 3600).rounded(.down) * 3600
        let trimmed = hours.filter { $0.time.timeIntervalSince1970 >= start }
        // Si l'heure courante sort de la série, on garde la série telle quelle.
        return trimmed.isEmpty ? hours : trimmed
    }

    /// Interroge tous les fournisseurs et recoupe leurs réponses.
    ///
    /// Séparé de la prévision principale, à dessein : si la comparaison échoue,
    /// l'application continue avec sa source habituelle.
    func modelConsensus(for parcelle: Parcelle) async throws -> Consensus? {
        await WeatherProviders.consensus(for: parcelle, session: session)
    }

    /// L'air de la ville : qualité et pollens. `nil` si la réponse n'a pas de
    /// mesure — une réponse vide n'est pas un air pur.
    func air(for parcelle: Parcelle) async throws -> AirSample? {
        var components = URLComponents(url: Self.airURL, resolvingAgainstBaseURL: false)!
        components.queryItems = [
            URLQueryItem(name: "latitude", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "longitude", value: String(format: "%.4f", parcelle.longitude)),
            URLQueryItem(name: "current", value: Self.airVariables.joined(separator: ",")),
            URLQueryItem(name: "timezone", value: "auto"),
        ]
        let (data, response) = try await session.data(from: components.url!)
        if let http = response as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
            throw AgroWeatherError.badStatus(http.statusCode)
        }
        return Self.decodeAir(data)
    }

    /// Lit la réponse du service de l'air. Séparé de l'appel pour être testé.
    static func decodeAir(_ data: Data) -> AirSample? {
        guard
            let objet = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
            let current = objet["current"] as? [String: Any]
        else { return nil }
        let nombre = { (nom: String) -> Double? in
            guard let valeur = (current[nom] as? NSNumber)?.doubleValue, valeur.isFinite else { return nil }
            return valeur
        }
        return AirSample(
            europeanAqi: nombre("european_aqi"),
            pm25: nombre("pm2_5"),
            pm10: nombre("pm10"),
            nitrogenDioxide: nombre("nitrogen_dioxide"),
            ozone: nombre("ozone"),
            // Un pollen absent n'est pas un pollen à zéro : on l'écarte.
            pollens: Pollen.allCases.compactMap { pollen in nombre(pollen.variable).map { (pollen, $0) } }
        )
    }

    /// Recherche une commune par son nom (géocodage Open-Meteo).
    func search(commune query: String) async throws -> [Parcelle] {
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.count >= 2 else { return [] }

        var components = URLComponents(url: Self.geocodingURL, resolvingAgainstBaseURL: false)!
        components.queryItems = [
            URLQueryItem(name: "name", value: trimmed),
            URLQueryItem(name: "count", value: "8"),
            URLQueryItem(name: "language", value: "fr"),
            URLQueryItem(name: "format", value: "json"),
        ]

        let payload: GeocodingPayload = try await get(components.url!)
        return (payload.results ?? []).map {
            Parcelle(
                name: $0.name,
                latitude: $0.latitude,
                longitude: $0.longitude,
                admin: $0.admin1,
                country: $0.country
            )
        }
    }

    private func get<T: Decodable>(_ url: URL) async throws -> T {
        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(from: url)
        } catch let error as URLError where error.code == .cancelled {
            throw error
        } catch {
            throw AgroWeatherError.unreachable
        }

        if let http = response as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
            throw AgroWeatherError.badStatus(http.statusCode)
        }

        do {
            return try JSONDecoder().decode(T.self, from: data)
        } catch {
            throw AgroWeatherError.malformedResponse
        }
    }
}

// MARK: - Décodage des réponses
//
// Ces types restent internes (et non privés) pour que la suite de tests
// vérifie directement le contrat de champs avec Open-Meteo.

/// Open-Meteo renvoie des tableaux parallèles indexés par `time`, avec des
/// `null` quand une variable manque sur le point de grille : on les ramène à 0
/// pour garder des séries de longueur homogène.
struct ForecastPayload: Decodable {
    let timezone: String
    let elevation: Double
    let current: CurrentBlock
    let hourly: HourlyBlock
    let daily: DailyBlock

    struct CurrentBlock: Decodable {
        let time: String
        let temperature2m: Double?
        let apparentTemperature: Double?
        let relativeHumidity2m: Double?
        let weatherCode: Int?
        let isDay: Int?
        let windSpeed10m: Double?
        let windGusts10m: Double?
        let pressureMsl: Double?

        enum CodingKeys: String, CodingKey {
            case time
            case temperature2m = "temperature_2m"
            case apparentTemperature = "apparent_temperature"
            case relativeHumidity2m = "relative_humidity_2m"
            case weatherCode = "weather_code"
            case isDay = "is_day"
            case windSpeed10m = "wind_speed_10m"
            case windGusts10m = "wind_gusts_10m"
            case pressureMsl = "pressure_msl"
        }

        func decode(in zone: TimeZone) -> CurrentSample {
            let formatter = DateFormatter.openMeteo(format: "yyyy-MM-dd'T'HH:mm", zone: zone)
            return CurrentSample(
                time: formatter.date(from: time) ?? Date(),
                temperature: temperature2m ?? 0,
                apparentTemperature: apparentTemperature ?? temperature2m ?? 0,
                weatherCode: weatherCode ?? 3,
                isDay: (isDay ?? 1) == 1,
                relativeHumidity: relativeHumidity2m ?? 0,
                windSpeed: windSpeed10m ?? 0,
                windGusts: windGusts10m ?? 0,
                pressure: pressureMsl ?? 0
            )
        }
    }

    struct HourlyBlock: Decodable {
        let time: [String]
        let weatherCode: [Int?]?
        let isDay: [Int?]?
        let precipitationProbability: [Double?]?
        let temperature2m: [Double?]?
        let apparentTemperature: [Double?]?
        let relativeHumidity2m: [Double?]?
        let dewPoint2m: [Double?]?
        let precipitation: [Double?]?
        let windSpeed10m: [Double?]?
        let windGusts10m: [Double?]?
        let uvIndex: [Double?]?

        enum CodingKeys: String, CodingKey {
            case time
            case weatherCode = "weather_code"
            case isDay = "is_day"
            case precipitationProbability = "precipitation_probability"
            case temperature2m = "temperature_2m"
            case apparentTemperature = "apparent_temperature"
            case relativeHumidity2m = "relative_humidity_2m"
            case dewPoint2m = "dew_point_2m"
            case precipitation
            case windSpeed10m = "wind_speed_10m"
            case windGusts10m = "wind_gusts_10m"
            case uvIndex = "uv_index"
        }

        func decode(in zone: TimeZone) -> [HourlySample] {
            let formatter = DateFormatter.openMeteo(format: "yyyy-MM-dd'T'HH:mm", zone: zone)
            let codes = intColumn(weatherCode)
            let day = intColumn(isDay)
            let rainProbability = column(precipitationProbability)
            let temperature = column(temperature2m)
            let apparent = column(apparentTemperature)
            let humidity = column(relativeHumidity2m)
            let dewPoint = column(dewPoint2m)
            let rain = column(precipitation)
            let wind = column(windSpeed10m)
            let gusts = column(windGusts10m)
            let uv = column(uvIndex)

            return time.enumerated().compactMap { index, stamp in
                guard let date = formatter.date(from: stamp) else { return nil }
                return HourlySample(
                    time: date,
                    weatherCode: codes(index) ?? 3,
                    isDay: (day(index) ?? 1) == 1,
                    precipitationProbability: rainProbability(index),
                    temperature: temperature(index),
                    apparentTemperature: apparent(index),
                    relativeHumidity: humidity(index),
                    dewPoint: dewPoint(index),
                    precipitation: rain(index),
                    windSpeed: wind(index),
                    windGusts: gusts(index),
                    uvIndex: uv(index)
                )
            }
        }
    }

    struct DailyBlock: Decodable {
        let time: [String]
        let weatherCode: [Int?]?
        let sunrise: [String?]?
        let sunset: [String?]?
        let temperature2mMin: [Double?]?
        let temperature2mMax: [Double?]?
        let precipitationSum: [Double?]?
        let precipitationProbabilityMax: [Double?]?
        let windGusts10mMax: [Double?]?
        let uvIndexMax: [Double?]?

        enum CodingKeys: String, CodingKey {
            case time
            case weatherCode = "weather_code"
            case sunrise
            case sunset
            case temperature2mMin = "temperature_2m_min"
            case temperature2mMax = "temperature_2m_max"
            case precipitationSum = "precipitation_sum"
            case precipitationProbabilityMax = "precipitation_probability_max"
            case windGusts10mMax = "wind_gusts_10m_max"
            case uvIndexMax = "uv_index_max"
        }

        func decode(in zone: TimeZone) -> [DailySample] {
            let formatter = DateFormatter.openMeteo(format: "yyyy-MM-dd", zone: zone)
            let stampFormatter = DateFormatter.openMeteo(format: "yyyy-MM-dd'T'HH:mm", zone: zone)
            let codes = intColumn(weatherCode)
            let tMin = column(temperature2mMin)
            let tMax = column(temperature2mMax)
            let rain = column(precipitationSum)
            let probability = column(precipitationProbabilityMax)
            let gusts = column(windGusts10mMax)
            let uv = column(uvIndexMax)

            return time.enumerated().compactMap { index, stamp in
                guard let date = formatter.date(from: stamp) else { return nil }
                return DailySample(
                    date: date,
                    weatherCode: codes(index) ?? 3,
                    temperatureMin: tMin(index),
                    temperatureMax: tMax(index),
                    precipitationSum: rain(index),
                    precipitationProbabilityMax: probability(index),
                    windGustsMax: gusts(index),
                    uvIndexMax: uv(index),
                    sunrise: parseStamp(sunrise, index, stampFormatter),
                    sunset: parseStamp(sunset, index, stampFormatter)
                )
            }
        }
    }
}

/// Réponse d'une requête multi-modèles : des colonnes suffixées, lues à la
/// demande plutôt que déclarées une par une.
struct ModelPayload: Decodable {
    let timezone: String?
    let hourly: Block

    struct Block: Decodable {
        let time: [String]
        /// Colonnes restantes, indexées par nom.
        let columns: [String: [Double?]]

        private struct Key: CodingKey {
            let stringValue: String
            init?(stringValue: String) { self.stringValue = stringValue }
            var intValue: Int? { nil }
            init?(intValue: Int) { nil }
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: Key.self)
            time = try container.decode([String].self, forKey: Key(stringValue: "time")!)

            var columns: [String: [Double?]] = [:]
            for key in container.allKeys where key.stringValue != "time" {
                if let values = try? container.decode([Double?].self, forKey: key) {
                    columns[key.stringValue] = values
                }
            }
            self.columns = columns
        }
    }

    func value(_ column: String, at index: Int) -> Double? {
        guard let values = hourly.columns[column], values.indices.contains(index) else { return nil }
        guard let value = values[index], value.isFinite else { return nil }
        return value
    }
}

struct GeocodingPayload: Decodable {
    let results: [Result]?

    struct Result: Decodable {
        let name: String
        let latitude: Double
        let longitude: Double
        let admin1: String?
        let country: String?
    }
}

/// Accès sûr à une colonne : hors borne ou `null` renvoient 0.
func column(_ values: [Double?]?) -> (Int) -> Double {
    { index in
        guard let values, values.indices.contains(index), let value = values[index], value.isFinite else {
            return 0
        }
        return value
    }
}

/// Même chose pour les colonnes entières, mais l'absence reste `nil` : les
/// codes météo et l'indicateur jour/nuit ont leur propre valeur de repli.
func intColumn(_ values: [Int?]?) -> (Int) -> Int? {
    { index in
        guard let values, values.indices.contains(index) else { return nil }
        return values[index]
    }
}

/// Horodatage optionnel d'une colonne de dates (lever et coucher du soleil).
func parseStamp(_ values: [String?]?, _ index: Int, _ formatter: DateFormatter) -> Date? {
    guard let values, values.indices.contains(index), let value = values[index] else { return nil }
    return formatter.date(from: value)
}

extension DateFormatter {
    static func openMeteo(format: String, zone: TimeZone) -> DateFormatter {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.timeZone = zone
        formatter.dateFormat = format
        return formatter
    }
}
