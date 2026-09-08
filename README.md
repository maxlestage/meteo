# Klima

Météo agricole : une application iOS en SwiftUI, une application web et un site
de présentation, tous bâtis sur le même cœur agronomique. Les données viennent
exclusivement de l'**API agricole Open-Meteo** — humidité et température du sol,
évapotranspiration de référence FAO-56, déficit de pression de vapeur — sans
clé d'API.

```
core/   Cœur partagé TypeScript : règles agronomiques, codes météo, client Open-Meteo
ios/    Application iOS (SwiftUI, projet Xcode avec project.pbxproj versionné)
web/    Application web complète (Bun + TypeScript + Vite + React)
site/   Site de présentation, avec la météo du jour
```

Les trois paquets JavaScript forment un espace de travail Bun : `bun install` à la
racine les installe ensemble, et `bun test` y exécute la suite du cœur partagé.

L'application reprend la présentation de l'application Météo du système —
commune, température, bandeau horaire, liste des sept jours — et range les
indicateurs agronomiques dans les tuiles de détail.

## Ce que l'application calcule

iOS et web appliquent les mêmes règles, avec les mêmes seuils :

| Indicateur | Règle |
| --- | --- |
| **Bilan hydrique** | Pluie − ET0 (FAO-56) sur 7 jours ; alerte d'irrigation sous −15 mm |
| **État du sol** | Humidité volumique 3–9 cm : saturé ≥ 0,35, sec ≤ 0,12 ; portance et aptitude au semis (sol ≥ 8 °C) |
| **Fenêtre de traitement** | Vent 3–19 km/h (limite réglementaire), rafales < 25 km/h, pas de pluie sous 2 h, 5–25 °C, HR ≥ 40 %, VPD ≤ 1,2 kPa |
| **Pression maladie** | Heures d'humectation du feuillage (HR ≥ 90 % entre 8 et 30 °C), à la manière des tables de Mills |
| **Risque de gel** | Mini nocturne : faible ≤ 1 °C, modéré ≤ −2 °C, sévère ≤ −4 °C ; gelée blanche si le point de rosée est négatif |
| **Degrés-jours** | Moyenne plafonnée, base 10 °C, plafond 30 °C |

Le vent, les rafales et la pluie imminente sont **rédhibitoires** : ils rendent
l'heure inexploitable pour un traitement quel que soit le reste du score.

S'y ajoutent les éléments d'une météo classique : conditions du moment, codes
temps WMO traduits en pictogrammes, probabilité de pluie horaire, amplitude
thermique de la semaine, lever et coucher du soleil.

Les seuils sont définis une seule fois par plateforme et doivent rester
synchronisés : `core/src/agro.ts` (`AgroThresholds`), consommé par le web et le
site, et `ios/Klima/Models/AgroIndicators.swift` (`AgroThresholds`). Les deux
suites de tests couvrent les mêmes cas, pour que le conseil rendu soit
identique au champ.

## iOS

```bash
open ios/Klima.xcodeproj
```

Le projet est un `project.pbxproj` classique, versionné et modifiable
directement — pas de générateur ni de Fastlane. Cible iOS 17, deux cibles :
`Klima` (application) et `KlimaTests` (tests unitaires), avec un
schéma partagé.

```bash
xcodebuild -project ios/Klima.xcodeproj -scheme Klima \
  -destination 'platform=iOS Simulator,name=iPhone 15' test
```

Avant la première exécution sur appareil, renseignez votre équipe de signature
(`DEVELOPMENT_TEAM`) et, si besoin, votre propre `PRODUCT_BUNDLE_IDENTIFIER`
dans les réglages de la cible.

Organisation :

```
Klima/
  App/          Point d'entrée SwiftUI
  Models/       Types de mesure et cœur agronomique (AgroIndicators)
  Services/     Client Open-Meteo, relevé de position
  ViewModels/   État du tableau de bord
  Views/        Tableau de bord, bandeau horaire, liste des jours, tuiles
  Resources/    Info.plist, catalogue d'assets
```

## Web

L'application complète : bandeau horaire, semaine et tuiles agronomiques.

```bash
bun install        # à la racine, installe core, web et site
cd web
bun run dev        # http://localhost:5173
bun run typecheck
bun run build      # dist/
```

La parcelle est mémorisée dans le navigateur ; la recherche de commune passe par
le géocodage Open-Meteo et le bouton « Me localiser » par la géolocalisation du
navigateur. Le fond suit le ciel : nuit, journée couverte ou journée dégagée.

## Site de présentation

La vitrine de Klima : ce que fait l'application, et une section « météo du jour »
qui la fait essayer sur sa propre commune — la journée en cours uniquement, la
semaine et le détail horaire restant l'affaire de l'application.

```bash
cd site
bun run dev        # http://localhost:5174
bun run typecheck
bun run build      # dist/
```

Les seuils affichés dans la page sont lus dans `AgroThresholds` : la vitrine ne
peut pas annoncer autre chose que ce que l'application applique.

### Publication sur GitHub Pages

`.github/workflows/pages.yml` construit `site/` et le publie à chaque poussée
sur la branche par défaut (et à la demande, via *Run workflow*). Le déploiement
échoue si les tests du cœur partagé ou la vérification de types échouent : rien
d'incohérent n'est mis en ligne.

Une seule chose à faire côté dépôt, une fois : **Settings → Pages → Source →
GitHub Actions**. Sans cela le job `deploy` s'arrête faute d'environnement
`github-pages`.

Le site est construit avec des chemins d'actifs relatifs : il fonctionne sous le
sous-chemin d'un dépôt (`https://<compte>.github.io/<dépôt>/`) comme à la racine
d'un domaine personnalisé, sans rien reconfigurer.

## Cœur partagé

```bash
cd core
bun test           # règles agronomiques, codes météo, journée en cours, formats
bun run typecheck
```

`core` n'a pas d'étape de compilation : le web et le site l'importent en
TypeScript via l'alias `@klima/core`, et `@klima/core/ui` pour les pictogrammes.

## Données

[Open-Meteo](https://open-meteo.com/) — API libre, sans clé. Variables
interrogées : `soil_temperature_6cm`, `soil_moisture_3_to_9cm`,
`et0_fao_evapotranspiration`, `vapour_pressure_deficit`, plus la température,
l'hygrométrie, la pluie et le vent nécessaires aux fenêtres de traitement, et
`weather_code`, `is_day`, `sunrise`, `sunset` pour la présentation.

L'API renvoie les horodatages en heure locale de la parcelle et les séries
horaires depuis minuit : les deux clients les ramènent en instants absolus et
recoupent la série à l'heure en cours, pour que « maintenant » soit bien le
premier élément affiché.
