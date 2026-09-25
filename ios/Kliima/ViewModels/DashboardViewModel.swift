import Foundation
#if canImport(WidgetKit)
import WidgetKit
#endif

/// État du tableau de bord : parcelle courante, prévision et indicateurs.
@MainActor
final class DashboardViewModel: ObservableObject {

    @Published private(set) var parcelle: Parcelle
    @Published private(set) var forecast: AgroForecast?
    @Published private(set) var summary: AgroSummary?
    /// Recoupement des modèles ; absent si la comparaison a échoué.
    @Published private(set) var consensus: Consensus?
    @Published private(set) var isLoading = false
    @Published var errorMessage: String?

    /// Résultats de la recherche de commune.
    @Published private(set) var searchResults: [Parcelle] = []

    private let service: AgroWeatherProviding
    private let location: LocationService
    private var loadTask: Task<Void, Never>?
    private var searchTask: Task<Void, Never>?

    /// D'où vient la parcelle affichée à l'ouverture.
    private let origin: ParcelleOrigin
    /// La position n'est demandée qu'une fois par lancement.
    private var positionDemandee = false

    init(
        service: AgroWeatherProviding = AgroWeatherService(),
        location: LocationService = LocationService()
    ) {
        self.service = service
        self.location = location
        if let memorisee = SharedStore.loadParcelle() {
            self.parcelle = memorisee
            self.origin = .memoire
        } else {
            self.parcelle = .chartres
            self.origin = .defaut
        }
    }

    /// Cale l'application sur la position de la personne, à la première
    /// ouverture seulement.
    ///
    /// Ne bloque pas l'affichage : la parcelle par défaut se charge pendant que
    /// le système demande l'autorisation, et la prévision bascule quand la
    /// position arrive. Attendre la réponse laisserait un écran vide derrière la
    /// boîte de dialogue, pour un geste que personne n'a demandé.
    ///
    /// Silencieux en cas d'échec, pour la même raison : un refus n'est pas une
    /// erreur à afficher. On garde la parcelle par défaut, et la recherche de
    /// commune reste là.
    func locateIfUnchosen() {
        guard Position.locatesOnStart(origin), !positionDemandee else { return }
        positionDemandee = true

        Task {
            guard let coordinate = try? await location.currentCoordinate() else { return }
            select(
                Position.parcelle(
                    named: Localized.text("search.myField"),
                    latitude: coordinate.latitude,
                    longitude: coordinate.longitude
                )
            )
        }
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
                // Le recoupement est un plus : son échec ne prive de rien.
                self.consensus = try? await service.modelConsensus(for: parcelle)
            } catch is CancellationError {
                return
            } catch {
                guard !Task.isCancelled else { return }
                self.forecast = nil
                self.summary = nil
                self.consensus = nil
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
                    Position.parcelle(
                        named: Localized.text("search.myField"),
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

    /// La parcelle est écrite dans le groupe partagé : le widget d'écran
    /// d'accueil la lit de son côté, et on lui demande de se redessiner.
    private func persist(_ parcelle: Parcelle) {
        SharedStore.save(parcelle)
        #if canImport(WidgetKit)
        WidgetCenter.shared.reloadAllTimelines()
        #endif
    }
}
