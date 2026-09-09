import Foundation

/// Formatage des dates dans le fuseau de la parcelle, pas celui du téléphone :
/// une parcelle consultée depuis l'étranger doit garder ses heures locales.
enum AgroFormat {

    /// « 22 h »
    static func hour(_ date: Date, in zone: TimeZone) -> String {
        formatter("HH 'h'", zone).string(from: date)
    }

    /// « mar. 22 h »
    static func weekdayHour(_ date: Date, in zone: TimeZone) -> String {
        formatter("EEE HH 'h'", zone).string(from: date).capitalizedFirst
    }

    /// « mer. »
    static func weekday(_ date: Date, in zone: TimeZone) -> String {
        formatter("EEE", zone).string(from: date).capitalizedFirst
    }

    /// « 06:52 », ou « — » si l'information manque.
    static func time(_ date: Date?, in zone: TimeZone) -> String {
        guard let date else { return "—" }
        return formatter("HH:mm", zone).string(from: date)
    }

    private static func formatter(_ format: String, _ zone: TimeZone) -> DateFormatter {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "fr_FR")
        formatter.timeZone = zone
        formatter.dateFormat = format
        return formatter
    }
}

private extension String {
    var capitalizedFirst: String {
        guard let first else { return self }
        return first.uppercased() + dropFirst()
    }
}
