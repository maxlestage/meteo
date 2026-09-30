import SwiftUI
import WidgetKit

/// Point d'entrée de l'extension : deux activités en direct — la fenêtre de
/// traitement et la météo — et les deux widgets d'écran d'accueil qui leur
/// répondent.
@main
struct KliimaWidgetsBundle: WidgetBundle {
    var body: some Widget {
        SprayLiveActivity()
        SprayWidget()
        WeatherLiveActivity()
        WeatherWidget()
    }
}
