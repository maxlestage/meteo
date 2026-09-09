import XCTest
@testable import Klima

/// Les mêmes cas que `core/src/register.test.ts`.
final class RegisterTests: XCTestCase {
    private let zone = TimeZone(identifier: "Europe/Paris")!

    private lazy var calendar: Calendar = {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = zone
        return calendar
    }()

    private func date(_ iso: String) -> Date {
        let formatter = DateFormatter()
        formatter.calendar = calendar
        formatter.timeZone = zone
        formatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss"
        return formatter.date(from: iso)!
    }

    private var start: Date { date("2026-04-15T08:00:00") }

    private func hour(_ offset: Double) -> HourlySample {
        HourlySample(
            time: start.addingTimeInterval(offset * 3600), weatherCode: 3, isDay: true,
            precipitationProbability: 10, temperature: 17.4, relativeHumidity: 62,
            dewPoint: 9, precipitation: 0, windSpeed: 11.2, windGusts: 18.5,
            soilTemperature6cm: 12, soilMoisture3to9cm: 0.22, et0: 0.1,
            vapourPressureDeficit: 0.6
        )
    }

    private var series: [HourlySample] { [hour(0), hour(1), hour(2), hour(3)] }

    private var options: Register.CsvOptions {
        var options = Register.CsvOptions()
        options.timeZone = zone
        return options
    }

    private var sample: TreatmentRecord {
        Register.record(
            hours: series, at: date("2026-04-15T09:00:00"),
            parcelle: "Le Clos", product: "Cuivre"
        )!
    }

    // MARK: Relevé

    func testPrendLesConditionsDeLHeureContenante() {
        let record = Register.record(
            hours: series, at: date("2026-04-15T09:37:00"), parcelle: "Le Clos"
        )!
        XCTAssertEqual(record.at, date("2026-04-15T09:00:00"))
        XCTAssertEqual(record.parcelle, "Le Clos")
        XCTAssertEqual(record.windSpeed, 11.2)
        XCTAssertEqual(record.relativeHumidity, 62)
    }

    func testPorteLeVerdictATitreIndicatif() {
        let record = Register.record(
            hours: series, at: date("2026-04-15T09:00:00"), parcelle: "Le Clos"
        )!
        XCTAssertTrue(SprayVerdict.allCases.contains(record.verdict))
        XCTAssertGreaterThanOrEqual(record.score, 0)
    }

    func testProduitVientDeLExploitant() {
        XCTAssertNil(Register.record(
            hours: series, at: date("2026-04-15T09:00:00"), parcelle: "Le Clos"
        )!.product)
        XCTAssertEqual(sample.product, "Cuivre")
    }

    func testHorsSerieAucuneLigne() {
        // Un document qu'on pourra vous opposer ne se remplit pas au jugé.
        XCTAssertNil(Register.record(
            hours: series, at: date("2026-04-20T09:00:00"), parcelle: "Le Clos"
        ))
    }

    // MARK: CSV

    func testSeparePointVirguleEtDecimeVirgule() {
        let line = Register.csv([sample], options: options).components(separatedBy: "\r\n")[1]
        XCTAssertTrue(line.contains(";"))
        XCTAssertTrue(line.contains("11,2"))
        XCTAssertFalse(line.contains("11.2"))
    }

    func testCommenceParLaMarqueDOrdreDesOctets() {
        // Sans elle, Excel lit l'UTF-8 comme du Latin-1.
        XCTAssertTrue(Register.csv([sample], options: options).hasPrefix("\u{FEFF}"))
    }

    func testParcelleAvecSeparateurNeCassePasLaColonne() {
        let piege = TreatmentRecord(
            at: sample.at, parcelle: "Le Clos ; bas", product: sample.product,
            temperature: sample.temperature, relativeHumidity: sample.relativeHumidity,
            windSpeed: sample.windSpeed, windGusts: sample.windGusts,
            precipitation: sample.precipitation, verdict: sample.verdict, score: sample.score
        )
        let line = Register.csv([piege], options: options).components(separatedBy: "\r\n")[1]

        XCTAssertTrue(line.contains("\"Le Clos ; bas\""))
        XCTAssertGreaterThan(line.components(separatedBy: ";").count, Register.columns.count)
        XCTAssertEqual(line.filter { $0 == "\"" }.count, 2)
    }

    func testGuillemetSeDouble() {
        let piege = TreatmentRecord(
            at: sample.at, parcelle: "Le \"Clos\"", product: sample.product,
            temperature: sample.temperature, relativeHumidity: sample.relativeHumidity,
            windSpeed: sample.windSpeed, windGusts: sample.windGusts,
            precipitation: sample.precipitation, verdict: sample.verdict, score: sample.score
        )
        XCTAssertTrue(Register.csv([piege], options: options).contains("\"Le \"\"Clos\"\"\""))
    }

    func testEnTeteSuitLOrdreEtSeTraduit() {
        var traduit = options
        traduit.headers = ["Date", "Heure", "Parcelle", "Produit", "Température",
                           "Humidité", "Vent", "Rafales", "Pluie", "Avis", "Score"]
        let header = Register.csv([sample], options: traduit).components(separatedBy: "\r\n")[0]
        XCTAssertEqual(
            header,
            "\u{FEFF}Date;Heure;Parcelle;Produit;Température;Humidité;Vent;Rafales;Pluie;Avis;Score"
        )
    }

    func testChaqueLigneAAutantDeChampsQueDeColonnes() {
        let line = Register.csv([sample], options: options).components(separatedBy: "\r\n")[1]
        XCTAssertEqual(line.components(separatedBy: ";").count, Register.columns.count)
    }

    func testExportVideGardeSonEnTete() {
        let lines = Register.csv([], options: options)
            .components(separatedBy: "\r\n")
            .filter { !$0.isEmpty }
        XCTAssertEqual(lines.count, 1)
    }

    func testPointEtVirgulePourUnTableurAnglais() {
        var anglais = options
        anglais.delimiter = ","
        anglais.decimal = "."
        let line = Register.csv([sample], options: anglais).components(separatedBy: "\r\n")[1]
        XCTAssertTrue(line.contains("11.2"))
        XCTAssertEqual(line.components(separatedBy: ",").count, Register.columns.count)
    }
}
