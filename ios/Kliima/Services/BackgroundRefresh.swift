import BackgroundTasks
import Foundation
#if canImport(ActivityKit)
import ActivityKit
#endif
#if canImport(WidgetKit)
import WidgetKit
#endif

/// Rafraîchissement en arrière-plan, application fermée.
///
/// Sans serveur, c'est le seul moyen de tenir une activité en direct à jour :
/// on demande au système de nous réveiller, il choisit le moment selon
/// l'usage et la batterie. Le rythme n'est donc pas garanti — c'est une
/// limite d'iOS, pas un réglage. Un suivi à la minute demanderait des
/// notifications poussées, donc un serveur.
enum BackgroundRefresh {

    /// À déclarer dans `BGTaskSchedulerPermittedIdentifiers` de l'Info.plist.
    static let identifier = "com.kliima.app.refresh"

    /// Demande le prochain réveil. À rappeler après chaque exécution : une
    /// demande ne vaut que pour une fois.
    static func schedule() {
        let request = BGAppRefreshTaskRequest(identifier: identifier)
        request.earliestBeginDate = Date(timeIntervalSinceNow: 30 * 60)
        try? BGTaskScheduler.shared.submit(request)
    }

    /// Recharge la prévision, met à jour l'activité en cours et les widgets.
    static func run() async {
        // La suivante d'abord : même en cas d'échec, la chaîne continue.
        schedule()

        let parcelle = SharedStore.loadParcelle() ?? .paris
        guard let forecast = try? await AgroWeatherService().forecast(for: parcelle, days: 2) else {
            return
        }

        await updateWeatherActivities(forecast: forecast)

        // Le réveil est aussi le moment d'examiner ce qu'il y a à dire. Le
        // palier vient du stockage partagé plutôt que de StoreKit : interroger
        // la boutique depuis une tâche de fond serait lent et inutile, l'écran
        // d'achat s'en chargeant à chaque ouverture.
        let state = await AlertScheduler.schedule(
            hours: forecast.hourly,
            plan: SharedStore.loadPlan(),
            state: SharedStore.loadAlertState()
        )
        SharedStore.save(state)

        #if canImport(WidgetKit)
        WidgetCenter.shared.reloadAllTimelines()
        #endif
    }

    /// Met à jour l'activité météo, s'il y en a une — avec l'heure suivante
    /// calculée d'avance, pour qu'elle bascule seule à l'heure pile.
    private static func updateWeatherActivities(forecast: AgroForecast) async {
        await WeatherActivityController.update(forecast: forecast)
    }
}
