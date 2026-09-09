import Foundation

/// État du tableau de bord : parcelle courante, prévision et indicateurs.
@MainActor
final class DashboardViewModel: ObservableObject {

    @Published private(set) var parcelle: Parcelle
    @Published private(set) var forecast: AgroForecast?
    @Published private(set) var summary: AgroSummary?
    @Published private(set) var isLoading = false
    @Published var errorMessage: String?

    /// Résultats de la recherche de commune.
    @Published private(set) var searchResults: [Parcelle] = []

    private let service: AgroWeatherProviding
    private let location: LocationService
    private let store: UserDefaults
    private var loadTask: Task<Void, Never>?
    private var searchTask: Task<Void, Never>?

    private static let storageKey = "klima.parcelle"

    init(
        service: AgroWeatherProviding = AgroWeatherService(),
        location: LocationService = LocationService(),
        store: UserDefaults = .standard
    ) {
        self.service = service
        self.location = location
        self.store = store
        self.parcelle = Self.storedParcelle(in: store) ?? .chartres
    }

    /// Fuseau de la parcelle, pour dater correctement les créneaux affichés.
    var timeZone: TimeZone {
        guard let identifier = forecast?.timezone, let zone = TimeZone(identifier: identifier) else {
            return .current
        }
        return zone
    }

    /// Charge la prévision. L'appel attend la fin du chargement : un « tirer
    /// pour rafraîchir » garde ainsi son indicateur jusqu'aux données reçues.
    func load() async {
        loadTask?.cancel()
        let parcelle = parcelle
        isLoading = true
        errorMessage = nil

        let task = Task {
            do {
                let forecast = try await service.forecast(for: parcelle, days: 7)
                guard !Task.isCancelled else { return }
                self.forecast = forecast
                self.summary = AgroIndicators.summarize(hours: forecast.hourly, days: forecast.daily)
            } catch is CancellationError {
                return
            } catch {
                guard !Task.isCancelled else { return }
                self.forecast = nil
                self.summary = nil
                self.errorMessage = (error as? LocalizedError)?.errorDescription
                    ?? Localized.text("app.error")
            }
            self.isLoading = false
        }
        loadTask = task
        await task.value
    }

    func select(_ parcelle: Parcelle) {
        self.parcelle = parcelle
        searchResults = []
        persist(parcelle)
        Task { await load() }
    }

    /// Recherche différée : on laisse l'utilisateur finir de taper.
    func search(_ query: String) {
        searchTask?.cancel()
        guard query.trimmingCharacters(in: .whitespaces).count >= 2 else {
            searchResults = []
            return
        }

        searchTask = Task {
            try? await Task.sleep(nanoseconds: 250_000_000)
            guard !Task.isCancelled else { return }
            let results = (try? await service.search(commune: query)) ?? []
            guard !Task.isCancelled else { return }
            self.searchResults = results
        }
    }

    func clearSearch() {
        searchTask?.cancel()
        searchResults = []
    }

    /// Cale la prévision sur la position de l'utilisateur.
    func useCurrentLocation() {
        Task {
            do {
                let coordinate = try await location.currentCoordinate()
                select(
                    Parcelle(
                        name: Localized.text("search.myField"),
                        latitude: coordinate.latitude,
                        longitude: coordinate.longitude
                    )
                )
            } catch {
                errorMessage = (error as? LocalizedError)?.errorDescription
                    ?? Localized.text("location.unavailable")
            }
        }
    }

    // MARK: Persistance

    private func persist(_ parcelle: Parcelle) {
        guard let data = try? JSONEncoder().encode(parcelle) else { return }
        store.set(data, forKey: Self.storageKey)
    }

    private static func storedParcelle(in store: UserDefaults) -> Parcelle? {
        guard let data = store.data(forKey: storageKey) else { return nil }
        return try? JSONDecoder().decode(Parcelle.self, from: data)
    }
}
