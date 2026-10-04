import Foundation

/// L'heure qui court, celle qui vient, et la suite — prises dans une série horaire.
///
/// L'île dynamique, l'écran verrouillé et les widgets ne recalculent rien
/// d'eux-mêmes : ils montrent ce qu'on leur a donné, jusqu'à ce qu'on leur
/// donne autre chose. Pour qu'ils restent justes quand l'heure tourne, on
/// calcule d'avance : l'heure en cours *et* la suivante pour l'île, une entrée
/// par heure à venir pour les widgets.
///
/// Les bornes viennent de la série elle-même, jamais d'un arrondi à l'heure
/// pleine : une parcelle à UTC+5:30 a ses heures à la demie, et un arrondi
/// ferait basculer l'affichage trente minutes trop tôt.
enum Horizon {

    /// L'heure de la série qui contient `instant`.
    ///
    /// Avant le début de la série, la première heure ; après sa fin, rien —
    /// mieux vaut ne rien montrer qu'une heure passée présentée comme actuelle.
    static func heure(contenant instant: Date, dans heures: [HourlySample]) -> HourlySample? {
        if let courante = heures.last(where: { $0.time <= instant }) {
            return instant < courante.time.addingTimeInterval(3600) ? courante : nil
        }
        return heures.first
    }

    /// L'heure qui suit celle qui contient `instant`.
    static func heureSuivante(apres instant: Date, dans heures: [HourlySample]) -> HourlySample? {
        guard let courante = heure(contenant: instant, dans: heures) else { return nil }
        return heures.first { $0.time > courante.time }
    }

    /// Le prochain début d'heure de la série, strictement après `instant` :
    /// le moment où ce qui est affiché cesse d'être vrai.
    static func prochaineBascule(apres instant: Date, dans heures: [HourlySample]) -> Date? {
        heures.first { $0.time > instant }?.time
    }

    /// Les heures à partir de celle qui contient `instant`.
    static func aPartirDe(_ instant: Date, dans heures: [HourlySample]) -> [HourlySample] {
        guard let courante = heure(contenant: instant, dans: heures) else { return [] }
        return heures.filter { $0.time >= courante.time }
    }

    /// Le jour de la série qui contient `instant`.
    static func jour(contenant instant: Date, dans jours: [DailySample]) -> DailySample? {
        jours.last { $0.date <= instant } ?? jours.first
    }

    /// Un moment d'une chronologie de widget : ce qu'il faut montrer à partir de `date`.
    struct Moment: Equatable {
        let date: Date
        /// Les conditions à afficher comme « maintenant ».
        let courant: CurrentSample
        /// Les heures à venir à partir de ce moment, la sienne comprise.
        let heures: [HourlySample]
        let jour: DailySample?
    }

    /// La chronologie des `nombre` heures à venir, calculée d'avance.
    ///
    /// Le premier moment est l'instant présent, avec les conditions mesurées —
    /// plus fraîches que l'heure prévue. Les suivants commencent à chaque
    /// début d'heure de la série, avec les conditions prévues pour elle : le
    /// widget bascule tout seul à l'heure pile, sans rien recharger.
    static func chronologie(
        courant: CurrentSample,
        heures: [HourlySample],
        jours: [DailySample],
        depuis instant: Date,
        nombre: Int = 12
    ) -> [Moment] {
        var moments = [Moment(
            date: instant,
            courant: courant,
            heures: aPartirDe(instant, dans: heures),
            jour: jour(contenant: instant, dans: jours)
        )]
        for heure in heures where heure.time > instant {
            guard moments.count <= nombre else { break }
            moments.append(Moment(
                date: heure.time,
                courant: CurrentSample(prevu: heure),
                heures: heures.filter { $0.time >= heure.time },
                jour: jour(contenant: heure.time, dans: jours)
            ))
        }
        return moments
    }
}

extension CurrentSample {
    /// Les conditions prévues pour une heure, présentées comme « maintenant ».
    ///
    /// La prévision horaire ne porte pas de température ressentie : on y met la
    /// température de l'air. Les widgets ne l'affichent pas ; qui voudrait
    /// l'afficher devra aller la chercher ailleurs plutôt que de faire passer
    /// l'une pour l'autre.
    init(prevu heure: HourlySample) {
        self.init(
            time: heure.time,
            temperature: heure.temperature,
            apparentTemperature: heure.temperature,
            weatherCode: heure.weatherCode,
            isDay: heure.isDay,
            relativeHumidity: heure.relativeHumidity,
            windSpeed: heure.windSpeed,
            windGusts: heure.windGusts
        )
    }
}
