import SwiftUI
import WidgetKit

/// Point d'entrée de l'extension : pour l'instant, la seule activité en direct
/// de Klima est le suivi d'une fenêtre de traitement.
@main
struct KlimaWidgetsBundle: WidgetBundle {
    var body: some Widget {
        SprayLiveActivity()
    }
}
