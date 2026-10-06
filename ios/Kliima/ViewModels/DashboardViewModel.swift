import Foundation
import SwiftUI
#if canImport(WidgetKit)
import WidgetKit
#endif

/// État du tableau de bord : ville courante, prévision, sources et air.
@MainActor
final class DashboardViewModel: ObservableObject {

    @Published private(set) var parcelle: Parcelle
    @Published private(set) var forecast: AgroForecast?
    /// Recoupement des sources ; absent si la comparaison a échoué.
    @Published private(set) var consensus: Consensus?
    /// Qualité de l'air et pollens ; absente si le service n'a pas répondu.
    @Published private(set) var air: AirSample?
    /// Le guetteur : la série au quart d'heure, et quand elle a été lue.
    @Published private(set) var quarts: [QuartSample]?
    @Published private(set) var quartsLusA: Date?
    @Published private(set) var veilleEnLecture = false
    /// La ville de la série : celle d'une autre ville ne s'affiche pas.
    private var quartsDe: Parcelle?
    @Published private(set) var isLoading = false
    @Published var errorMessage: String?

    /// Les villes enregistrées, dans l'ordre choisi — fermées comprises :
    /// c'est la vue qui sépare ce que le palier ouvre.
    @Published private(set) var villes: [Parcelle] = SharedStore.loadVilles()
    /// Les conditions du moment de chaque ville enregistrée, par identifiant.
    @Published private(set) var apercus: [String: CurrentSample] = [:]

    /// Résultats de la recherche de commune.
    @Published private(set) var searchResults: [Parcelle] = []

    private let service: AgroWeatherProviding
    private let location: LocationService
    /// Le direct : ce que le relais pousse, par WebSocket.
    private let direct = DirectRelais()
    /// Le dernier corps reçu de chaque sujet, pour la ville affichée.
    private var corpsDirects: [String: Data] = [:]
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
            self.parcelle = .paris
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
                // L'accord des sources vient avec la prévision recoupée ;
                // l'air est un plus, son échec ne prive de rien.
                self.consensus = forecast.consensus
                let a = try? await service.air(for: parcelle)
                guard !Task.isCancelled else { return }
                self.air = a ?? nil
            } catch is CancellationError {
                return
            } catch {
                guard !Task.isCancelled else { return }
                self.forecast = nil
                self.consensus = nil
                self.air = nil
                self.errorMessage = (error as? LocalizedError)?.errorDescription
                    ?? Localized.text("app.error")
            }
            self.isLoading = false
        }
        loadTask = task
        await task.value
    }

    // MARK: Villes enregistrées

    /// Vrai si la ville affichée est déjà dans la liste.
    var villeAfficheeEnregistree: Bool { Villes.position(villes, parcelle) != nil }

    /// Enregistre la ville affichée, si le palier le permet.
    @discardableResult
    func enregistrerVilleAffichee(plan: Plan) -> Villes.Ajout {
        var liste = villes
        let ajout = Villes.ajouter(&liste, parcelle, plan: plan)
        if ajout == .ajoutee {
            villes = liste
            SharedStore.save(villes: liste)
            Task { await chargerApercu(parcelle) }
        }
        return ajout
    }

    func retirer(_ ville: Parcelle) {
        Villes.retirer(&villes, ville)
        SharedStore.save(villes: villes)
    }

    func deplacer(depuis: IndexSet, vers: Int) {
        villes.move(fromOffsets: depuis, toOffset: vers)
        SharedStore.save(villes: villes)
    }

    /// Les conditions du moment de chaque ville, une à une : la liste se
    /// remplit à mesure, et une ville muette n'empêche pas les autres.
    func chargerApercus(_ liste: [Parcelle]) async {
        for ville in liste {
            guard !Task.isCancelled else { return }
            await chargerApercu(ville)
        }
    }

    private func chargerApercu(_ ville: Parcelle) async {
        guard let prevision = try? await service.forecast(for: ville, days: 1) else { return }
        apercus[ville.id] = prevision.current
    }

    // MARK: Direct

    /// Suit la ville affichée en direct : le relais pousse la prévision, les
    /// sources, le quart d'heure et l'air dès qu'ils changent. Rappelé à chaque
    /// changement de ville (`.task(id:)` de la vue).
    func suivreEnDirect() {
        let suivie = parcelle
        corpsDirects = [:]
        direct.suivre(suivie, jours: 7) { [weak self] message in
            self?.recevoir(message, pour: suivie)
        }
    }

    private func recevoir(_ message: DirectRelais.Message, pour suivie: Parcelle) {
        // Ce qui était en route pour une autre ville est écarté.
        guard suivie == parcelle,
              message.latitude == suivie.latitude,
              message.longitude == suivie.longitude
        else { return }
        corpsDirects[message.sujet] = message.corps

        switch message.sujet {
        case "quarts":
            if let lus = AgroWeatherService.decodeQuarts(message.corps) {
                quarts = lus
                quartsDe = suivie
                quartsLusA = Date()
            }
        case "air":
            if let lu = AgroWeatherService.decodeAir(message.corps) {
                air = lu
            }
        default:
            // La base et les sept modèles d'abord ; MET Norway et la station
            // quand ils sont là.
            guard
                let base = corpsDirects["base"],
                corpsDirects["ensemble"] != nil,
                let prevision = try? AgroWeatherService.decodeBase(base, parcelle: suivie)
            else { return }
            let sources = WeatherProviders.ensemble(
                openMeteo: corpsDirects["ensemble"],
                met: corpsDirects["met"],
                station: corpsDirects["station"]
            )
            let recoupee = AgroWeatherService.recouper(prevision, avec: sources)
            forecast = recoupee
            consensus = recoupee.consensus
            errorMessage = nil
        }
    }

    /// La série au quart d'heure de la ville affichée, ou rien.
    var quartsDeLaVille: [QuartSample]? { quartsDe == parcelle ? quarts : nil }

    /// Le guetteur veille : il lit, puis relit à chaque quart d'heure, tant
    /// que la tâche qui l'appelle vit (`.task(id:)` de la vue, relancée au
    /// changement de ville).
    ///
    /// L'attente se fait par pas de trente secondes plutôt qu'en un long
    /// sommeil : une application mise en arrière-plan ne voit pas passer le
    /// temps, et doit relire dès qu'elle revient si l'heure est passée.
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
        let lus = try? await service.quarts(for: parcelle)
        defer { veilleEnLecture = false }
        // La ville a changé pendant la lecture : la tâche suivante relira.
        guard parcelle == self.parcelle else { return }
        if let lus {
            quarts = lus
            quartsDe = parcelle
        } else if quartsDe != parcelle {
            quarts = nil
        }
        quartsLusA = Date()
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
