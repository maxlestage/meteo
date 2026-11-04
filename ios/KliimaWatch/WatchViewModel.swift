import CoreLocation
import Foundation
import WidgetKit

/// État de l'application montre.
///
/// La montre est autonome : elle interroge l'API elle-même et se cale sur la
/// position du poignet, sans passer par le téléphone. Dans la rue, c'est
/// souvent le seul écran qu'on sort.
@MainActor
final class WatchViewModel: ObservableObject {

    /// La dernière ville connue plutôt que Paris : ouvrir la montre ne doit
    /// pas montrer une autre ville le temps que la position arrive.
    @Published private(set) var parcelle: Parcelle = SharedStore.loadParcelle() ?? .paris
    /// Quand la prévision affichée a été chargée.
    @Published private(set) var chargeeA: Date?
    @Published private(set) var forecast: AgroForecast?
    @Published private(set) var isLoading = false
    @Published private(set) var errorMessage: String?

    /// Le guetteur : la série au quart d'heure, et quand on l'a lue.
    @Published private(set) var quarts: [QuartSample]?
    @Published private(set) var quartsLusA: Date?
    @Published private(set) var veilleEnLecture = false
    /// La ville de la série : celle d'une autre ville ne s'affiche pas.
    private var quartsDe: Parcelle?

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
    /// Le relevé tant qu'il est de l'heure en cours, l'heure prévue ensuite.
    var current: CurrentSample? { forecast?.maintenant() }

    /// La série au quart d'heure de la ville affichée, ou rien : arrivé
    /// ailleurs, le poignet ne dit pas « sec » d'après le ciel d'une autre ville.
    var quartsDeLaVille: [QuartSample]? {
        guard let ville = forecast?.parcelle, quartsDe == ville else { return nil }
        return quarts
    }

    /// Charge la prévision, et cale la montre sur le poignet.
    ///
    /// La position ne bloque pas l'affichage — c'est la règle de l'iPhone
    /// (`DashboardViewModel.locateIfUnchosen`) : la montre attendait la
    /// position, puis le nom de la commune, avant de demander la moindre
    /// prévision. À la première ouverture, la demande d'autorisation laissait
    /// un sablier derrière elle ; ensuite, chaque relevé lent du poignet
    /// faisait attendre une ville qu'on connaissait déjà. La dernière ville
    /// connue se charge maintenant tout de suite, pendant que la position est
    /// relevée, et la prévision bascule quand celle-ci désigne une autre
    /// ville.
    func load() async {
        isLoading = true
        errorMessage = nil
        let avant = parcelle

        async let poignet = villeDuPoignet()
        await chargerPrevision()
        isLoading = false

        // La position est un confort : sans elle, on garde la ville affichée.
        if let ici = await poignet, ici != parcelle {
            parcelle = ici
            // La complication de cadran lira la même ville.
            SharedStore.save(ici)
            await chargerPrevision()
        }

        // Arrivé dans une autre ville, le guetteur la lit tout de suite
        // plutôt qu'au prochain quart.
        if parcelle != avant, quartsDe != parcelle {
            await relireQuarts()
        }
    }

    /// La ville où se trouve le poignet, arrondie à la maille et nommée par
    /// sa commune — comme les widgets de l'iPhone : « Ma ville » ne disait
    /// pas où l'on était. Rien sans position.
    private func villeDuPoignet() async -> Parcelle? {
        guard let coordinate = try? await location.currentCoordinate() else { return nil }
        let lieu = CLLocation(latitude: coordinate.latitude, longitude: coordinate.longitude)
        let commune = try? await CLGeocoder().reverseGeocodeLocation(lieu).first?.locality
        return Position.parcelle(
            named: commune ?? Localized.text("search.myField"),
            latitude: coordinate.latitude,
            longitude: coordinate.longitude
        )
    }

    /// La prévision de la ville courante. Une relecture qui échoue garde la
    /// prévision d'avant : elle reste plus juste qu'un écran vide.
    private func chargerPrevision() async {
        let ville = parcelle
        do {
            // Deux jours suffisent au poignet : les douze heures qui viennent
            // débordent parfois sur demain.
            let forecast = try await service.forecast(for: ville, days: 2)
            // La ville a changé pendant la lecture : la suivante la remplacera.
            guard ville == parcelle else { return }
            self.forecast = forecast
            chargeeA = Date()
            errorMessage = nil
            // Ce que la montre vient d'apprendre, le cadran et la pile
            // intelligente le montrent aussi, sans attendre leur prochain tour.
            WidgetCenter.shared.reloadAllTimelines()
        } catch {
            guard ville == parcelle else { return }
            self.errorMessage = (error as? LocalizedError)?.errorDescription
                ?? Localized.text("app.error")
        }
    }

    /// Le guetteur relit le ciel une minute après chaque quart, tant que
    /// l'écran est ouvert.
    func veiller() async {
        while !Task.isCancelled {
            await relireQuarts()
            let echeance = Veille.prochaineLecture(quartsLusA ?? Date())
            while !Task.isCancelled, Date() < echeance {
                try? await Task.sleep(nanoseconds: 30_000_000_000)
            }
        }
    }

    /// Une lecture du guetteur. Un échec garde la série d'avant, qui reste
    /// vraie pour les quarts qu'elle couvre encore — sauf si elle est d'une
    /// autre ville.
    func relireQuarts() async {
        let parcelle = parcelle
        veilleEnLecture = true
        defer { veilleEnLecture = false }
        let lus = try? await service.quarts(for: parcelle)
        // La ville a changé pendant la lecture : `load()` relira.
        guard parcelle == self.parcelle else { return }
        if let lus {
            quarts = lus
            quartsDe = parcelle
        } else if quartsDe != parcelle {
            quarts = nil
        }
        quartsLusA = Date()
    }
}
