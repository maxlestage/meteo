import Foundation

/// Mise en forme des nombres et des dates.
///
/// Deux règles : les nombres suivent la langue de l'appareil (virgule décimale
/// en français et en espagnol, point en anglais), et les heures suivent le
/// fuseau de la parcelle — une parcelle consultée depuis l'étranger garde ses
/// heures locales. L'équivalent web est `core/src/format.ts`.
enum AgroFormat {

    // MARK: Nombres

    /// « 4,8 » en français, « 4.8 » en anglais.
    ///
    /// La locale est un paramètre — avec la locale de l'appareil par défaut —
    /// pour que les tests puissent vérifier chaque langue.
    static func decimal(_ value: Double, decimals: Int = 1, locale: Locale = .current) -> String {
        let formatter = numberFormatter(locale)
        formatter.minimumFractionDigits = decimals
        formatter.maximumFractionDigits = decimals
        return formatter.string(from: value as NSNumber) ?? String(value)
    }

    /// « 4,8 mm », l'espace insécable évitant une coupure en fin de ligne.
    static func unit(
        _ value: Double,
        _ unit: String,
        decimals: Int = 1,
        locale: Locale = .current
    ) -> String {
        "\(decimal(value, decimals: decimals, locale: locale))\u{00a0}\(unit)"
    }

    /// « +2,7 mm » : le signe rend un bilan lisible d'un coup d'œil.
    static func signedUnit(
        _ value: Double,
        _ unit: String,
        decimals: Int = 1,
        locale: Locale = .current
    ) -> String {
        (value > 0 ? "+" : "") + self.unit(value, unit, decimals: decimals, locale: locale)
    }

    /// « 27 % », ponctué selon la langue.
    static func percent(_ value: Double, locale: Locale = .current) -> String {
        let formatter = NumberFormatter()
        formatter.locale = locale
        formatter.numberStyle = .percent
        formatter.maximumFractionDigits = 0
        return formatter.string(from: (value / 100) as NSNumber) ?? "\(Int(value.rounded()))%"
    }

    /// « 16° », arrondi comme sur un bulletin météo.
    static func temperature(_ value: Double) -> String {
        "\(Int(value.rounded()))°"
    }

    // MARK: Dates

    /// « 22 h » en français, « 22 » ailleurs : le format suit la langue.
    static func hour(_ date: Date, in zone: TimeZone, locale: Locale = .current) -> String {
        formatter(template: "j", zone, locale).string(from: date)
    }

    /// « mar. 22 h »
    static func weekdayHour(_ date: Date, in zone: TimeZone, locale: Locale = .current) -> String {
        formatter(template: "Ej", zone, locale).string(from: date).capitalizedFirst
    }

    /// « mer. »
    static func weekday(_ date: Date, in zone: TimeZone, locale: Locale = .current) -> String {
        formatter(template: "E", zone, locale).string(from: date).capitalizedFirst
    }

    /// « 06:52 », ou « — » si l'information manque.
    static func time(_ date: Date?, in zone: TimeZone, locale: Locale = .current) -> String {
        guard let date else { return "—" }
        return formatter(template: "jm", zone, locale).string(from: date)
    }

    // MARK: Fabriques

    private static func numberFormatter(_ locale: Locale) -> NumberFormatter {
        let formatter = NumberFormatter()
        formatter.locale = locale
        formatter.numberStyle = .decimal
        return formatter
    }

    /// Le gabarit (`j`, `Ej`…) laisse la locale choisir l'ordre et le suffixe :
    /// « 22 h » en français, « 10 PM » en anglais américain.
    private static func formatter(template: String, _ zone: TimeZone, _ locale: Locale) -> DateFormatter {
        let formatter = DateFormatter()
        formatter.locale = locale
        formatter.timeZone = zone
        formatter.setLocalizedDateFormatFromTemplate(template)
        return formatter
    }
}

private extension String {
    var capitalizedFirst: String {
        guard let first else { return self }
        return first.uppercased() + dropFirst()
    }
}
