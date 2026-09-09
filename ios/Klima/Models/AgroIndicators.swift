import Foundation

/// Seuils agronomiques partagés avec le site web (`web/src/domain/agro.ts`).
/// Les valeurs suivent les recommandations usuelles de pulvérisation
/// (vent < 19 km/h — arrêté du 4 mai 2017) et le modèle FAO-56.
enum AgroThresholds {
    /// Base de calcul des degrés-jours (maïs, tournesol : 6 à 10 °C).
    static let gddBase = 10.0
    /// Plafond au-delà duquel la plante ne capitalise plus (°C).
    static let gddCeiling = 30.0
    /// Vent minimum : en dessous, l'inversion thermique fait dériver le produit.
    static let sprayWindMin = 3.0
    /// Vent maximum réglementaire pour la pulvérisation (km/h).
    static let sprayWindMax = 19.0
    static let sprayGustMax = 25.0
    static let sprayTempMin = 5.0
    static let sprayTempMax = 25.0
    static let sprayHumidityMin = 40.0
    /// Au-delà de ce VPD, la gouttelette s'évapore avant d'atteindre la cible.
    static let sprayVpdMax = 1.2
    /// Pluie tolérée sur l'heure et l'heure suivante (mm).
    static let sprayRainMax = 0.1
    /// Humidité relative à partir de laquelle on considère le feuillage mouillé.
    static let leafWetnessHumidity = 90.0
    static let diseaseTempMin = 8.0
    static let diseaseTempMax = 30.0
    /// Humidité volumique du sol : ressuyage et portance.
    static let soilTooWet = 0.35
    static let soilIdealMin = 0.15
    static let soilTooDry = 0.12
    /// Déficit hydrique déclenchant une alerte d'irrigation (mm sur 7 jours).
    static let irrigationDeficit = -15.0
}

// MARK: - Bilan hydrique

enum WaterStatus: String {
    case deficit, equilibre, excedent

    /// Clé de catalogue, par exemple « water.deficit ».
    var labelKey: String { "water.\(rawValue)" }

    /// Libellé traduit dans la langue de l'appareil.
    var label: String { Localized.text(labelKey) }
}

struct WaterBalance: Equatable {
    /// Cumul de pluie sur la période (mm).
    let precipitation: Double
    /// Cumul d'évapotranspiration de référence sur la période (mm).
    let evapotranspiration: Double
    /// Pluie − ET0 (mm). Négatif = la parcelle puise dans sa réserve.
    let balance: Double
    let status: WaterStatus
    /// Conseil d'irrigation exprimé en mm à apporter (0 si inutile).
    let irrigationAdvice: Double
}

// MARK: - Pulvérisation

enum SprayVerdict: String {
    case favorable, acceptable, defavorable

    /// Clé de catalogue, par exemple « spray.favorable ».
    var labelKey: String { "spray.\(rawValue)" }

    /// Libellé traduit dans la langue de l'appareil.
    var label: String { Localized.text(labelKey) }
}

/// Motif de dégradation d'une heure, sous forme structurée : le domaine dit ce
/// qui cloche et avec quelles valeurs, l'interface le formule dans sa langue.
enum SprayBlocker: Equatable, Hashable {
    case windTooStrong(wind: Double, limit: Double)
    case windTooWeak
    case gusts(Double)
    case rain(Double)
    case tooHot(Double)
    case tooCold(Double)
    case dryAir(Double)
    case vapourPressureDeficit(Double)

    /// Motif formulé dans la langue de l'appareil, unités comprises.
    var text: String {
        switch self {
        case let .windTooStrong(wind, limit):
            return Localized.text(
                "spray.windTooStrong",
                AgroFormat.unit(wind, "km/h", decimals: 0),
                AgroFormat.unit(limit, "km/h", decimals: 0)
            )
        case .windTooWeak:
            return Localized.text("spray.windTooWeak")
        case let .gusts(gusts):
            return Localized.text("spray.gusts", AgroFormat.unit(gusts, "km/h", decimals: 0))
        case let .rain(amount):
            return Localized.text("spray.rain", AgroFormat.unit(amount, "mm"))
        case let .tooHot(temperature):
            return Localized.text("spray.tooHot", AgroFormat.unit(temperature, "°C", decimals: 0))
        case let .tooCold(temperature):
            return Localized.text("spray.tooCold", AgroFormat.unit(temperature, "°C", decimals: 0))
        case let .dryAir(humidity):
            return Localized.text("spray.dryAir", AgroFormat.percent(humidity))
        case let .vapourPressureDeficit(vpd):
            return Localized.text(
                "spray.vapourPressureDeficit",
                AgroFormat.unit(vpd, "kPa", decimals: 2)
            )
        }
    }
}

struct SprayWindow: Equatable, Identifiable {
    let time: Date
    let verdict: SprayVerdict
    /// Score 0–100 : 100 = conditions idéales.
    let score: Int
    /// Motifs de dégradation, à traduire à l'affichage.
    let blockers: [SprayBlocker]

    var id: Date { time }
}

/// Plage continue exploitable pour un traitement.
struct SprayOpportunity: Equatable {
    let start: Date
    let end: Date
    let score: Int
}

// MARK: - Gel

enum FrostSeverity: String {
    case aucun, faible, modere, severe

    /// Clé de catalogue, par exemple « frost.aucun ».
    var labelKey: String { "frost.\(rawValue)" }

    /// Libellé traduit dans la langue de l'appareil.
    var label: String { Localized.text(labelKey) }
}

struct FrostRisk: Equatable {
    let severity: FrostSeverity
    /// Température minimale attendue (°C).
    let minTemperature: Double
    /// Vrai si le point de rosée est négatif : gelée blanche probable.
    let hoarFrost: Bool
}

// MARK: - Maladies

enum DiseaseLevel: String {
    case faible, moyenne, elevee

    /// Clé de catalogue, par exemple « disease.faible ».
    var labelKey: String { "disease.\(rawValue)" }

    /// Libellé traduit dans la langue de l'appareil.
    var label: String { Localized.text(labelKey) }
}

struct DiseasePressure: Equatable {
    /// Heures d'humectation du feuillage (HR ≥ 90 % dans la plage de température).
    let leafWetnessHours: Int
    let level: DiseaseLevel
}

// MARK: - Sol

enum SoilState: String {
    case sature, ressuye, sec

    /// Clé de catalogue, par exemple « soil.sature ».
    var labelKey: String { "soil.\(rawValue)" }

    /// Libellé traduit dans la langue de l'appareil.
    var label: String { Localized.text(labelKey) }
}

struct SoilCondition: Equatable {
    /// Humidité volumique moyenne 3–9 cm (m³/m³).
    let moisture: Double
    /// Température moyenne du sol à 6 cm (°C).
    let temperature: Double
    let state: SoilState
    /// Vrai si le sol porte les engins sans risque de tassement.
    let trafficable: Bool
    /// Vrai si le sol est assez chaud pour lever (semis de printemps).
    let sowable: Bool
}

// MARK: - Synthèse

struct AgroSummary: Equatable {
    let water: WaterBalance
    let soil: SoilCondition
    let disease: DiseasePressure
    let frost: FrostRisk
    let gdd: Double
    let nextSpray: SprayOpportunity?
}

/// Cœur agronomique de l'application : des fonctions pures qui transforment les
/// mesures brutes en indicateurs directement exploitables au champ. La même
/// logique est implémentée à l'identique côté web (`web/src/domain/agro.ts`).
enum AgroIndicators {

    // MARK: Degrés-jours

    /// Degrés-jours d'une journée (méthode « moyenne plafonnée »).
    /// GDD = clamp((Tmin + Tmax) / 2, base, plafond) − base
    static func growingDegreeDays(
        temperatureMin: Double,
        temperatureMax: Double,
        base: Double = AgroThresholds.gddBase,
        ceiling: Double = AgroThresholds.gddCeiling
    ) -> Double {
        let mean = (temperatureMin + temperatureMax) / 2
        let capped = min(max(mean, base), ceiling)
        return round(capped - base, decimals: 1)
    }

    /// Cumul de degrés-jours sur une série de journées.
    static func cumulativeGdd(_ days: [DailySample], base: Double = AgroThresholds.gddBase) -> Double {
        let total = days.reduce(0.0) { sum, day in
            sum + growingDegreeDays(
                temperatureMin: day.temperatureMin,
                temperatureMax: day.temperatureMax,
                base: base
            )
        }
        return round(total, decimals: 1)
    }

    // MARK: Bilan hydrique

    /// Bilan hydrique climatique P − ET0 sur la période fournie.
    static func waterBalance(_ days: [DailySample]) -> WaterBalance {
        let precipitation = round(days.reduce(0) { $0 + $1.precipitationSum }, decimals: 1)
        let evapotranspiration = round(days.reduce(0) { $0 + $1.et0Sum }, decimals: 1)
        let balance = round(precipitation - evapotranspiration, decimals: 1)

        let status: WaterStatus
        if balance <= AgroThresholds.irrigationDeficit {
            status = .deficit
        } else if balance >= 15 {
            status = .excedent
        } else {
            status = .equilibre
        }

        return WaterBalance(
            precipitation: precipitation,
            evapotranspiration: evapotranspiration,
            balance: balance,
            status: status,
            irrigationAdvice: status == .deficit ? abs(balance) : 0
        )
    }

    // MARK: Pulvérisation

    /// Évalue l'heure `index` de la série pour un traitement phytosanitaire.
    ///
    /// Deux familles de critères :
    ///  - les critères rédhibitoires (vent au-delà de la limite réglementaire,
    ///    rafales, pluie imminente) rendent l'heure inexploitable quoi qu'il arrive ;
    ///  - les critères de confort (température, hygrométrie, VPD) dégradent le score.
    static func evaluateSprayHour(_ hours: [HourlySample], at index: Int) -> SprayWindow? {
        guard hours.indices.contains(index) else { return nil }
        let hour = hours[index]
        let next = hours.indices.contains(index + 1) ? hours[index + 1] : nil

        var blockers: [SprayBlocker] = []
        var score = 100
        /// Un critère rédhibitoire interdit le passage, quel que soit le reste.
        var disqualified = false

        if hour.windSpeed > AgroThresholds.sprayWindMax {
            blockers.append(.windTooStrong(wind: hour.windSpeed, limit: AgroThresholds.sprayWindMax))
            score -= 45
            disqualified = true
        } else if hour.windSpeed < AgroThresholds.sprayWindMin {
            blockers.append(.windTooWeak)
            score -= 25
        }

        if hour.windGusts > AgroThresholds.sprayGustMax {
            blockers.append(.gusts(hour.windGusts))
            score -= 20
            disqualified = true
        }

        let rainSoon = hour.precipitation + (next?.precipitation ?? 0)
        if rainSoon > AgroThresholds.sprayRainMax {
            blockers.append(.rain(round(rainSoon, decimals: 1)))
            score -= 45
            disqualified = true
        }

        if hour.temperature > AgroThresholds.sprayTempMax {
            blockers.append(.tooHot(hour.temperature))
            score -= 20
        } else if hour.temperature < AgroThresholds.sprayTempMin {
            blockers.append(.tooCold(hour.temperature))
            score -= 20
        }

        if hour.relativeHumidity < AgroThresholds.sprayHumidityMin {
            blockers.append(.dryAir(hour.relativeHumidity))
            score -= 15
        }

        if hour.vapourPressureDeficit > AgroThresholds.sprayVpdMax {
            blockers.append(.vapourPressureDeficit(round(hour.vapourPressureDeficit, decimals: 2)))
            score -= 15
        }

        score = min(max(score, 0), 100)
        if disqualified { score = min(score, 35) }

        let verdict: SprayVerdict
        if disqualified {
            verdict = .defavorable
        } else if score >= 80 {
            verdict = .favorable
        } else if score >= 55 {
            verdict = .acceptable
        } else {
            verdict = .defavorable
        }

        return SprayWindow(time: hour.time, verdict: verdict, score: score, blockers: blockers)
    }

    /// Évalue toute la série horaire.
    static func sprayWindows(_ hours: [HourlySample]) -> [SprayWindow] {
        hours.indices.compactMap { evaluateSprayHour(hours, at: $0) }
    }

    /// Prochaine plage d'au moins `minLength` heures consécutives exploitables.
    static func nextSprayOpportunity(_ windows: [SprayWindow], minLength: Int = 2) -> SprayOpportunity? {
        var run: [SprayWindow] = []
        for window in windows {
            guard window.verdict != .defavorable else {
                run.removeAll()
                continue
            }
            run.append(window)
            if run.count >= minLength, let first = run.first, let last = run.last {
                let mean = Double(run.reduce(0) { $0 + $1.score }) / Double(run.count)
                return SprayOpportunity(
                    start: first.time,
                    end: last.time.addingTimeInterval(3600),
                    score: Int(mean.rounded())
                )
            }
        }
        return nil
    }

    // MARK: Gel

    /// Risque de gel d'une nuit à partir de la température mini et du point de rosée.
    static func frostRisk(minTemperature: Double, minDewPoint: Double) -> FrostRisk {
        let severity: FrostSeverity
        if minTemperature <= -4 {
            severity = .severe
        } else if minTemperature <= -2 {
            severity = .modere
        } else if minTemperature <= 1 {
            severity = .faible
        } else {
            severity = .aucun
        }

        return FrostRisk(
            severity: severity,
            minTemperature: round(minTemperature, decimals: 1),
            hoarFrost: severity != .aucun && minDewPoint <= 0
        )
    }

    // MARK: Maladies

    /// Approximation de la pression cryptogamique (mildiou, septoriose) : on compte
    /// les heures d'humectation du feuillage dans la plage de température favorable
    /// au champignon, à la manière des tables de Mills.
    static func diseasePressure(_ hours: [HourlySample]) -> DiseasePressure {
        let leafWetnessHours = hours.filter { hour in
            hour.relativeHumidity >= AgroThresholds.leafWetnessHumidity
                && hour.temperature >= AgroThresholds.diseaseTempMin
                && hour.temperature <= AgroThresholds.diseaseTempMax
        }.count

        let level: DiseaseLevel
        if leafWetnessHours >= 12 {
            level = .elevee
        } else if leafWetnessHours >= 6 {
            level = .moyenne
        } else {
            level = .faible
        }

        return DiseasePressure(leafWetnessHours: leafWetnessHours, level: level)
    }

    // MARK: Sol

    /// État du sol moyenné sur la série horaire fournie.
    static func soilCondition(_ hours: [HourlySample]) -> SoilCondition {
        guard !hours.isEmpty else {
            return SoilCondition(moisture: 0, temperature: 0, state: .sec, trafficable: false, sowable: false)
        }

        let moisture = round(mean(hours.map(\.soilMoisture3to9cm)), decimals: 3)
        let temperature = round(mean(hours.map(\.soilTemperature6cm)), decimals: 1)

        let state: SoilState
        if moisture >= AgroThresholds.soilTooWet {
            state = .sature
        } else if moisture <= AgroThresholds.soilTooDry {
            state = .sec
        } else {
            state = .ressuye
        }

        return SoilCondition(
            moisture: moisture,
            temperature: temperature,
            state: state,
            trafficable: state != .sature,
            sowable: temperature >= 8 && moisture >= AgroThresholds.soilIdealMin && state != .sature
        )
    }

    // MARK: Synthèse

    /// Assemble tous les indicateurs pour le tableau de bord.
    static func summarize(hours: [HourlySample], days: [DailySample]) -> AgroSummary {
        let next24h = Array(hours.prefix(24))
        let tonight = Array(hours.prefix(18))

        let minTemperature = tonight.map(\.temperature).min() ?? days.first?.temperatureMin ?? 0
        let minDewPoint = tonight.map(\.dewPoint).min() ?? 0

        return AgroSummary(
            water: waterBalance(days),
            soil: soilCondition(next24h),
            disease: diseasePressure(next24h),
            frost: frostRisk(minTemperature: minTemperature, minDewPoint: minDewPoint),
            gdd: cumulativeGdd(days),
            nextSpray: nextSprayOpportunity(sprayWindows(hours))
        )
    }

    // MARK: Utilitaires

    private static func mean(_ values: [Double]) -> Double {
        guard !values.isEmpty else { return 0 }
        return values.reduce(0, +) / Double(values.count)
    }

    private static func round(_ value: Double, decimals: Int) -> Double {
        let factor = pow(10.0, Double(decimals))
        return (value * factor).rounded() / factor
    }
}
