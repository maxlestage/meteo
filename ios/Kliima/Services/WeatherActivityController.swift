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

    /// Vrai si une activité est encore à l'écran. Une activité qu'on vient de
    /// fermer reste quelques instants dans `Activity.activities` : la compter
    /// empêchait de la rouvrir, et l'île disparaissait sans revenir.
    private static var hasActivities: Bool {
        #if canImport(ActivityKit)
        if #available(iOS 16.2, *) {
            return !vivantes.isEmpty
        }
        #endif
        return false
    }

    #if canImport(ActivityKit)
    @available(iOS 16.2, *)
    private static var vivantes: [Activity<WeatherActivityAttributes>] {
        Activity<WeatherActivityAttributes>.activities.filter {
            $0.activityState == .active || $0.activityState == .stale
        }
    }
    #endif

    // MARK: Ce que la personne a demandé

    /// iOS termine une activité en direct au bout de huit heures, sans rien
    /// dire. On se souvient donc qu'elle a été demandée, et quand : à la
    /// prochaine ouverture, une activité que le système a fermée revient
    /// d'elle-même, et une activité de plus de six heures est rouverte avant
    /// d'être coupée. Une activité retirée à la main avant ses huit heures, en
    /// la balayant de l'écran verrouillé, ne revient pas.
    private static let demandeeCle = "kliima.activite.demandee"
    private static let ouverteCle = "kliima.activite.ouverteLe"
    /// Ce que dure une activité avant qu'iOS la coupe, à quelques minutes près.
    static let dureeMax: TimeInterval = 8 * 3600
    /// Au-delà, on la rouvre à la prochaine ouverture plutôt que de la
    /// laisser s'éteindre.
    static let renouveler: TimeInterval = 6 * 3600

    private static var demandee: Bool {
        get { UserDefaults.standard.bool(forKey: demandeeCle) }
        set { UserDefaults.standard.set(newValue, forKey: demandeeCle) }
    }

    private static var ouverteLe: Date? {
        get { UserDefaults.standard.object(forKey: ouverteCle) as? Date }
        set { UserDefaults.standard.set(newValue, forKey: ouverteCle) }
    }

    /// Ce qu'il faut faire d'une activité demandée, selon ce qu'iOS montre
    /// encore et depuis quand elle est ouverte. Pur, pour être testé.
    enum Suite: Equatable { case rien, rouvrir, oublier }

    static func suite(demandee: Bool, affichee: Bool, ouverteLe: Date?, maintenant: Date) -> Suite {
        guard demandee else { return .rien }
        let age = ouverteLe.map { maintenant.timeIntervalSince($0) } ?? dureeMax
        if affichee { return age >= renouveler ? .rouvrir : .rien }
        // Plus là : coupée par iOS au bout de ses huit heures — on la rouvre —,
        // ou balayée à la main avant — on n'insiste pas.
        return age >= dureeMax - 15 * 60 ? .rouvrir : .oublier
    }

    /// Ouvre le suivi de la météo d'une parcelle — une seule fois.
    func start(parcelle: Parcelle, forecast: AgroForecast) {
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *), isAvailable else { return }
        // Déjà affichée : on la met à jour plutôt que d'en empiler une autre.
        guard !Self.hasActivities else {
            isRunning = true
            Self.demandee = true
            Task { await refresh(forecast: forecast) }
            return
        }
        ouvrir(parcelle: parcelle, forecast: forecast)
        #endif
    }

    #if canImport(ActivityKit)
    @available(iOS 16.2, *)
    private func ouvrir(parcelle: Parcelle, forecast: AgroForecast) {
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
        if let activity {
            Self.demandee = true
            Self.ouverteLe = now
            Self.suivre(activity)
        }
    }
    #endif

    /// Reflète la prévision fraîchement chargée sur toute activité affichée.
    ///
    /// Et rouvre l'activité quand il le faut : coupée par iOS au bout de huit
    /// heures, vieille de plus de six, ou ouverte pour une autre ville ou sous
    /// un autre nom — le nom et le lieu sont figés à l'ouverture, et y verser
    /// la prévision d'ailleurs montrait Paris sous le nom de Lyon.
    func refresh(forecast: AgroForecast) async {
        #if canImport(ActivityKit)
        if #available(iOS 16.2, *), isAvailable {
            let affichee = Self.vivantes.first
            // Ouverte par une version qui ne s'en souvenait pas : elle a été
            // demandée, on ne sait pas quand — elle sera rouverte, pour huit
            // heures pleines.
            if affichee != nil, !Self.demandee {
                Self.demandee = true
                Self.ouverteLe = nil
            }
            let ailleurs = affichee.map {
                $0.attributes.parcelleName != forecast.parcelle.name
                    || $0.attributes.latitude != forecast.parcelle.latitude
                    || $0.attributes.longitude != forecast.parcelle.longitude
            } ?? false
            switch Self.suite(demandee: Self.demandee, affichee: affichee != nil,
                              ouverteLe: Self.ouverteLe, maintenant: Date()) {
            case .rouvrir:
                await fermer()
                ouvrir(parcelle: forecast.parcelle, forecast: forecast)
                return
            case .oublier:
                Self.demandee = false
            case .rien where ailleurs:
                await fermer()
                ouvrir(parcelle: forecast.parcelle, forecast: forecast)
                return
            case .rien:
                break
            }
        }
        #endif
        await Self.update(forecast: forecast)
        isRunning = Self.hasActivities
    }

    /// Ferme toutes les activités météo, à la demande de la personne : elles
    /// ne reviendront pas d'elles-mêmes.
    func stop() async {
        Self.demandee = false
        await fermer()
    }

    /// Ferme toutes les activités météo, y compris celles d'un lancement précédent.
    private func fermer() async {
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

    /// L'état affichable : les conditions mesurées — l'heure prévue si le
    /// relevé date d'une heure passée, comme le relais —, et l'heure qui vient.
    static func state(from forecast: AgroForecast, at now: Date) -> WeatherActivityAttributes.ContentState {
        let present = Horizon.conditions(a: now, releve: forecast.current, dans: forecast.hourly)
        let suivante = Horizon.heureSuivante(apres: now, dans: forecast.hourly)
        // Le jour embrasse ce qui se montre comme « maintenant », puis l'heure
        // suivante si elle est du même jour : une fois l'île périmée, c'est
        // elle qu'on lit à côté du minimum et du maximum. Comme le relais.
        var today = Horizon.jour(contenant: now, embrassant: present.temperature,
                                 dans: forecast.daily)
        if let jour = today, let suivante, suivante.time >= jour.date,
           suivante.time < Horizon.finDuJour(jour, dans: forecast.daily) {
            today = jour.embrassant(suivante.temperature)
        }
        let next = suivante.map {
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
            temperature: present.temperature,
            apparentTemperature: present.apparentTemperature,
            weatherCode: present.weatherCode,
            isDay: present.isDay,
            windSpeed: present.windSpeed,
            temperatureMin: today?.temperatureMin ?? present.temperature,
            temperatureMax: today?.temperatureMax ?? present.temperature,
            updatedAt: now,
            next: next
        )
    }
}
