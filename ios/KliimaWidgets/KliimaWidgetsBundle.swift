import SwiftUI
import WidgetKit

/// Point d'entrée de l'extension : deux activités en direct — la fenêtre de
/// traitement et la météo —, les widgets d'écran d'accueil qui leur répondent,
/// et la météo sur l'écran verrouillé.
@main
struct KliimaWidgetsBundle: WidgetBundle {
    var body: some Widget {
        SprayLiveActivity()
        SprayWidget()
        WeatherLiveActivity()
        WeatherWidget()
        WeatherLockScreenWidget()
    }
}
