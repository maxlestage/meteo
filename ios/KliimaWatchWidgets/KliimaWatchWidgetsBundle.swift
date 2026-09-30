import SwiftUI
import WidgetKit

/// Complications de cadran : la prochaine fenêtre de traitement et le temps
/// qu'il fait, au poignet.
@main
struct KliimaWatchWidgetsBundle: WidgetBundle {
    var body: some Widget {
        SprayComplication()
        WeatherComplication()
    }
}
