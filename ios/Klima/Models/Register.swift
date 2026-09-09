import Foundation

/// Le registre : les conditions à l'heure d'un traitement.
///
/// Miroir de `core/src/register.ts`.
///
/// Tenir un registre des traitements phytosanitaires est une obligation. Klima
/// **n'est pas ce registre** et ne prétend pas l'être : il fournit la partie
/// pénible à reconstituer après coup — les conditions météo relevées à l'heure
/// de l'application. Le reste (produit, dose, culture, opérateur) appartient à
/// l'exploitant, et ce module se contente de le recopier s'il le fournit.
struct TreatmentRecord: Equatable {
    /// Heure de l'application, dans le fuseau de la parcelle.
    let at: Date
    let parcelle: String
    /// Renseigné par l'exploitant : Klima ne l'invente pas.
    let product: String?
    let temperature: Double
    let relativeHumidity: Double
    let windSpeed: Double
    let windGusts: Double
    let precipitation: Double
    /// Ce que Klima aurait conseillé — indicatif, sans valeur réglementaire.
    let verdict: SprayVerdict
    let score: Int
}

enum Register {
    /// Les colonnes du document, dans l'ordre. Les en-têtes sont des clés.
    static let columns = [
        "date", "heure", "parcelle", "produit", "temperature",
        "humidite", "vent", "rafales", "pluie", "verdict", "score",
    ]

    /// Relève les conditions de l'heure qui contient `at`.
    ///
    /// Renvoie `nil` si la série ne couvre pas ce moment : mieux vaut une ligne
    /// absente qu'une ligne inventée dans un document qu'on pourra vous opposer.
    static func record(
        hours: [HourlySample],
        at: Date,
        parcelle: String,
        product: String? = nil
    ) -> TreatmentRecord? {
        let slot = floor(at.timeIntervalSince1970 / 3600)
        guard let index = hours.firstIndex(where: {
            floor($0.time.timeIntervalSince1970 / 3600) == slot
        }) else { return nil }

        let hour = hours[index]
        guard let window = AgroIndicators.evaluateSprayHour(hours, at: index) else { return nil }

        return TreatmentRecord(
            at: hour.time,
            parcelle: parcelle,
            product: product,
            temperature: hour.temperature,
            relativeHumidity: hour.relativeHumidity,
            windSpeed: hour.windSpeed,
            windGusts: hour.windGusts,
            precipitation: hour.precipitation,
            verdict: window.verdict,
            score: window.score
        )
    }

    struct CsvOptions {
        /// Séparateur de colonnes. Le point-virgule par défaut : Excel en
        /// langue française lit un fichier à virgules comme une seule colonne.
        var delimiter: String = ";"
        /// Séparateur décimal. La virgule par défaut, pour la même raison.
        var decimal: String = ","
        /// En-têtes déjà traduits, dans l'ordre de `columns`.
        var headers: [String]?
        /// Fuseau de la parcelle : les heures sont les siennes, pas celles du lecteur.
        var timeZone: TimeZone = .current
    }

    /// Met les relevés en CSV.
    ///
    /// Le fichier commence par une marque d'ordre des octets : sans elle, Excel
    /// lit l'UTF-8 comme du Latin-1 et « évapotranspiration » perd ses accents.
    static func csv(_ records: [TreatmentRecord], options: CsvOptions = CsvOptions()) -> String {
        let headers = options.headers ?? columns

        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = options.timeZone

        func number(_ value: Double, _ digits: Int = 1) -> String {
            String(format: "%.\(digits)f", value).replacingOccurrences(of: ".", with: options.decimal)
        }

        func two(_ value: Int) -> String { value < 10 ? "0\(value)" : "\(value)" }

        var lines = [headers.map { escape($0, options.delimiter) }.joined(separator: options.delimiter)]

        for record in records {
            let parts = calendar.dateComponents(
                [.year, .month, .day, .hour, .minute], from: record.at
            )
            lines.append([
                "\(parts.year ?? 0)-\(two(parts.month ?? 0))-\(two(parts.day ?? 0))",
                "\(two(parts.hour ?? 0)):\(two(parts.minute ?? 0))",
                escape(record.parcelle, options.delimiter),
                escape(record.product ?? "", options.delimiter),
                number(record.temperature),
                number(record.relativeHumidity, 0),
                number(record.windSpeed),
                number(record.windGusts),
                number(record.precipitation),
                record.verdict.rawValue,
                String(record.score),
            ].joined(separator: options.delimiter))
        }

        return "\u{FEFF}" + lines.joined(separator: "\r\n") + "\r\n"
    }

    /// Un guillemet se double, et tout champ qui contient un séparateur, un
    /// guillemet ou un saut de ligne se met entre guillemets. Sans ça, une
    /// parcelle nommée « Le Clos ; bas » casserait la colonne suivante.
    private static func escape(_ value: String, _ delimiter: String) -> String {
        let needsQuotes = value.contains(delimiter)
            || value.contains("\"")
            || value.contains("\n")
            || value.contains("\r")
        let escaped = value.replacingOccurrences(of: "\"", with: "\"\"")
        return needsQuotes ? "\"\(escaped)\"" : escaped
    }
}
