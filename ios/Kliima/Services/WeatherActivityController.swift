import Foundation
#if canImport(ActivityKit)
import ActivityKit
#endif

/// Démarre, met à jour et termine l'activité en direct de la météo.
///
/// **Le système est la seule source de vérité.** Une activité survit au
/// processus qui l'a ouverte : l'application fermée puis rouverte, le
/// contrôleur d'avant gardait une référence en mémoire qui n'existait plus. Il
/// se croyait éteint alors que l'île affichait encore la météo — il ne la
/// mettait donc plus à jour, et le bouton en ouvrait une seconde par-dessus.
/// Tout passe désormais par `Activity.activities`, qu'iOS tient à jour.
@MainActor
final class WeatherActivityController: ObservableObject {

    /// Vrai quand une activité météo est affichée, d'où qu'elle vienne.
    @Published private(set) var isRunning = false

    init() {
        isRunning = Self.hasActivities
        Self.suivreLesJetons()
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

    private static var hasActivities: Bool {
        #if canImport(ActivityKit)
        if #available(iOS 16.2, *) {
            return !Activity<WeatherActivityAttributes>.activities.isEmpty
        }
        #endif
        return false
    }

    /// Ouvre le suivi de la météo d'une parcelle — une seule fois.
    func start(parcelle: Parcelle, forecast: AgroForecast) {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *), isAvailable else { return }
        // Déjà affichée : on la met à jour plutôt que d'en empiler une autre.
        guard !Self.hasActivities else {
            isRunning = true
            Task { await refresh(forecast: forecast) }
            return
        }

        let attributes = WeatherActivityAttributes(
            parcelleName: parcelle.name,
            timeZoneIdentifier: forecast.timezone,
            latitude: parcelle.latitude,
            longitude: parcelle.longitude
        )
        let now = Date()
        let content = ActivityContent(
            state: Self.state(from: forecast, at: now),
            staleDate: Self.staleDate(for: forecast, at: now)
        )
        // Avec un jeton de poussée quand il y a un relais pour s'en servir.
        // Si iOS refuse — droit de poussée absent de la signature, par
        // exemple —, on retente sans : une île qui bascule seule vaut mieux
        // que pas d'île du tout.
        let activity: Activity<WeatherActivityAttributes>?
        if PlanGrant.relayURL != nil,
           let avecPoussee = try? Activity.request(attributes: attributes, content: content, pushType: .token) {
            activity = avecPoussee
        } else {
            activity = try? Activity.request(attributes: attributes, content: content, pushType: nil)
        }
        // Quota atteint ou activités refusées : l'application continue sans.
        isRunning = activity != nil || Self.hasActivities
        if let activity { Self.suivre(activity) }
        #endif
    }

    /// Reflète la prévision fraîchement chargée sur toute activité affichée.
    func refresh(forecast: AgroForecast) async {
        await Self.update(forecast: forecast)
        isRunning = Self.hasActivities
    }

    /// Ferme toutes les activités météo, y compris celles d'un lancement précédent.
    func stop() async {
        #if canImport(ActivityKit)
        if #available(iOS 16.2, *) {
            for activity in Activity<WeatherActivityAttributes>.activities {
                // Le relais d'abord : il cesse de pousser vers une île fermée.
                if let jeton = activity.pushToken {
                    await IlesRelais.retirer(jeton: jeton)
                }
                await activity.end(nil, dismissalPolicy: .immediate)
            }
        }
        #endif
        isRunning = false
    }

    /// Met à jour toutes les activités météo affichées.
    ///
    /// Statique : le réveil d'arrière-plan s'en sert sans instance,
    /// l'application pouvant être fermée.
    static func update(forecast: AgroForecast, at now: Date = Date()) async {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *) else { return }
        suivreLesJetons()
        for activity in Activity<WeatherActivityAttributes>.activities {
            await activity.update(ActivityContent(
                state: state(from: forecast, at: now),
                staleDate: staleDate(for: forecast, at: now)
            ))
        }
        #endif
    }

    // MARK: Poussée par le relais

    /// Les activités dont on écoute déjà le jeton.
    private static var suivies = Set<String>()

    /// Écoute le jeton de chaque activité affichée, y compris celles d'un
    /// lancement précédent : une activité survit à l'application, et son
    /// inscription au relais — en mémoire, sur le serveur — peut ne pas avoir
    /// survécu à un redémarrage de celui-ci.
    static func suivreLesJetons() {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *) else { return }
        for activity in Activity<WeatherActivityAttributes>.activities {
            suivre(activity)
        }
        #endif
    }

    #if canImport(ActivityKit)
    /// Inscrit l'activité au relais, et la réinscrit à chaque jeton neuf :
    /// Apple peut en changer au cours de la vie de l'activité.
    @available(iOS 16.2, *)
    private static func suivre(_ activity: Activity<WeatherActivityAttributes>) {
        guard PlanGrant.relayURL != nil,
              let latitude = activity.attributes.latitude,
              let longitude = activity.attributes.longitude,
              Self.suivies.insert(activity.id).inserted
        else { return }

        Task {
            for await jeton in activity.pushTokenUpdates {
                await IlesRelais.inscrire(jeton: jeton, latitude: latitude, longitude: longitude)
            }
            // La file se tarit quand l'activité se termine.
            Self.suivies.remove(activity.id)
        }
    }
    #endif

    /// Le moment où ce qui est affiché cesse d'être l'heure en cours : le
    /// début de l'heure suivante de la série. La vue montre alors l'heure
    /// suivante, calculée d'avance — voir `WeatherActivityAttributes`.
    static func staleDate(for forecast: AgroForecast, at now: Date) -> Date {
        Horizon.prochaineBascule(apres: now, dans: forecast.hourly)
            ?? now.addingTimeInterval(3600)
    }

    /// L'état affichable : les conditions mesurées, et l'heure qui vient.
    static func state(from forecast: AgroForecast, at now: Date) -> WeatherActivityAttributes.ContentState {
        let today = Horizon.jour(contenant: now, dans: forecast.daily)
        let next = Horizon.heureSuivante(apres: now, dans: forecast.hourly).map {
            WeatherActivityAttributes.NextHour(
                start: $0.time,
                temperature: $0.temperature,
                weatherCode: $0.weatherCode,
                isDay: $0.isDay,
                precipitationProbability: $0.precipitationProbability,
                windSpeed: $0.windSpeed
            )
        }
        return WeatherActivityAttributes.ContentState(
            temperature: forecast.current.temperature,
            apparentTemperature: forecast.current.apparentTemperature,
            weatherCode: forecast.current.weatherCode,
            isDay: forecast.current.isDay,
            windSpeed: forecast.current.windSpeed,
            temperatureMin: today?.temperatureMin ?? forecast.current.temperature,
            temperatureMax: today?.temperatureMax ?? forecast.current.temperature,
            updatedAt: now,
            next: next
        )
    }
}
