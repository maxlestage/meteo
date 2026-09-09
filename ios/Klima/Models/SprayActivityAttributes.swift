import Foundation

/// Données de l'activité en direct qui suit une fenêtre de traitement.
///
/// La fenêtre est le seul élément vraiment vivant de Klima : elle a un début,
/// une fin, et des conditions qui peuvent se dégrader entre-temps. C'est donc
/// elle que l'écran verrouillé et l'île dynamique affichent.
struct SprayActivityAttributes: Codable, Hashable {

    /// Ce qui évolue pendant la fenêtre.
    struct ContentState: Codable, Hashable {
        /// Verdict courant : la fenêtre peut se dégrader après son ouverture.
        var verdict: SprayVerdict
        /// Score 0–100 des conditions du moment.
        var score: Int
        var windSpeed: Double
        var windGusts: Double
        /// Premier motif de blocage, s'il y en a un.
        var blocker: SprayBlocker?
        /// Horodatage du relevé, pour dater l'affichage.
        var updatedAt: Date

        /// Vrai quand les conditions ne permettent plus de traiter.
        var isBlocked: Bool { verdict == .defavorable }
    }

    /// Parcelle suivie.
    var parcelleName: String
    var windowStart: Date
    var windowEnd: Date
    /// Fuseau de la parcelle : l'activité affiche ses heures, pas les nôtres.
    var timeZoneIdentifier: String

    var timeZone: TimeZone {
        TimeZone(identifier: timeZoneIdentifier) ?? .current
    }
}

#if canImport(ActivityKit)
import ActivityKit

extension SprayActivityAttributes: ActivityAttributes {}
#endif
