import Foundation
#if canImport(ActivityKit)
import ActivityKit
#endif

/// Démarre, met à jour et termine l'activité en direct de la fenêtre de
/// traitement.
///
/// Les mises à jour sont locales : elles suivent les rechargements de
/// l'application. Un suivi à la minute, application fermée, demanderait des
/// notifications poussées et donc un serveur — Kliima n'en a pas.
@MainActor
final class SprayActivityController: ObservableObject {

    /// Vrai quand une activité est en cours pour la fenêtre affichée.
    @Published private(set) var isRunning = false

    #if canImport(ActivityKit)
    private var activity: Activity<SprayActivityAttributes>?
    #endif

    /// Vrai si l'appareil et les réglages autorisent les activités en direct.
    var isAvailable: Bool {
        #if canImport(ActivityKit)
        if #available(iOS 16.2, *) {
            return ActivityAuthorizationInfo().areActivitiesEnabled
        }
        return false
        #else
        return false
        #endif
    }

    /// Ouvre le suivi d'une fenêtre à venir.
    func start(
        parcelle: Parcelle,
        opportunity: SprayOpportunity,
        timeZone: TimeZone,
        hours: [HourlySample]
    ) {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *), isAvailable, activity == nil else { return }

        // L'application a pu être tuée en laissant une activité derrière elle :
        // `activity` est alors nul ici alors que le système en affiche encore
        // une. Sans ce ménage, on en ouvre une seconde par-dessus.
        Task { await Self.endPastActivities() }

        let attributes = SprayActivityAttributes(
            parcelleName: parcelle.name,
            windowStart: opportunity.start,
            windowEnd: opportunity.end,
            timeZoneIdentifier: timeZone.identifier
        )

        do {
            activity = try Activity.request(
                attributes: attributes,
                content: ActivityContent(
                    state: Self.state(from: hours, at: opportunity.start),
                    staleDate: opportunity.end
                ),
                pushType: nil
            )
            isRunning = true
        } catch {
            // Quota atteint ou activités refusées : l'application continue sans.
            isRunning = false
        }
        #endif
    }

    /// Reflète la prévision fraîchement chargée sur l'activité en cours.
    func refresh(hours: [HourlySample], opportunity: SprayOpportunity?) async {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *), let activity else { return }

        // Sa propre fenêtre d'abord : une activité qui a passé son heure se
        // ferme, même si une autre occasion se présente plus tard.
        guard activity.attributes.windowEnd > Date() else {
            await stop()
            return
        }

        guard let opportunity, opportunity.end > Date() else {
            await stop()
            return
        }

        await activity.update(
            ActivityContent(
                state: Self.state(from: hours, at: max(opportunity.start, Date())),
                staleDate: opportunity.end
            )
        )
        #endif
    }

    /// Ferme l'activité, la fenêtre étant passée ou abandonnée.
    func stop() async {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *), let activity else { return }
        await activity.end(nil, dismissalPolicy: .immediate)
        self.activity = nil
        #endif
        isRunning = false
    }

    /// Ferme les activités dont la fenêtre est passée, où qu'elles viennent.
    ///
    /// Statique : elles survivent au processus qui les a ouvertes, et personne
    /// d'autre ne les fermera.
    static func endPastActivities() async {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *) else { return }
        for activity in Activity<SprayActivityAttributes>.activities
        where activity.attributes.windowEnd <= Date() {
            await activity.end(nil, dismissalPolicy: .default)
        }
        #endif
    }

    /// Conditions de l'heure la plus proche de `date`.
    ///
    /// Statique : la tâche d'arrière-plan la réutilise sans passer par une
    /// instance, l'application pouvant être fermée.
    static func state(from hours: [HourlySample], at date: Date) -> SprayActivityAttributes.ContentState {
        let windows = AgroIndicators.sprayWindows(hours)
        let index = hours.firstIndex { $0.time >= date } ?? hours.indices.first ?? 0
        let hour = hours.indices.contains(index) ? hours[index] : nil
        let window = windows.indices.contains(index) ? windows[index] : nil

        return SprayActivityAttributes.ContentState(
            verdict: window?.verdict ?? .defavorable,
            score: window?.score ?? 0,
            windSpeed: hour?.windSpeed ?? 0,
            windGusts: hour?.windGusts ?? 0,
            blocker: window?.blockers.first,
            updatedAt: Date()
        )
    }
}
