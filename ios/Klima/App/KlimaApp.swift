import SwiftUI

@main
struct KlimaApp: App {
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
