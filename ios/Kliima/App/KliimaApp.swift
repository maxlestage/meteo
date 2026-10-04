import SwiftUI
import UserNotifications

@main
struct KliimaApp: App {
    init() {
        // Avant toute notification : sans délégué, celles qui arrivent
        // application ouverte ne s'affichent pas.
        UNUserNotificationCenter.current().delegate = AlertScheduler.presentation
    }

    var body: some Scene {
        WindowGroup {
            DashboardView()
                .onAppear { BackgroundRefresh.schedule() }
        }
        // Le système nous réveille de temps à autre : on en profite pour
        // rafraîchir l'activité en direct et les widgets.
        .backgroundTask(.appRefresh(BackgroundRefresh.identifier)) {
            await BackgroundRefresh.run()
        }
    }
}
