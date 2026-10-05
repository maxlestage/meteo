import Foundation

/// État de l'application montre.
///
/// La montre est autonome : elle interroge l'API elle-même et se cale sur la
/// position du poignet, sans passer par le téléphone. Dans la rue, c'est
/// souvent le seul écran qu'on sort.
@MainActor
final class WatchViewModel: ObservableObject {

    @Published private(set) var parcelle: Parcelle = .paris
    @Published private(set) var forecast: AgroForecast?
    @Published private(set) var isLoading = false
    @Published private(set) var errorMessage: String?

    private let service: AgroWeatherProviding
    private let location: LocationService

    init(
        service: AgroWeatherProviding = AgroWeatherService(),
        location: LocationService = LocationService()
    ) {
        self.service = service
        self.location = location
    }

    /// Fuseau de la ville, pour dater les créneaux affichés.
    var timeZone: TimeZone {
        guard let identifier = forecast?.timezone, let zone = TimeZone(identifier: identifier) else {
            return .current
        }
        return zone
    }

    /// Conditions du moment, ou la première heure disponible.
    var current: CurrentSample? { forecast?.current }

    func load() async {
        isLoading = true
        errorMessage = nil

        // La position est un confort : sans elle, on garde la ville par défaut.
        if let coordinate = try? await location.currentCoordinate() {
            parcelle = Position.parcelle(
                named: Localized.text("search.myField"),
                latitude: coordinate.latitude,
                longitude: coordinate.longitude
            )
            // La complication de cadran lira la même ville.
            SharedStore.save(parcelle)
        }

        do {
            // Deux jours suffisent au poignet : les douze heures qui viennent
            // débordent parfois sur demain.
            let forecast = try await service.forecast(for: parcelle, days: 2)
            self.forecast = forecast
        } catch {
            self.forecast = nil
            self.errorMessage = (error as? LocalizedError)?.errorDescription
                ?? Localized.text("app.error")
        }

        isLoading = false
    }
}
