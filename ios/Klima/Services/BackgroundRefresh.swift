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
    static let identifier = "com.klima.app.refresh"

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

        let parcelle = SharedStore.loadParcelle() ?? .chartres
        guard let forecast = try? await AgroWeatherService().forecast(for: parcelle, days: 2) else {
            return
        }

        let summary = AgroIndicators.summarize(hours: forecast.hourly, days: forecast.daily)
        await updateActivities(hours: forecast.hourly, opportunity: summary.nextSpray)

        // Le réveil est aussi le moment d'examiner ce qu'il y a à dire. Le
        // palier vient du stockage partagé plutôt que de StoreKit : interroger
        // la boutique depuis une tâche de fond serait lent et inutile, l'écran
        // d'achat s'en chargeant à chaque ouverture.
        let state = await AlertScheduler.schedule(
            summary: summary,
            hours: forecast.hourly,
            plan: SharedStore.loadPlan(),
            state: SharedStore.loadAlertState()
        )
        SharedStore.save(state)

        #if canImport(WidgetKit)
        WidgetCenter.shared.reloadAllTimelines()
        #endif
    }

    /// Met à jour les activités en cours, sans passer par le contrôleur :
    /// l'application peut avoir été fermée depuis leur démarrage.
    private static func updateActivities(hours: [HourlySample], opportunity: SprayOpportunity?) async {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *) else { return }

        for activity in Activity<SprayActivityAttributes>.activities {
            guard let opportunity, opportunity.end > Date() else {
                await activity.end(nil, dismissalPolicy: .default)
                continue
            }
            await activity.update(
                ActivityContent(
                    state: SprayActivityController.state(from: hours, at: max(opportunity.start, Date())),
                    staleDate: opportunity.end
                )
            )
        }
        #endif
    }
}
