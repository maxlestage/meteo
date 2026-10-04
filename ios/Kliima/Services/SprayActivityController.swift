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

    /// Vrai quand une activité de traitement est affichée, d'où qu'elle vienne.
    ///
    /// Le système est la seule source de vérité : une activité survit au
    /// processus qui l'a ouverte. Une référence gardée en mémoire se perdait à
    /// la relance — le contrôleur se croyait éteint, ne mettait plus rien à
    /// jour, et en ouvrait une seconde par-dessus.
    @Published private(set) var isRunning = false

    init() {
        isRunning = Self.hasActivities
    }

    private static var hasActivities: Bool {
        #if canImport(ActivityKit)
        if #available(iOS 16.2, *) {
            return !Activity<SprayActivityAttributes>.activities.isEmpty
        }
        #endif
        return false
    }

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
        guard #available(iOS 16.2, *), isAvailable else { return }
        // Déjà affichée — par ce lancement ou un précédent : on n'en empile pas
        // une seconde.
        guard !Self.hasActivities else {
            isRunning = true
            return
        }

        let attributes = SprayActivityAttributes(
            parcelleName: parcelle.name,
            windowStart: opportunity.start,
            windowEnd: opportunity.end,
            timeZoneIdentifier: timeZone.identifier
        )

        do {
            _ = try Activity.request(
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

    /// Reflète la prévision fraîchement chargée sur toute activité affichée.
    func refresh(hours: [HourlySample], opportunity: SprayOpportunity?) async {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *) else { return }
        for activity in Activity<SprayActivityAttributes>.activities {
            // Sa propre fenêtre d'abord : une activité qui a passé son heure se
            // ferme, même si une autre occasion se présente plus tard.
            guard activity.attributes.windowEnd > Date(),
                  let opportunity, opportunity.end > Date()
            else {
                await activity.end(nil, dismissalPolicy: .default)
                continue
            }
            await activity.update(ActivityContent(
                state: Self.state(from: hours, at: max(opportunity.start, Date())),
                staleDate: opportunity.end
            ))
        }
        #endif
        isRunning = Self.hasActivities
    }

    /// Ferme toutes les activités de traitement, y compris celles d'un
    /// lancement précédent.
    func stop() async {
        #if canImport(ActivityKit)
        if #available(iOS 16.2, *) {
            for activity in Activity<SprayActivityAttributes>.activities {
                await activity.end(nil, dismissalPolicy: .immediate)
            }
        }
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

    /// Conditions de l'heure qui contient `date`.
    ///
    /// C'était la première heure *qui commence après* `date` : à 14 h 20,
    /// l'activité montrait le vent de 15 h. L'heure en cours est celle qui
    /// contient l'instant — `Horizon` le dit, pour tous les écrans.
    ///
    /// Statique : la tâche d'arrière-plan la réutilise sans passer par une
    /// instance, l'application pouvant être fermée.
    static func state(from hours: [HourlySample], at date: Date) -> SprayActivityAttributes.ContentState {
        let windows = AgroIndicators.sprayWindows(hours)
        let courante = Horizon.heure(contenant: date, dans: hours)
        let index = courante.flatMap { heure in hours.firstIndex { $0.time == heure.time } }
            ?? hours.indices.first ?? 0
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
