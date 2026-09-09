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
}

/// Client de l'API agricole Open-Meteo.
///
/// On n'interroge que les variables agronomiques : température et humidité du
/// sol, évapotranspiration de référence FAO-56, déficit de pression de vapeur,
/// en plus des paramètres nécessaires au calcul des fenêtres de traitement.
/// L'API est libre d'accès et ne demande aucune clé.
struct AgroWeatherService: AgroWeatherProviding {

    private static let forecastURL = URL(string: "https://api.open-meteo.com/v1/forecast")!
    private static let geocodingURL = URL(string: "https://geocoding-api.open-meteo.com/v1/search")!

    private static let currentVariables = [
        "temperature_2m",
        "apparent_temperature",
        "relative_humidity_2m",
        "weather_code",
        "is_day",
        "wind_speed_10m",
        "wind_gusts_10m",
    ]

    private static let hourlyVariables = [
        "temperature_2m",
        "weather_code",
        "is_day",
        "precipitation_probability",
        "relative_humidity_2m",
        "dew_point_2m",
        "precipitation",
        "wind_speed_10m",
        "wind_gusts_10m",
        "soil_temperature_6cm",
        "soil_moisture_3_to_9cm",
        "et0_fao_evapotranspiration",
        "vapour_pressure_deficit",
    ]

    private static let dailyVariables = [
        "weather_code",
        "sunrise",
        "sunset",
        "temperature_2m_min",
        "temperature_2m_max",
        "precipitation_sum",
        "precipitation_probability_max",
        "et0_fao_evapotranspiration",
        "wind_gusts_10m_max",
    ]

    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    /// Récupère la prévision agricole d'une parcelle sur `days` jours.
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

        enum CodingKeys: String, CodingKey {
            case time
            case temperature2m = "temperature_2m"
            case apparentTemperature = "apparent_temperature"
            case relativeHumidity2m = "relative_humidity_2m"
            case weatherCode = "weather_code"
            case isDay = "is_day"
            case windSpeed10m = "wind_speed_10m"
            case windGusts10m = "wind_gusts_10m"
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
                windGusts: windGusts10m ?? 0
            )
        }
    }

    struct HourlyBlock: Decodable {
        let time: [String]
        let weatherCode: [Int?]?
        let isDay: [Int?]?
        let precipitationProbability: [Double?]?
        let temperature2m: [Double?]?
        let relativeHumidity2m: [Double?]?
        let dewPoint2m: [Double?]?
        let precipitation: [Double?]?
        let windSpeed10m: [Double?]?
        let windGusts10m: [Double?]?
        let soilTemperature6cm: [Double?]?
        let soilMoisture3to9cm: [Double?]?
        let et0FaoEvapotranspiration: [Double?]?
        let vapourPressureDeficit: [Double?]?

        enum CodingKeys: String, CodingKey {
            case time
            case weatherCode = "weather_code"
            case isDay = "is_day"
            case precipitationProbability = "precipitation_probability"
            case temperature2m = "temperature_2m"
            case relativeHumidity2m = "relative_humidity_2m"
            case dewPoint2m = "dew_point_2m"
            case precipitation
            case windSpeed10m = "wind_speed_10m"
            case windGusts10m = "wind_gusts_10m"
            case soilTemperature6cm = "soil_temperature_6cm"
            case soilMoisture3to9cm = "soil_moisture_3_to_9cm"
            case et0FaoEvapotranspiration = "et0_fao_evapotranspiration"
            case vapourPressureDeficit = "vapour_pressure_deficit"
        }

        func decode(in zone: TimeZone) -> [HourlySample] {
            let formatter = DateFormatter.openMeteo(format: "yyyy-MM-dd'T'HH:mm", zone: zone)
            let codes = intColumn(weatherCode)
            let day = intColumn(isDay)
            let rainProbability = column(precipitationProbability)
            let temperature = column(temperature2m)
            let humidity = column(relativeHumidity2m)
            let dewPoint = column(dewPoint2m)
            let rain = column(precipitation)
            let wind = column(windSpeed10m)
            let gusts = column(windGusts10m)
            let soilTemperature = column(soilTemperature6cm)
            let soilMoisture = column(soilMoisture3to9cm)
            let et0 = column(et0FaoEvapotranspiration)
            let vpd = column(vapourPressureDeficit)

            return time.enumerated().compactMap { index, stamp in
                guard let date = formatter.date(from: stamp) else { return nil }
                return HourlySample(
                    time: date,
                    weatherCode: codes(index) ?? 3,
                    isDay: (day(index) ?? 1) == 1,
                    precipitationProbability: rainProbability(index),
                    temperature: temperature(index),
                    relativeHumidity: humidity(index),
                    dewPoint: dewPoint(index),
                    precipitation: rain(index),
                    windSpeed: wind(index),
                    windGusts: gusts(index),
                    soilTemperature6cm: soilTemperature(index),
                    soilMoisture3to9cm: soilMoisture(index),
                    et0: et0(index),
                    vapourPressureDeficit: vpd(index)
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
        let et0FaoEvapotranspiration: [Double?]?
        let windGusts10mMax: [Double?]?

        enum CodingKeys: String, CodingKey {
            case time
            case weatherCode = "weather_code"
            case sunrise
            case sunset
            case temperature2mMin = "temperature_2m_min"
            case temperature2mMax = "temperature_2m_max"
            case precipitationSum = "precipitation_sum"
            case precipitationProbabilityMax = "precipitation_probability_max"
            case et0FaoEvapotranspiration = "et0_fao_evapotranspiration"
            case windGusts10mMax = "wind_gusts_10m_max"
        }

        func decode(in zone: TimeZone) -> [DailySample] {
            let formatter = DateFormatter.openMeteo(format: "yyyy-MM-dd", zone: zone)
            let stampFormatter = DateFormatter.openMeteo(format: "yyyy-MM-dd'T'HH:mm", zone: zone)
            let codes = intColumn(weatherCode)
            let tMin = column(temperature2mMin)
            let tMax = column(temperature2mMax)
            let rain = column(precipitationSum)
            let probability = column(precipitationProbabilityMax)
            let et0 = column(et0FaoEvapotranspiration)
            let gusts = column(windGusts10mMax)

            return time.enumerated().compactMap { index, stamp in
                guard let date = formatter.date(from: stamp) else { return nil }
                return DailySample(
                    date: date,
                    weatherCode: codes(index) ?? 3,
                    temperatureMin: tMin(index),
                    temperatureMax: tMax(index),
                    precipitationSum: rain(index),
                    precipitationProbabilityMax: probability(index),
                    et0Sum: et0(index),
                    windGustsMax: gusts(index),
                    sunrise: parseStamp(sunrise, index, stampFormatter),
                    sunset: parseStamp(sunset, index, stampFormatter)
                )
            }
        }
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
