import Foundation
import UserNotifications

/// Pose les notifications que le domaine a jugé utiles.
///
/// La règle — quoi dire, quand, et quand se taire — vit dans `Alerts.swift`,
/// testée des deux côtés. Ce service ne fait que trois choses : demander la
/// permission, traduire une alerte en notification, et retenir ce qui est
/// parti pour ne pas le redire.
///
/// **Une limite à connaître.** Le réveil en arrière-plan est accordé par le
/// système quand il le décide : une alerte de gel calculée à vingt heures peut
/// n'être posée qu'au réveil suivant. C'est acceptable pour une fenêtre de
/// traitement annoncée plusieurs heures à l'avance ; ça ne l'est pas pour une
/// alerte qui devrait partir à la minute. La version poussée depuis le relais
/// lèvera cette limite ; en attendant, l'application ne promet que ce qu'elle
/// tient.
///
/// **Ce qui n'a jamais tourné.** Aucun appareil n'a exécuté ce fichier : il est
/// vérifié syntaxiquement, pas à l'usage.
enum AlertScheduler {

    /// Demande l'autorisation. Sans elle, on ne pose rien et on ne redemande
    /// pas : le système ne montre la question qu'une fois.
    static func requestAuthorization() async -> Bool {
        let center = UNUserNotificationCenter.current()
        let settings = await center.notificationSettings()

        switch settings.authorizationStatus {
        case .authorized, .provisional, .ephemeral:
            return true
        case .denied:
            return false
        case .notDetermined:
            return (try? await center.requestAuthorization(options: [.alert, .sound])) ?? false
        @unknown default:
            return false
        }
    }

    /// Examine la prévision et pose ce qu'il y a à dire.
    ///
    /// Renvoie l'état mis à jour, que l'appelant enregistre : c'est lui qui
    /// empêche la répétition au réveil suivant.
    @discardableResult
    static func schedule(
        summary: AgroSummary,
        hours: [HourlySample],
        plan: Plan,
        state: AlertState,
        now: Date = Date(),
        center: UNUserNotificationCenter = .current()
    ) async -> AlertState {
        // Le palier se vérifie ici, jamais dans le domaine : `Alerts` reste une
        // fonction de la météo, et rien d'autre.
        guard plan.allows(.alertes) else { return Alerts.recordSoil(state, summary.soil.state) }

        let options = AlertOptions(now: now)
        let alerts = Alerts.evaluate(summary: summary, hours: hours, state: state, options: options)

        var sent: [Alert] = []
        for alert in alerts {
            guard let at = Alerts.deferPastQuietHours(send: now, event: alert.at, options: options)
            else { continue }
            if await post(alert, at: at, now: now, center: center) { sent.append(alert) }
        }

        var next = Alerts.recordSent(state, sent, at: now)
        next = Alerts.recordSoil(next, summary.soil.state)
        return next
    }

    /// Traduit une alerte en notification et la programme.
    private static func post(
        _ alert: Alert,
        at: Date,
        now: Date,
        center: UNUserNotificationCenter
    ) async -> Bool {
        let content = UNMutableNotificationContent()
        content.title = Localized.text(alert.titleKey)
        content.body = Localized.text(alert.bodyKey, formatted(alert))
        content.sound = .default

        // Un délai nul n'est pas accepté : on pose alors la notification pour
        // la seconde suivante plutôt que de la perdre.
        let delay = max(1, at.timeIntervalSince(now))
        let request = UNNotificationRequest(
            // Une identité par nature et par heure : reposer la même alerte
            // remplace la précédente au lieu d'en empiler deux.
            identifier: "klima.\(alert.kind.rawValue).\(Int(at.timeIntervalSince1970 / 3600))",
            content: content,
            trigger: UNTimeIntervalNotificationTrigger(timeInterval: delay, repeats: false)
        )

        do {
            try await center.add(request)
            return true
        } catch {
            return false
        }
    }

    /// Met le paramètre de l'alerte dans la langue et les unités du lecteur.
    private static func formatted(_ alert: Alert) -> String {
        switch alert.kind {
        case .fenetre:
            return AgroFormat.decimal(alert.params["score"] ?? 0, decimals: 0)
        case .gel:
            return AgroFormat.decimal(alert.params["temperature"] ?? 0, decimals: 1)
        case .sol:
            return AgroFormat.percent(alert.params["moisture"] ?? 0)
        case .pluie:
            return AgroFormat.decimal(alert.params["rain"] ?? 0, decimals: 1)
        }
    }
}
