import SwiftUI
import WidgetKit

/// Point d'entrée de l'extension : l'activité en direct de la météo, le widget
/// d'écran d'accueil, et la météo sur l'écran verrouillé.
@main
struct KliimaWidgetsBundle: WidgetBundle {
    var body: some Widget {
        // Sur iOS 18, l'activité sait aussi se montrer sur la montre.
        if #available(iOS 18.0, *) {
            WeatherLiveActivityMontre()
        } else {
            WeatherLiveActivity()
        }
        WeatherWidget()
        WeatherLockScreenWidget()
    }
}
