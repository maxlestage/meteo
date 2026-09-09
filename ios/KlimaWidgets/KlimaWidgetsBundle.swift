import SwiftUI
import WidgetKit

/// Point d'entrée de l'extension : l'activité en direct qui suit une fenêtre de
/// traitement, et le widget d'écran d'accueil qui annonce la prochaine.
@main
struct KlimaWidgetsBundle: WidgetBundle {
    var body: some Widget {
        SprayLiveActivity()
        SprayWidget()
    }
}
