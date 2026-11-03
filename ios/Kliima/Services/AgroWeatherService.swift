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
        case let .badStatus(code) where code == 429 || code >= 500:
            // Surcharge ou panne passagère du fournisseur : ce n'est pas un
            // code qu'on demande à quelqu'un de comprendre.
            return Localized.text("api.busy")
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
    func quarts(for parcelle: Parcelle) async throws -> [QuartSample]?
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

    /// Les variables lues au quart d'heure, pour le guetteur — celles de
    /// `klima-api/src/veille.rs`.
    static let quartVariables = [
        "precipitation", "weather_code", "temperature_2m",
        "apparent_temperature", "wind_gusts_10m", "is_day",
    ]

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
    /// Ce que l'apprenti sait des modèles autour d'une ville. L'application le
    /// demande au relais (`ApprentiRelais`) ; les extensions, qui n'ont pas de
    /// relais, relisent ce que l'application a appris en dernier.
    private let apprenti: (Parcelle) async -> Apprentissage?

    init(
        session: URLSession = .shared,
        apprenti: @escaping (Parcelle) async -> Apprentissage? = { SharedStore.loadApprentissage(pour: $0) }
    ) {
        self.session = session
        self.apprenti = apprenti
    }

    /// La prévision d'une ville sur `days` jours, refaite avec toutes les
    /// sources (`Fusion`).
    ///
    /// La base et les sources partent ensemble. Si les sources se taisent, la
    /// base reste : leur silence ne prive de rien. Le widget, la montre et les
    /// alertes passent par ici, et ont donc la même prévision que l'écran.
    func forecast(for parcelle: Parcelle, days: Int = 7) async throws -> AgroForecast {
        async let ensemble = WeatherProviders.ensemble(for: parcelle, days: days, session: session)
        async let appris = apprenti(parcelle)
        let base = try await previsionDeBase(for: parcelle, days: days)
        let sources = await ensemble

        return Self.recouper(base, avec: sources, apprentissage: await appris)
    }

    /// La prévision de base refaite avec ce que les sources ont dit — par
    /// requête ou par le direct (`DirectRelais`).
    ///
    /// `radar` : ce que voit le radar. Quand il voit la ville, c'est lui qui
    /// dit ce qui tombe — il regarde la ville même, l'aéroport à quelques
    /// kilomètres ; sinon, l'aéroport. Comme `Forecast::recoupee` côté Rust.
    ///
    /// `apprentissage` : ce que l'apprenti sait des modèles. Chacun est
    /// redressé de son biais appris avant de voter (`Apprenti.corriger`).
    static func recouper(
        _ base: AgroForecast, avec sources: WeatherProviders.Ensemble, radar: RadarPrevision? = nil,
        apprentissage: Apprentissage? = nil
    ) -> AgroForecast {
        let tombe: Int?
        if let radar, radar.voitLaVille {
            tombe = Radar.tombe(radar.maintenant)
        } else {
            tombe = sources.ciel?.tombe?.code
        }
        let recoupement = Fusion.recouper(
            heures: base.hourly,
            jours: base.daily,
            courant: base.current,
            series: apprentissage.map { Apprenti.corriger(sources.series, $0.notes) } ?? sources.series,
            observation: FusionObservation(temperature: sources.observation, tombe: tombe)
        )
        var noms = recoupement.sources
        if sources.observation != nil { noms.append(WeatherProviders.brightSkySource.id) }

        var forecast = AgroForecast(
            parcelle: base.parcelle,
            timezone: base.timezone,
            elevation: base.elevation,
            current: recoupement.courant,
            hourly: recoupement.heures,
            daily: recoupement.jours,
            fetchedAt: base.fetchedAt
        )
        forecast.sources = noms
        forecast.consensus = sources.consensus
        forecast.ciel = sources.ciel
        forecast.apprentissage = apprentissage
        return forecast
    }

    /// Une prévision déjà recoupée, à qui le radar arrive après coup : ce
    /// qu'il voit tomber sur la ville s'impose à l'instant et à l'heure en
    /// cours. (Il ne peut pas retirer ici une pluie que l'aéroport a mise : le
    /// direct, qui recoupe tout d'un coup, le fera à son premier message.)
    static func appliquer(_ radar: RadarPrevision, a forecast: AgroForecast) -> AgroForecast {
        guard let tombe = Radar.tombe(radar.maintenant) else { return forecast }
        let r = Fusion.recouper(
            heures: forecast.hourly, jours: forecast.daily, courant: forecast.current,
            series: [], observation: FusionObservation(temperature: nil, tombe: tombe)
        )
        var vu = AgroForecast(
            parcelle: forecast.parcelle, timezone: forecast.timezone, elevation: forecast.elevation,
            current: r.courant, hourly: r.heures, daily: r.jours, fetchedAt: forecast.fetchedAt
        )
        vu.sources = forecast.sources
        vu.consensus = forecast.consensus
        vu.ciel = forecast.ciel
        vu.apprentissage = forecast.apprentissage
        return vu
    }

    /// La prévision de base : le meilleur modèle qu'Open-Meteo choisit pour le
    /// point. Un premier jet, que les sources refont.
    func previsionDeBase(for parcelle: Parcelle, days: Int = 7) async throws -> AgroForecast {
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

        let cache = Self.fichierDeBase(parcelle, days: days)
        do {
            let data = try await Self.lireAvecRelais(components, session: session)
            let payload = try Self.decoder(ForecastPayload.self, data)
            try? data.write(to: cache, options: .atomic)
            return Self.base(payload, parcelle: parcelle)
        } catch {
            // Ni Open-Meteo ni le relais : MET Norway, qui ne dépend pas
            // d'eux, fait la prévision de base — moins détaillée (ni UV ni
            // lever du soleil), mais de l'heure.
            // MET Norway date en UTC sans dire le fuseau : celui de la ville
            // vient de la dernière réponse d'Open-Meteo gardée, quel que soit
            // son âge — un fuseau ne change pas.
            let zone = (try? Data(contentsOf: cache)).flatMap(Self.fuseauGarde) ?? .current
            if let met = try? await WeatherProviders.metNorwayPayload(parcelle, session),
               let deSecours = Self.baseMetNorway(met, parcelle: parcelle, zone: zone) {
                return deSecours
            }
            // Puis la dernière prévision reçue pour cette ville, si elle a
            // moins de douze heures, plutôt qu'un écran vide. Son heure de
            // réception est la sienne : l'écran le dit.
            guard let data = try? Data(contentsOf: cache),
                  let date = (try? FileManager.default.attributesOfItem(atPath: cache.path))?[.modificationDate] as? Date,
                  Date().timeIntervalSince(date) < 12 * 3600,
                  let payload = try? Self.decoder(ForecastPayload.self, data)
            else { throw error }
            // Coupée à l'heure qu'il est, pas à celle de sa réception : sinon
            // le bandeau et la pluie partaient de midi à vingt-deux heures,
            // et « aujourd'hui » se posait sur la veille passé minuit.
            return Self.base(payload, parcelle: parcelle, fetchedAt: date, maintenant: Date())
        }
    }

    /// La prévision de base faite par MET Norway seul, quand Open-Meteo ne
    /// répond plus. Horaire sur les deux à trois jours où MET Norway l'est ;
    /// les jours se résument d'après ces heures. Sa forme complète donne les
    /// rafales, la probabilité de pluie et le point de rosée : ils sont lus
    /// tels quels. Ce qu'une réponse ne donne pas n'est pas inventé : l'UV
    /// reste inconnu (pas « 0 · faible » sous un soleil de midi ; MET Norway
    /// n'en donne qu'un par ciel clair), sans rafale annoncée elles valent le
    /// vent moyen, sans probabilité le risque vient du cumul annoncé.
    /// Les heures et les jours sont ceux de la ville (`zone`) : le fuseau de
    /// l'appareil n'est qu'un dernier recours, quand aucune réponse d'Open-Meteo
    /// n'a jamais été gardée pour elle.
    static func baseMetNorway(_ payload: MetPayload, parcelle: Parcelle, zone: TimeZone = .current) -> AgroForecast? {
        var calendrier = Calendar(identifier: .gregorian)
        calendrier.timeZone = zone
        let debut = Date(timeIntervalSince1970: (Date().timeIntervalSince1970 / 3600).rounded(.down) * 3600)

        let heures: [HourlySample] = (payload.properties?.timeseries ?? []).compactMap { entree in
            guard let time = entree.time, time >= debut,
                  let instant = entree.data?.instant?.details,
                  let temperature = instant["air_temperature"],
                  let prochaine = entree.data?.next_1_hours
            else { return nil }
            let symbole = prochaine.summary?.symbol_code ?? ""
            let pluie = prochaine.details?["precipitation_amount"] ?? 0
            let vent = (instant["wind_speed"] ?? 0) * 3.6
            // Une rafale n'est jamais plus faible que le vent moyen.
            let rafales = max(vent, (instant["wind_speed_of_gust"] ?? 0) * 3.6)
            let humidite = instant["relative_humidity"] ?? 0
            let probabilite: Double = prochaine.details?["probability_of_precipitation"]
                .map { min(max($0, 0), 100) }
                ?? (pluie >= 0.1 ? 70 : (pluie > 0 ? 30 : 5))
            let heureLocale = calendrier.component(.hour, from: time)
            let jour = symbole.hasSuffix("_day") || (!symbole.hasSuffix("_night") && (7..<19).contains(heureLocale))
            return HourlySample(
                time: time,
                weatherCode: WeatherProviders.codeDuSymbole(symbole) ?? (pluie >= 0.1 ? 61 : 2),
                isDay: jour,
                precipitationProbability: probabilite,
                temperature: temperature,
                apparentTemperature: Self.ressenti(temperature, vent: vent),
                relativeHumidity: humidite,
                dewPoint: instant["dew_point_temperature"] ?? Self.pointDeRosee(temperature, humidite: humidite),
                precipitation: pluie,
                windSpeed: vent,
                windGusts: rafales,
                uvIndex: nil
            )
        }
        guard let premiere = heures.first else { return nil }

        let parJour = Dictionary(grouping: heures) { calendrier.startOfDay(for: $0.time) }
        let aujourdhui = calendrier.startOfDay(for: premiere.time)
        let jours: [DailySample] = parJour.keys.sorted().compactMap { date in
            guard let h = parJour[date], let derniere = h.last else { return nil }
            // La série horaire de MET Norway s'arrête en cours de journée :
            // un jour dont on n'a que la nuit afficherait un maximum de 4 °C
            // et pas de pluie pour une après-midi douce et mouillée. Seul
            // aujourd'hui se résume en partie — depuis maintenant — ; un jour
            // suivant n'entre que couvert jusqu'à minuit.
            if date > aujourdhui,
               let finDuJour = calendrier.date(byAdding: .day, value: 1, to: date),
               derniere.time.addingTimeInterval(3600) < finDuJour {
                return nil
            }
            let mouillees = h.filter { $0.weatherCode >= 51 }
            let code = mouillees.map(\.weatherCode).max()
                ?? Dictionary(grouping: h, by: \.weatherCode).max { $0.value.count < $1.value.count }?.key ?? 2
            return DailySample(
                date: date, weatherCode: code,
                temperatureMin: h.map(\.temperature).min() ?? premiere.temperature,
                temperatureMax: h.map(\.temperature).max() ?? premiere.temperature,
                precipitationSum: h.map(\.precipitation).reduce(0, +),
                precipitationProbabilityMax: h.map(\.precipitationProbability).max() ?? 0,
                windGustsMax: h.map(\.windGusts).max() ?? 0,
                uvIndexMax: nil, sunrise: nil, sunset: nil
            )
        }
        let pression = payload.nearest(to: premiere.time)?.data?.instant?.details?["air_pressure_at_sea_level"] ?? 0
        let courant = CurrentSample(
            time: premiere.time, temperature: premiere.temperature,
            apparentTemperature: premiere.apparentTemperature, weatherCode: premiere.weatherCode,
            isDay: premiere.isDay, relativeHumidity: premiere.relativeHumidity,
            windSpeed: premiere.windSpeed, windGusts: premiere.windGusts, pressure: pression
        )
        return AgroForecast(
            parcelle: parcelle, timezone: zone.identifier, elevation: 0,
            current: courant, hourly: heures, daily: jours, fetchedAt: Date()
        )
    }

    /// Le refroidissement éolien sous 10 °C avec du vent, la température
    /// sinon — la formule d'Environnement Canada.
    static func ressenti(_ t: Double, vent v: Double) -> Double {
        guard t <= 10, v > 4.8 else { return t }
        let p = pow(v, 0.16)
        return 13.12 + 0.6215 * t - 11.37 * p + 0.3965 * t * p
    }

    /// Le point de rosée par l'approximation de Magnus.
    static func pointDeRosee(_ t: Double, humidite: Double) -> Double {
        guard humidite > 0 else { return t }
        let a = 17.62, b = 243.12
        let g = log(humidite / 100) + a * t / (b + t)
        return b * g / (a - g)
    }

    /// Le fuseau de la ville lu dans une réponse d'Open-Meteo gardée.
    static func fuseauGarde(_ data: Data) -> TimeZone? {
        struct Fuseau: Decodable { let timezone: String }
        guard let fuseau = try? JSONDecoder().decode(Fuseau.self, from: data) else { return nil }
        return TimeZone(identifier: fuseau.timezone)
    }

    /// Où garder la dernière prévision de base reçue pour une ville.
    static func fichierDeBase(_ parcelle: Parcelle, days: Int) -> URL {
        let dossier = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0]
        let nom = String(format: "base-%.3f_%.3f-%d.json", parcelle.latitude, parcelle.longitude, days)
        return dossier.appendingPathComponent(nom)
    }

    /// L'adresse du relais, quand ce binaire en a une (l'application ; ni le
    /// widget ni la montre).
    private static var relais: URL? {
        guard let valeur = Bundle.main.object(forInfoDictionaryKey: "KliimaRelay") as? String,
              !valeur.trimmingCharacters(in: .whitespaces).isEmpty
        else { return nil }
        return URL(string: valeur.trimmingCharacters(in: .whitespaces))
    }

    /// Open-Meteo en direct, en réessayant une panne passagère ; puis, s'il
    /// se tait, la même question au relais, qui garde ses réponses et en sert
    /// une un peu ancienne quand le fournisseur ne répond plus.
    static func lireAvecRelais(_ components: URLComponents, session: URLSession) async throws -> Data {
        do {
            return try await lire(components.url!, session: session, essais: 3)
        } catch let erreur as AgroWeatherError {
            guard let relais, var parRelais = URLComponents(url: relais, resolvingAgainstBaseURL: false) else {
                throw erreur
            }
            let chemin = parRelais.path.hasSuffix("/") ? String(parRelais.path.dropLast()) : parRelais.path
            parRelais.path = chemin + "/v1/open-meteo/forecast"
            parRelais.queryItems = components.queryItems
            guard let url = parRelais.url, let data = try? await lire(url, session: session, essais: 1) else {
                throw erreur
            }
            return data
        }
    }

    /// Une lecture, réessayée après une seconde puis trois quand le service
    /// ne répond pas ou se dit surchargé (429, 5xx). Un refus franc (400…)
    /// ne se réessaie pas.
    static func lire(_ url: URL, session: URLSession, essais: Int) async throws -> Data {
        var derniere: AgroWeatherError = .unreachable
        for essai in 0..<max(essais, 1) {
            if essai > 0 {
                try await Task.sleep(nanoseconds: essai == 1 ? 1_000_000_000 : 3_000_000_000)
            }
            let data: Data
            let response: URLResponse
            // Huit secondes : Open-Meteo en panne ne répond pas du tout, et la
            // minute par défaut, trois fois, laissait l'écran vide trois minutes.
            var requete = URLRequest(url: url)
            requete.timeoutInterval = 8
            do {
                (data, response) = try await session.data(for: requete)
            } catch let error as URLError where error.code == .cancelled {
                throw error
            } catch {
                derniere = .unreachable
                continue
            }
            if let http = response as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
                derniere = .badStatus(http.statusCode)
                if http.statusCode == 429 || http.statusCode >= 500 { continue }
                throw derniere
            }
            return data
        }
        throw derniere
    }

    private static func decoder<T: Decodable>(_ type: T.Type, _ data: Data) throws -> T {
        do {
            return try JSONDecoder().decode(type, from: data)
        } catch {
            throw AgroWeatherError.malformedResponse
        }
    }

    /// Lit une prévision de base reçue telle quelle — du direct, par exemple.
    static func decodeBase(_ data: Data, parcelle: Parcelle) throws -> AgroForecast {
        do {
            return base(try JSONDecoder().decode(ForecastPayload.self, from: data), parcelle: parcelle)
        } catch {
            throw AgroWeatherError.malformedResponse
        }
    }

    /// `maintenant` : l'heure qu'il est, pour une réponse gardée en mémoire —
    /// ses heures et ses jours passés sont retirés. Sans lui, la réponse est
    /// fraîche et son propre « maintenant » fait foi.
    private static func base(
        _ payload: ForecastPayload, parcelle: Parcelle, fetchedAt: Date = Date(), maintenant: Date? = nil
    ) -> AgroForecast {
        let zone = TimeZone(identifier: payload.timezone) ?? .current
        let current = payload.current.decode(in: zone)
        let jours = payload.daily.decode(in: zone)

        return AgroForecast(
            parcelle: parcelle,
            timezone: payload.timezone,
            elevation: payload.elevation,
            current: current,
            // L'API renvoie la journée entière depuis minuit : on repart de
            // l'heure en cours, pour que « maintenant » soit bien le premier
            // élément des séries.
            hourly: fromCurrentHour(payload.hourly.decode(in: zone), now: max(current.time, maintenant ?? current.time)),
            daily: maintenant.map { fromToday(jours, now: $0, in: zone) } ?? jours,
            fetchedAt: fetchedAt
        )
    }

    /// Retire les jours déjà finis, comptés à l'heure de la ville : le premier
    /// jour gardé est celui que l'écran appelle « aujourd'hui ».
    static func fromToday(_ days: [DailySample], now: Date, in zone: TimeZone) -> [DailySample] {
        var calendrier = Calendar(identifier: .gregorian)
        calendrier.timeZone = zone
        let today = calendrier.startOfDay(for: now)
        let kept = days.filter { $0.date >= today }
        // Si aujourd'hui sort de la série, on la garde telle quelle.
        return kept.isEmpty ? days : kept
    }

    /// Coupe la série horaire au début de l'heure en cours.
    ///
    /// L'heure en cours est celle qui contient `now`, pas celle qui suit
    /// l'heure ronde en temps universel : à 9 h 40 à Bombay (UTC+5:30),
    /// l'heure de 9 h commence à 4 h 30 UTC, avant 4 h, et la série s'ouvrait
    /// sur 10 h — « Maintenant » montrait l'heure suivante. Même chose à
    /// Adélaïde, Terre-Neuve ou Téhéran, et au quart d'heure à Katmandou.
    static func fromCurrentHour(_ hours: [HourlySample], now: Date) -> [HourlySample] {
        // Une série qui commence plus tard est tout entière à venir.
        guard let derniere = hours.lastIndex(where: { $0.time <= now }) else { return hours }
        // Un trou dans la série : on repart de l'heure qui suit.
        let debut = now < hours[derniere].time.addingTimeInterval(3600) ? derniere : derniere + 1
        // Si l'heure courante sort de la série, on garde la série telle quelle.
        return debut < hours.count ? Array(hours[debut...]) : hours
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

    /// La prévision au quart d'heure, pour le guetteur : le quart entamé et
    /// les dix suivants. Un appel à part de la prévision horaire : elle se
    /// relit tous les quarts d'heure, celle-là non. `nil` si la réponse n'a pas
    /// de série — le guetteur se tait plutôt que de dire « sec ».
    func quarts(for parcelle: Parcelle) async throws -> [QuartSample]? {
        var components = URLComponents(url: Self.forecastURL, resolvingAgainstBaseURL: false)!
        components.queryItems = [
            URLQueryItem(name: "latitude", value: String(format: "%.4f", parcelle.latitude)),
            URLQueryItem(name: "longitude", value: String(format: "%.4f", parcelle.longitude)),
            URLQueryItem(name: "minutely_15", value: Self.quartVariables.joined(separator: ",")),
            URLQueryItem(name: "past_minutely_15", value: "1"),
            URLQueryItem(name: "forecast_minutely_15", value: "10"),
            URLQueryItem(name: "wind_speed_unit", value: "kmh"),
            URLQueryItem(name: "timezone", value: "auto"),
        ]
        let (data, response) = try await session.data(from: components.url!)
        if let http = response as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
            throw AgroWeatherError.badStatus(http.statusCode)
        }
        return Self.decodeQuarts(data)
    }

    /// Lit la réponse au quart d'heure, dans le fuseau de la ville. Séparé de
    /// l'appel pour être testé.
    static func decodeQuarts(_ data: Data) -> [QuartSample]? {
        guard
            let payload = try? JSONDecoder().decode(QuartsPayload.self, from: data),
            let block = payload.minutely15
        else { return nil }
        let zone = TimeZone(identifier: payload.timezone) ?? .current
        let quarts = block.decode(in: zone)
        return quarts.isEmpty ? nil : quarts
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
/// La réponse au quart d'heure.
struct QuartsPayload: Decodable {
    let timezone: String
    let minutely15: Block?

    enum CodingKeys: String, CodingKey {
        case timezone
        case minutely15 = "minutely_15"
    }

    struct Block: Decodable {
        let time: [String]
        let precipitation: [Double?]?
        let weatherCode: [Int?]?
        let temperature2m: [Double?]?
        let apparentTemperature: [Double?]?
        let windGusts10m: [Double?]?
        let isDay: [Int?]?

        enum CodingKeys: String, CodingKey {
            case time
            case precipitation
            case weatherCode = "weather_code"
            case temperature2m = "temperature_2m"
            case apparentTemperature = "apparent_temperature"
            case windGusts10m = "wind_gusts_10m"
            case isDay = "is_day"
        }

        func decode(in zone: TimeZone) -> [QuartSample] {
            let formatter = DateFormatter.openMeteo(format: "yyyy-MM-dd'T'HH:mm", zone: zone)
            let rain = column(precipitation)
            let codes = intColumn(weatherCode)
            let temperature = column(temperature2m)
            let apparent = column(apparentTemperature)
            let gusts = column(windGusts10m)
            let day = intColumn(isDay)

            return time.enumerated().compactMap { index, stamp in
                guard let date = formatter.date(from: stamp) else { return nil }
                return QuartSample(
                    time: date,
                    precipitation: rain(index),
                    weatherCode: codes(index) ?? 3,
                    temperature: temperature(index),
                    apparentTemperature: apparent(index),
                    windGusts: gusts(index),
                    isDay: (day(index) ?? 1) == 1
                )
            }
        }
    }
}

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
