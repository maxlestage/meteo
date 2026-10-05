import SwiftUI
import WidgetKit

/// Complication de cadran : le temps qu'il fait en ville, au poignet.
@main
struct KliimaWatchWidgetsBundle: WidgetBundle {
    var body: some Widget {
        WeatherComplication()
    }
}
