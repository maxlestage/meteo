import Foundation

/// État de l'application montre.
///
/// La montre est autonome : elle interroge l'API elle-même et se cale sur la
/// position du poignet, sans passer par le téléphone. Au champ, c'est souvent
/// le seul écran qu'on sort.
@MainActor
final class WatchViewModel: ObservableObject {

    @Published private(set) var parcelle: Parcelle = .chartres
    @Published private(set) var forecast: AgroForecast?
    @Published private(set) var summary: AgroSummary?
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

    /// Fuseau de la parcelle, pour dater les créneaux affichés.
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

        // La position est un confort : sans elle, on garde la parcelle par défaut.
        if let coordinate = try? await location.currentCoordinate() {
            parcelle = Parcelle(
                name: Localized.text("search.myField"),
                latitude: coordinate.latitude,
                longitude: coordinate.longitude
            )
        }

        do {
            // Deux jours suffisent au poignet : aujourd'hui et la nuit qui suit.
            let forecast = try await service.forecast(for: parcelle, days: 2)
            self.forecast = forecast
            self.summary = AgroIndicators.summarize(hours: forecast.hourly, days: forecast.daily)
        } catch {
            self.forecast = nil
            self.summary = nil
            self.errorMessage = (error as? LocalizedError)?.errorDescription
                ?? Localized.text("app.error")
        }

        isLoading = false
    }
}
