import SwiftUI
import UserNotifications

@main
struct KliimaApp: App {
    /// L'apparence choisie : celle du téléphone par défaut.
    @AppStorage(Apparence.cle) private var apparence: Apparence = .automatique

    init() {
        // Avant toute notification : sans délégué, celles qui arrivent
        // application ouverte ne s'affichent pas.
        UNUserNotificationCenter.current().delegate = AlertScheduler.presentation
    }

    var body: some Scene {
        WindowGroup {
            DashboardView()
                .onAppear {
                    BackgroundRefresh.schedule()
                    // Les fenêtres existent maintenant : on y pose le choix.
                    apparence.appliquer()
                }
        }
        // Le système nous réveille de temps à autre : on en profite pour
        // rafraîchir l'activité en direct et les widgets.
        .backgroundTask(.appRefresh(BackgroundRefresh.identifier)) {
            await BackgroundRefresh.run()
        }
    }
}
