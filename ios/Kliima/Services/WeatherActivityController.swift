import Foundation
#if canImport(ActivityKit)
import ActivityKit
#endif

/// Démarre, met à jour et termine l'activité en direct de la météo.
///
/// Comme celle de la fenêtre de traitement, elle se met à jour depuis
/// l'application et depuis le réveil d'arrière-plan — sans serveur, il n'y a
/// pas de suivi à la minute.
///
/// Elle n'a en revanche pas de fin naturelle : c'est l'utilisateur qui
/// l'ouvre et la ferme, et iOS la termine de lui-même au bout de huit heures
/// environ. Il n'y a donc rien ici qui ressemble au ménage des activités
/// passées — il n'y a pas d'heure à dépasser.
@MainActor
final class WeatherActivityController: ObservableObject {

    /// Vrai quand une activité météo est en cours.
    @Published private(set) var isRunning = false

    #if canImport(ActivityKit)
    private var activity: Activity<WeatherActivityAttributes>?
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

    /// Ouvre le suivi de la météo d'une parcelle.
    func start(parcelle: Parcelle, forecast: AgroForecast) {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *), isAvailable, activity == nil else { return }

        let attributes = WeatherActivityAttributes(
            parcelleName: parcelle.name,
            timeZoneIdentifier: forecast.timezone
        )

        do {
            activity = try Activity.request(
                attributes: attributes,
                content: ActivityContent(
                    state: Self.state(from: forecast),
                    staleDate: Date(timeIntervalSinceNow: 3600)
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
    func refresh(forecast: AgroForecast) async {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *), let activity else { return }
        await activity.update(
            ActivityContent(
                state: Self.state(from: forecast),
                staleDate: Date(timeIntervalSinceNow: 3600)
            )
        )
        #endif
    }

    /// Ferme l'activité.
    func stop() async {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *), let activity else { return }
        await activity.end(nil, dismissalPolicy: .immediate)
        self.activity = nil
        #endif
        isRunning = false
    }

    /// L'état affichable d'une prévision.
    ///
    /// Statique : le réveil d'arrière-plan la réutilise sans instance,
    /// l'application pouvant être fermée.
    static func state(from forecast: AgroForecast) -> WeatherActivityAttributes.ContentState {
        let today = forecast.daily.first
        return WeatherActivityAttributes.ContentState(
            temperature: forecast.current.temperature,
            apparentTemperature: forecast.current.apparentTemperature,
            weatherCode: forecast.current.weatherCode,
            isDay: forecast.current.isDay,
            windSpeed: forecast.current.windSpeed,
            temperatureMin: today?.temperatureMin ?? forecast.current.temperature,
            temperatureMax: today?.temperatureMax ?? forecast.current.temperature,
            updatedAt: Date()
        )
    }
}
