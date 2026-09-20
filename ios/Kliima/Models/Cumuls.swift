import Foundation

/// Les cumuls depuis une date choisie.
///
/// Miroir de `core/src/cumuls.ts`.
///
/// Un agriculteur ne raisonne pas en « sept derniers jours » mais depuis un
/// événement : le semis, le dernier traitement, la reprise de végétation.
///
/// Le résultat dit toujours **ce qu'il a réellement couvert**. Une série de
/// prévision ne remonte pas dans le passé : demander le cumul depuis un semis
/// d'octobre à une série qui commence hier donnerait un chiffre faux, et un
/// chiffre faux dans un outil de décision est pire qu'une absence de chiffre.
struct Cumul: Equatable {
    /// Date demandée.
    let requestedFrom: Date
    /// Première journée réellement disponible dans la série.
    let from: Date
    /// Dernière journée prise en compte.
    let to: Date
    /// Journées effectivement additionnées.
    let days: Int
    /// Journées demandées qui manquent à la série. Au-delà de zéro, le cumul
    /// est un minorant.
    let missingDays: Int
    /// Cumul de pluie (mm).
    let precipitation: Double
    /// Cumul d'évapotranspiration de référence (mm).
    let evapotranspiration: Double
    /// Pluie − ET0 (mm). Négatif : la parcelle a puisé dans sa réserve.
    let balance: Double
    /// Degrés-jours capitalisés depuis la date.
    let gdd: Double

    /// Vrai si le cumul couvre toute la période demandée.
    var isComplete: Bool { missingDays == 0 }
}

enum Cumuls {
    /// Additionne les journées depuis `from` incluse.
    ///
    /// Renvoie `nil` quand aucune journée de la série n'entre dans la période :
    /// il n'y a alors rien d'honnête à afficher.
    static func accumulate(
        _ days: [DailySample],
        from: Date,
        base: Double = AgroThresholds.gddBase,
        calendar: Calendar = .current
    ) -> Cumul? {
        let start = calendar.startOfDay(for: from)
        let kept = days.filter { calendar.startOfDay(for: $0.date) >= start }
        guard let firstDay = kept.first, let lastDay = kept.last else { return nil }

        let first = calendar.startOfDay(for: firstDay.date)
        let last = calendar.startOfDay(for: lastDay.date)

        let precipitation = kept.reduce(0) { $0 + $1.precipitationSum }
        let evapotranspiration = kept.reduce(0) { $0 + $1.et0Sum }
        let gdd = kept.reduce(0.0) {
            $0 + AgroIndicators.growingDegreeDays(
                temperatureMin: $1.temperatureMin,
                temperatureMax: $1.temperatureMax,
                base: base
            )
        }

        // Ce que la série ne couvre pas : les journées entre la demande et son début.
        let missing = calendar.dateComponents([.day], from: start, to: first).day ?? 0

        return Cumul(
            requestedFrom: start,
            from: first,
            to: last,
            days: kept.count,
            missingDays: max(0, missing),
            precipitation: round(precipitation),
            evapotranspiration: round(evapotranspiration),
            balance: round(precipitation - evapotranspiration),
            gdd: round(gdd)
        )
    }

    private static func round(_ value: Double, digits: Int = 1) -> Double {
        let factor = pow(10.0, Double(digits))
        return (value * factor).rounded() / factor
    }
}
