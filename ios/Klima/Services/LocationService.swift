import CoreLocation
import Foundation

enum LocationError: LocalizedError {
    case denied
    case unavailable

    var errorDescription: String? {
        switch self {
        case .denied:
            return "Localisation refusée. Recherchez la commune à la main."
        case .unavailable:
            return "Position indisponible pour le moment."
        }
    }
}

/// Relevé ponctuel de la position, pour caler la prévision sur la parcelle
/// où se trouve l'utilisateur. On ne suit pas la position en continu.
final class LocationService: NSObject, CLLocationManagerDelegate {

    private let manager = CLLocationManager()
    private var continuation: CheckedContinuation<CLLocationCoordinate2D, Error>?

    override init() {
        super.init()
        manager.delegate = self
        manager.desiredAccuracy = kCLLocationAccuracyHundredMeters
    }

    /// Demande l'autorisation si besoin, puis renvoie une position unique.
    func currentCoordinate() async throws -> CLLocationCoordinate2D {
        if case .denied = manager.authorizationStatus { throw LocationError.denied }
        if case .restricted = manager.authorizationStatus { throw LocationError.denied }

        return try await withCheckedThrowingContinuation { continuation in
            // Une seule demande à la fois : la précédente est abandonnée.
            self.continuation?.resume(throwing: LocationError.unavailable)
            self.continuation = continuation

            if manager.authorizationStatus == .notDetermined {
                manager.requestWhenInUseAuthorization()
            } else {
                manager.requestLocation()
            }
        }
    }

    // MARK: CLLocationManagerDelegate

    func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) {
        switch manager.authorizationStatus {
        case .authorizedWhenInUse, .authorizedAlways:
            guard continuation != nil else { return }
            manager.requestLocation()
        case .denied, .restricted:
            finish(with: .failure(LocationError.denied))
        case .notDetermined:
            break
        @unknown default:
            break
        }
    }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        guard let coordinate = locations.last?.coordinate else {
            finish(with: .failure(LocationError.unavailable))
            return
        }
        finish(with: .success(coordinate))
    }

    func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
        finish(with: .failure(LocationError.unavailable))
    }

    private func finish(with result: Result<CLLocationCoordinate2D, Error>) {
        guard let continuation else { return }
        self.continuation = nil
        continuation.resume(with: result)
    }
}
