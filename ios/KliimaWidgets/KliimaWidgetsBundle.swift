import SwiftUI
import WidgetKit

/// Point d'entrée de l'extension : l'activité en direct de la météo, le widget
/// d'écran d'accueil, et la météo sur l'écran verrouillé.
@main
struct KliimaWidgetsBundle: WidgetBundle {
    var body: some Widget {
        WeatherLiveActivity()
        WeatherWidget()
        PluieWidget()
        HeuresWidget()
        SemaineWidget()
        DehorsWidget()
        WeatherLockScreenWidget()
    }
}
