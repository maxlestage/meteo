import XCTest
@testable import Klima

/// Vérifie le catalogue de chaînes lui-même : une traduction oubliée doit faire
/// échouer la suite, pas apparaître en clair dans l'application.
final class LocalizationTests: XCTestCase {

    private let languages = ["fr", "en", "es"]

    /// Le catalogue est lu sur le disque, à côté des sources : le test vaut
    /// aussi bien dans Xcode que dans une compilation en ligne de commande.
    private func catalog(_ name: String) throws -> [String: Any] {
        let url = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("Klima/Resources/\(name).xcstrings")
        let data = try Data(contentsOf: url)
        let json = try XCTUnwrap(try JSONSerialization.jsonObject(with: data) as? [String: Any])
        return try XCTUnwrap(json["strings"] as? [String: Any])
    }

    private func values(_ entry: Any) throws -> [String: String] {
        let entry = try XCTUnwrap(entry as? [String: Any])
        let localizations = try XCTUnwrap(entry["localizations"] as? [String: Any])
        return localizations.compactMapValues { value in
            (value as? [String: Any])
                .flatMap { $0["stringUnit"] as? [String: Any] }
                .flatMap { $0["value"] as? String }
        }
    }

    func testEveryKeyIsTranslatedInEveryLanguage() throws {
        for name in ["Localizable", "InfoPlist"] {
            let strings = try catalog(name)
            XCTAssertFalse(strings.isEmpty, "\(name) est vide")

            for (key, entry) in strings {
                let translations = try values(entry)
                for language in languages {
                    let value = translations[language]
                    XCTAssertNotNil(value, "\(name) : « \(key) » n'est pas traduit en \(language)")
                    XCTAssertFalse(
                        value?.isEmpty ?? true,
                        "\(name) : « \(key) » est vide en \(language)"
                    )
                }
            }
        }
    }

    /// Un motif à trous doit avoir les mêmes trous dans les trois langues,
    /// faute de quoi le formatage produit du texte tronqué à l'exécution.
    func testPlaceholdersMatchAcrossLanguages() throws {
        for (key, entry) in try catalog("Localizable") {
            let translations = try values(entry)
            let counts = Set(translations.values.map(placeholderCount))
            XCTAssertEqual(counts.count, 1, "« \(key) » n'a pas le même nombre de valeurs partout")
        }
    }

    /// Chaque code météo documenté a bien son libellé.
    func testEveryWeatherCodeHasALabel() throws {
        let strings = try catalog("Localizable")
        let codes = [0, 1, 2, 3, 45, 48, 51, 53, 55, 56, 57, 61, 63, 65, 66, 67, 71, 73, 75, 77,
                     80, 81, 82, 85, 86, 95, 96, 99]
        for code in codes {
            let key = WeatherCondition.forCode(code).labelKey
            XCTAssertNotNil(strings[key], "code \(code) : « \(key) » absent du catalogue")
        }
    }

    /// Les états du domaine s'affichent tous.
    func testDomainStatesHaveLabels() throws {
        let strings = try catalog("Localizable")
        let keys = [SoilState.sature, .ressuye, .sec].map(\.labelKey)
            + [WaterStatus.deficit, .equilibre, .excedent].map(\.labelKey)
            + [FrostSeverity.aucun, .faible, .modere, .severe].map(\.labelKey)
            + [DiseaseLevel.faible, .moyenne, .elevee].map(\.labelKey)
            + [SprayVerdict.favorable, .acceptable, .defavorable].map(\.labelKey)
        for key in keys {
            XCTAssertNotNil(strings[key], "« \(key) » absent du catalogue")
        }
    }

    /// Les alertes et les motifs de blocage passent par le même catalogue :
    /// une clé absente ferait apparaître son identifiant à l'écran.
    func testAlertAndPlanKeysHaveLabels() throws {
        let strings = try catalog("Localizable")
        let keys = AlertKind.allCases.flatMap { ["alert.\($0.rawValue).title", "alert.\($0.rawValue).body"] }
            + Feature.allCases.map(\.upgradeReasonKey)
            + Plan.allCases.map { "plan.\($0.rawValue)" }
        for key in keys {
            XCTAssertNotNil(strings[key], "« \(key) » absent du catalogue")
        }
    }

    /// Un corps d'alerte porte un paramètre : il doit être positionnel, sinon
    /// il s'affiche tel quel au lieu d'être remplacé.
    func testAlertBodiesUsePositionalPlaceholders() throws {
        let strings = try catalog("Localizable")
        for kind in AlertKind.allCases {
            let translations = try values(try XCTUnwrap(strings["alert.\(kind.rawValue).body"]))
            for (language, value) in translations {
                XCTAssertTrue(
                    value.contains("%1$@"),
                    "alert.\(kind.rawValue).body en \(language) n'a pas de trou positionnel"
                )
                XCTAssertFalse(value.contains("{"), "trou à la mode TypeScript en \(language)")
            }
        }
    }

    private func placeholderCount(_ value: String) -> Int {
        var count = 0
        var index = value.startIndex
        while let found = value[index...].range(of: "%") {
            let next = value.index(after: found.lowerBound)
            if next < value.endIndex, value[next] != "%" { count += 1 }
            index = next < value.endIndex ? value.index(after: next) : value.endIndex
        }
        return count
    }
}
