import Foundation

/// Les phrases du guetteur, partagées par la carte de l'iPhone et l'écran de
/// la montre.
///
/// Le modèle (`Veille`) rend des états ; ici, on les dit, dans la langue de
/// l'utilisateur et aux heures de la ville.
enum VeilleTextes {

    /// La demi-heure en cours, en une phrase.
    static func immediat(_ lecture: VeilleLecture, in zone: TimeZone) -> String {
        let heure = { AgroFormat.time($0, in: zone) }
        switch lecture.immediat {
        case .sec:
            return Localized.text("veille.now.dry")
        case let .commence(debut, precipitation):
            return Localized.text("veille.now.starts", heure(debut), precipitation.label)
        case let .dure(precipitation):
            return Localized.text("veille.now.continues", precipitation.label)
        case let .cesse(fin):
            return Localized.text("veille.now.stops", heure(fin))
        }
    }

    /// Les deux heures à venir : d'abord la pluie, puis les rafales et la
    /// température au bout de la fenêtre. La montre ne garde que la première.
    static func suite(_ lecture: VeilleLecture, in zone: TimeZone) -> [String] {
        let heure = { AgroFormat.time($0, in: zone) }
        let fin = heure(lecture.finFenetre)
        var phrases: [String] = []

        switch lecture.suite {
        case .sec:
            phrases.append(Localized.text("veille.next.dry", fin))
        case let .episode(debut, arret?, precipitation, cumul):
            phrases.append(Localized.text(
                "veille.next.episode", heure(debut), heure(arret), precipitation.label, AgroFormat.unit(cumul, "mm")
            ))
        case let .episode(debut, .none, precipitation, _):
            phrases.append(Localized.text("veille.next.episodeOpen", heure(debut), fin, precipitation.label))
        case let .persiste(precipitation, cumul):
            phrases.append(Localized.text("veille.next.persists", fin, precipitation.label, AgroFormat.unit(cumul, "mm")))
        case let .accalmie(arret, .none):
            phrases.append(Localized.text("veille.next.lull", heure(arret), fin))
        case let .accalmie(arret, reprise?):
            phrases.append(Localized.text("veille.next.lullReturn", heure(arret), heure(reprise)))
        }

        if let rafales = lecture.rafales {
            phrases.append(Localized.text(
                "veille.gusts", AgroFormat.unit(rafales.valeur, "km/h", decimals: 0), heure(rafales.quand)
            ))
        }
        if let dernier = lecture.quarts.last {
            phrases.append(Localized.text(
                "veille.temperature",
                AgroFormat.temperature(dernier.temperature),
                heure(dernier.time),
                AgroFormat.temperature(dernier.apparentTemperature)
            ))
        }
        return phrases
    }

    /// Ce qu'on voit tomber à l'aéroport le plus proche, en une phrase ; `nil`
    /// quand rien n'y tombe — un ciel sec observé n'ajoute rien à la prévision.
    static func vu(_ ciel: CielObserve?, in zone: TimeZone) -> String? {
        guard let ciel, let precipitation = ciel.precipitation else { return nil }
        return Localized.text(
            "ciel.observed",
            ciel.nom,
            String(Int(ciel.distanceKm)),
            AgroFormat.time(ciel.time, in: zone),
            precipitation.label
        )
    }
}
