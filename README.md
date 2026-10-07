# Klima

La météo d'une ville : une application iOS en SwiftUI (avec sa montre et ses
widgets), une application web et un site de présentation, tous bâtis sur le
même cœur. Klima répond aux questions qu'on se pose avant de sortir — va-t-il
pleuvoir, et quand ; que faut-il emporter ; le soleil tape-t-il ; l'air est-il
bon — à partir des modèles des grands instituts, redistribués par
**Open-Meteo**, des prévisions d'air de **Copernicus**, et de trois
fournisseurs indépendants recoupés.

**Deux noms, et c'est voulu.** Le projet, le site et l'application web
s'appellent **Klima** ; l'application iPhone et sa montre s'appellent
**Kliima ‣**. Le signe, les couleurs et le cœur sont les mêmes.

```
rust/   Tout ce qui tourne hors iOS, en Rust
  klima-core/   Cœur : règles de la ville, air, alertes, codes météo, langues, paliers
  klima-api/    Formats de fil : adresses des fournisseurs, lecture des réponses
  klima-relay/  Le relais (Axum) : cache mutualisé, clé commerciale, site servi
  klima-ui/     Ce que les deux interfaces web partagent (Yew)
  klima-web/    Application web complète (Yew + WebAssembly)
  klima-site/   Site de présentation, avec la météo du jour (Yew + WebAssembly)
ios/    Application iOS (SwiftUI, projet Xcode avec project.pbxproj versionné)
```

Le tout est traduit en **français, anglais et espagnol**.

Le dépôt a tenu une version TypeScript de tout cela — cœur, relais, deux
interfaces React. Elle est partie : les crates la reproduisent, cas de test
pour cas de test, et ont été essayées contre les vrais fournisseurs. Ce qui
reste à faire pour que le relais Rust prenne la main en production est dans
[`rust/DEPLOIEMENT.md`](rust/DEPLOIEMENT.md).

L'application reprend la présentation de l'application Météo du système —
ville, température, bandeau horaire, liste des sept jours — et y ajoute ce
qu'une ville regarde : la pluie qui vient, ce qu'il faut emporter, l'UV, l'air
et les pollens, l'accord des sources.

## Langues

Le domaine ne fabrique jamais de phrase : il renvoie des états et des motifs
structurés — `Pluie::Prevue { debut, probabilite, cumul }`, `Conseil::Parapluie`,
`QualiteAir::Mediocre` — que l'interface traduit. Les textes vivent donc dans des catalogues, jamais dans le
code de calcul.

| Surface | Catalogue | Choix de la langue |
| --- | --- | --- |
| Commun web et site | `rust/klima-core/src/messages.rs` | — |
| Application web | `rust/klima-web/src/messages.rs` | Sélecteur, sinon le navigateur |
| Site de présentation | `rust/klima-site/src/messages.rs` | Sélecteur, sinon le navigateur |
| iOS | `ios/Kliima/Resources/Localizable.xcstrings` | Réglages du système |

Les nombres et les dates suivent la langue : virgule décimale en français et en
espagnol, point en anglais ; horloge sur 24 h en français, sur 12 h en anglais
américain. Les heures restent en revanche celles du fuseau de la ville, pas
celui du lecteur.

Les suites de tests vérifient que les trois langues portent exactement les mêmes
clés et les mêmes valeurs à interpoler : une traduction oubliée fait échouer la
compilation, elle n'apparaît pas en clair dans l'application.

## Plusieurs fournisseurs, recoupés

Un seul service donne un chiffre ; plusieurs donnent un chiffre **et** une idée
de sa fiabilité.

| Fournisseur | Sources | Nature | Plateformes |
| --- | --- | --- | --- |
| Open-Meteo | Météo-France (AROME/ARPEGE), ECMWF (IFS), DWD (ICON), NOAA (GFS), Met Office (UM), ECCC (GEM), JMA (GSM) | Sorties de modèles | Web et natif |
| MET Norway | Locationforecast 2.0 | Sortie de modèle | Natif, ou web **par le relais** |
| Bright Sky | Observation DWD | Mesure de station | Web et natif |

MET Norway impose un en-tête `User-Agent` identifiant l'application ; un
navigateur interdit de le fixer. La règle n'est donc pas « natif seulement »
mais **« seulement là où l'on peut se nommer »** : en appel direct, iOS et
watchOS, où `URLSession` le permet ; par le relais, le serveur, qui pose
l'en-tête pour tout le monde — et le web gagne alors la même source que le
natif. Jamais de requête anonyme contre leur volonté. Bright Sky apporte un point de comparaison d'une autre nature : une
observation de station, qui dit ce qu'il fait et non ce qui est prévu ; sa
couverture suit le réseau du DWD.

**Chaque fournisseur est isolé** : une panne, un refus ou une absence de
couverture n'en écarte qu'un, et l'interface annonce combien ont répondu.

La valeur retenue est la **médiane**, moins sensible qu'une moyenne à une source
isolée. L'accord est jugé fort quand les sources tiennent dans 1,5 °C et
s'entendent sur la pluie, faible au-delà de 3 °C d'écart. Quand elles divergent,
l'application le dit plutôt que d'afficher une fausse précision.

### Le direct : la météo poussée par WebSocket

Avec un relais, le web, le site et l'iPhone ouvrent une connexion WebSocket
(`/v1/direct`) et s'abonnent à la ville affichée. Le relais pousse alors six
sujets — la prévision de base et l'instant présent, les sept modèles, le quart
d'heure du guetteur, MET Norway, la station, l'air —, chacun dans la forme
exacte que le fournisseur renvoie : les clients les lisent avec les décodeurs
de leurs requêtes. Toutes les trente secondes, le relais relit chaque sujet
**par les mêmes caches que les requêtes HTTP**, et ne pousse que ce qui a
changé. Les caches fixent le rythme des fournisseurs : cinq minutes pour le
quart d'heure, dix pour l'instant présent et la station, une heure pour les
modèles ; le direct fixe celui des écrans. Un battement part toutes les
vingt-cinq secondes (Heroku coupe une connexion muette à cinquante-cinq).

Le direct s'ajoute aux requêtes, il ne les remplace pas : la première
prévision arrive par HTTP, et une connexion tombée se rouvre après deux
secondes, puis quatre, huit… jusqu'à une minute. Le widget, la montre et les
tâches de fond, qui n'ont pas de connexion qui dure, restent sur les requêtes.

### La prévision elle-même est recoupée

Les sources ne servent pas qu'à dire leur accord : elles **font** la prévision
(`klima-core/src/fusion.rs`, miroir `ios/Kliima/Models/Fusion.swift`). Pour
chaque heure et chaque jour, chaque source vote :

- **la médiane** pour ce qui se mesure — température, ressenti, pluie, vent,
  rafales, minimales et maximales du jour ;
- **le risque de pluie** : la part des sources qui annoncent au moins 0,1 mm,
  mêlée à moitié au risque que les modèles publient eux-mêmes ;
- **le temps qu'il fait** : la majorité mouillée ou sèche, puis le code le plus
  cité de ce camp — à égalité, le plus marqué ;
- **l'instant présent** : la base décalée d'autant que l'heure recoupée, puis la
  médiane avec la station de Bright Sky s'il y en a une à portée. Mais **ce
  qui tombe maintenant ne se vote pas** (`code_present`) : la majorité des
  modèles mondiaux effaçait l'averse que le modèle le plus fin voyait. Ce
  qu'un aéroport voit tomber l'emporte, puis ce que la base fait tomber ; le
  vote ne fait que peindre le ciel. Quand quelque chose tombe, l'heure en
  cours devient pluvieuse — 50 % au moins, 100 % si on l'a vu.

### Le ciel observé : les aéroports

Toutes les autres sources sont des prévisions ; une averse qu'aucun modèle
n'a vue venir n'existait pour aucune. Les aéroports, eux, regardent : leur
bulletin METAR dit le temps présent (`-RA`, `+SHRA`, `TS`…). L'Aviation
Weather Center de la NOAA les publie tous, dans le domaine public
(`klima-core/src/ciel.rs`, `klima-api/src/ciel.rs`, miroir
`ios/Kliima/Models/Ciel.swift`, mêmes cas de test).

On garde le bulletin le plus proche à moins de 30 km et d'au plus 1 h 15. La
règle est à sens unique : une pluie vue s'ajoute à la prévision — l'en-tête,
l'heure en cours, et le quart en cours du guetteur (`veille::observer`) —, un
ciel sec vu n'en retire rien : un aéroport à quinze kilomètres qui ne voit
rien ne prouve pas qu'il ne pleut pas en ville. L'en-tête le dit alors :
« Vu à l'aéroport de Quimper/Pluguffan (5 km) à 16:00 : pluie modérée »,
avec la mention de la source. Le service demande qu'on se nomme : le natif
l'appelle en direct avec son `User-Agent`, le web par le relais
(`/v1/aviation/metar`, gardé dix minutes, et poussé par le direct sous le
sujet `ciel`).

Le bulletin dit plus que l'instant : sa **tendance** vaut deux heures, la
fenêtre exacte du guetteur. `NOSIG` (rien ne changera) fait durer ce qu'on
voit tomber deux heures ; `TEMPO TSRA` (des orages par moments) s'ajoute aux
phrases du guetteur — « Prévision de l'aéroport de Bordeaux/Mérignac : orage
par moments d'ici 19:30 » ; `BECMG NSW` en annonce la fin. La force vient du
signe : `+TSRA` est un orage fort, pas un orage tout court. Et quand les
modèles n'ont pas vu ce qui tombe — ce jour-là, à Bordeaux, tous donnaient
zéro sous un `+TSRA` —, le guetteur ne donne pas de fin : « Les modèles ne
voient pas ce qui tombe : sa fin ne se prévoit pas. »

### Le radar : l'heure qui vient, vue d'en haut

Les modèles ratent les cellules ; le radar les voit. La mosaïque européenne
**OPERA** d'EUMETNET assemble toutes les cinq minutes les radars de
Météo-France et de ses voisins, au kilomètre, dans un seau public sans clé
(CC BY 4.0) :
`https://s3.waw3-1.cloudferro.com/openradar-24h/AAAA/MM/JJ/OPERA/COMP/OPERA@AAAAMMJJTHHMM@0@DBZH.tiff`.

Le relais (`klima-relay/src/radar.rs`) télécharge l'image la plus récente —
une pour toutes les villes, 3 Mo, publiée sept à dix minutes après son
heure — et celle d'un quart d'heure plus tôt, lit le GeoTIFF à la main (en-tête,
tuiles deflate de 512 pixels, décompressées à la demande) et vérifie que la
projection est bien celle qu'il attend. Le cœur (`klima-core/src/radar.rs`)
fait le reste, en pur : projection azimutale équivalente de Lambert,
réflectivité en débit (Marshall–Palmer, Z = 200 R^1,6), déplacement des
averses par recherche du meilleur décalage entre les deux images, puis
extrapolation sur huit quarts. `/v1/radar?lat=…&lon=…` rend le résultat (le
débit du moment, les huit quarts, la vitesse et la direction), gardé cinq
minutes, et le direct le pousse sous le sujet `radar`.

Le guetteur en refait l'heure qui vient (`veille::radariser`) : le premier
quart au radar, le huitième pour un huitième, la prévision au quart d'heure
pour le reste ; il dit d'où viennent les averses (« Les averses filent vers
le nord à 28 km/h ») et cite sa source. Quand le radar voit la ville, il fait
foi pour ce qui tombe maintenant, avant l'aéroport, qui regarde à quelques
kilomètres. Le web et l'iPhone l'ont par le relais ; la montre, qui
interroge les fournisseurs elle-même, ne l'a pas.

Ce qu'une seule source fournit — humidité, point de rosée, UV, jour ou nuit,
lever et coucher — reste celui de la prévision de base (le « meilleur » modèle
d'Open-Meteo pour le point). Une heure qu'aucune source ne couvre garde ses
valeurs de base. Les sept modèles d'Open-Meteo arrivent en **une** requête
(`klima-api/src/ensemble.rs`), dont la même réponse sert aussi l'accord ; MET
Norway entre par ses pas horaires, sur deux jours et demi.

La prévision de base s'affiche d'abord ; la prévision recoupée la remplace
quand les sources ont répondu, et l'en-tête dit « Recoupée de 8 sources ». Si
elles se taisent toutes, la base reste servie. Sur iPhone, le service rend
directement la prévision recoupée : le widget, la montre et les alertes ont
la même que l'écran. Les seuils vivent dans
`ConsensusThresholds`, des deux côtés, avec les mêmes cas de test.

Les licences imposent des mentions : elles sont affichées sous la comparaison,
une par licence effectivement utilisée.

## Ce que l'application dit

iOS et web appliquent les mêmes règles, avec les mêmes seuils :

| Réponse | Règle |
| --- | --- |
| **Le guetteur** | Au quart d'heure, relu tous les quarts d'heure : la demi-heure en cours (le quart entamé et le suivant) — sec, ça commence, ça dure, ça cesse —, puis les deux heures — sec, une averse entre deux heures avec son cumul, une pluie qui ne s'arrête pas, une accalmie et sa reprise. Un quart est mouillé dès 0,1 mm ; faible sous 2,5 mm/h, forte dès 7,6 mm/h ; neige et orage d'après le code météo ; rafales signalées dès 50 km/h |
| **Mes villes** (iPhone) | Le palier libre garde une ville ; Kliima ‣ Pro autant qu'on veut, chacune avec sa température du moment, à un geste. Deux villes de la même maille de prévision (0,02°) sont la même. Après une résiliation, rien n'est effacé : les villes au-delà de la limite restent, fermées d'un cadenas, et reviennent avec l'abonnement |
| **Pluie à venir** | Sur 12 h : une heure est pluvieuse dès 0,1 mm ou 50 % de risque. Klima dit si rien n'est prévu, s'il pleut et quand ça cesse, ou quand la pluie arrive, avec son risque et son cumul |
| **À emporter** | Parapluie s'il pleut ; manteau sous 10 °C ressentis ; lunettes dès l'indice UV 3, crème dès 6 ; eau au-delà de 30 °C ; prudence sous 0 °C ; gare au parapluie dès 50 km/h de rafales |
| **Indice UV** | Échelle de l'OMS (faible, modéré, élevé, très élevé, extrême), arrondie avant d'être classée |
| **Qualité de l'air** | Indice européen et ses six classes, de bonne à extrêmement médiocre ; particules fines |
| **Pollens** | Six espèces (aulne, bouleau, graminées, armoise, olivier, ambroisie), le dominant nommé avec son intensité ; Europe seulement |
| **Alertes** | Pluie dans les 2 h tant qu'il fait sec, orage, gel, chaleur dès 33 °C, rafales dès 70 km/h ; jamais entre 22 h et 7 h, six heures de garde par nature |

S'y ajoutent les éléments d'une météo classique : conditions du moment,
ressenti, humidité et point de rosée, vent et rafales, pression, codes temps
WMO traduits en pictogrammes, amplitude thermique de la semaine, lever et
coucher du soleil.

Les seuils sont définis une seule fois par plateforme et doivent rester
synchronisés : `rust/klima-core/src/ville.rs`, `air.rs` et `alerts.rs`
(modules `seuils`), et leurs miroirs `ios/Kliima/Models/Ville.swift`,
`Air.swift` et `Alerts.swift`. Les deux suites de tests couvrent les mêmes cas.

## iOS

```bash
open ios/Kliima.xcodeproj
```

Le projet est un `project.pbxproj` classique, versionné et modifiable
directement — pas de Fastlane, pas de CocoaPods, pas d'étape de résolution.
Cible iOS 18, cinq cibles : `Kliima` (application), `KliimaWidgets`,
`KliimaWatch`, `KliimaWatchWidgets` et `KliimaTests`, avec deux schémas
partagés.

Ajouter un fichier veut dire retoucher une demi-douzaine de sections dans
soixante kilo-octets de plist, avec des identifiants à inventer : `ios/Tools/`
s'en charge, et un validateur relit le résultat. Les identifiants étant dérivés
d'un hachage, régénérer un arbre propre ne produit aucune modification — c'est
vérifiable en une commande.

```bash
python3 ios/Tools/gen_pbxproj.py ios/Kliima.xcodeproj/project.pbxproj
python3 ios/Tools/validate_pbxproj.py ios/Kliima.xcodeproj/project.pbxproj
```

```bash
xcodebuild -project ios/Kliima.xcodeproj -scheme Kliima \
  -destination 'platform=iOS Simulator,name=iPhone 15' test
```

Avant la première exécution sur appareil, renseignez votre équipe de signature
(`DEVELOPMENT_TEAM`, laissée vide dans le projet) et, si besoin, votre propre
`PRODUCT_BUNDLE_IDENTIFIER` dans les réglages de la cible.

Pour un envoi TestFlight, `.github/workflows/testflight.yml` fait le travail
sur un exécuteur macOS, sur déclenchement manuel : tests, archive, export,
envoi. La marche à suivre — identifiants à enregistrer chez Apple, six secrets
à poser — est dans [`ios/Tools/TESTFLIGHT.md`](ios/Tools/TESTFLIGHT.md).

Les cinq cibles se partagent le même noyau (`Kliima/Models`) :

```
Kliima/           Application iPhone
  App/           Point d'entrée SwiftUI
  Models/        Types de mesure, règles de la ville et de l'air, formats, textes
  Services/      Client Open-Meteo, position, activité en direct
  ViewModels/    État du tableau de bord
  Views/         Tableau de bord, bandeau horaire, liste des jours, tuiles
  Resources/     Info.plist, assets, catalogues de chaînes
KliimaWidgets/    Extension iOS : activité en direct et widget d'écran d'accueil
KliimaWatch/      Application watchOS autonome
KliimaWatchWidgets/ Extension watchOS : complications de cadran
```

L'application, ses widgets et
la complication lisent la même ville via un **groupe d'applications**
(`group.com.kliima.app`) : il doit être déclaré dans le compte développeur avant
la première compilation signée.

### Activité en direct et île dynamique

Un bouton sous la température ouvre le suivi de la météo : l'écran verrouillé
et l'île dynamique montrent la température, le ciel, le vent et l'heure qui
vient, calculée d'avance. La péremption tombe au début de l'heure suivante :
l'île bascule seule à l'heure pile, même sans réseau. Le relais, s'il a une clé
APNs, pousse ce qui change en temps réel — voir
[`rust/DEPLOIEMENT.md`](rust/DEPLOIEMENT.md).

### Widgets d'écran d'accueil

Cinq widgets, chacun pour une question qu'on se pose devant la porte, tous
sur la même prévision (une entrée par heure calculée d'avance, qui bascule
seule) et avec le ciel qu'il fait en fond :

- **Météo** (petit, moyen, grand) : la température, le ciel, les bornes du
  jour et la pluie qui vient ; en moyen, quatre heures sur un panneau de
  verre ; en grand, le ressenti, le vent et l'UV en tuiles, puis six heures.
- **Pluie** (petit, moyen, grand) : la phrase de la carte Pluie et les douze
  heures en barres ; en moyen, ce qu'il faut emporter ; en grand, de hautes
  barres avec leurs heures.
- **Prochaines heures** (moyen, grand) : six heures ; en grand, les quatre
  jours suivants.
- **La semaine** (moyen, grand) : les jours, du plus frais au plus chaud, sur
  une même échelle.
- **Pour sortir** (petit, moyen, grand) : le ressenti, le vent, l'UV et ce
  qu'il faut emporter ; en grand, l'humidité et la pluie en plus.

Les widgets montrent la ville choisie dans l'application quand le groupe
d'applications la leur transmet. Sans lui, ils partent de la position — si
l'on a permis la localisation « lorsque l'app ou les widgets sont actifs »
(`NSWidgetWantsLocation`) —, arrondie à la maille et nommée par sa commune ;
Paris en dernier recours. Un widget ne demande jamais l'autorisation : il ne
le peut pas.

Et la météo sur l'écran verrouillé.

### Complication de cadran

`KliimaWatchWidgets` fournit les quatre formes de watchOS — circulaire,
rectangulaire, en ligne et d'angle — avec la température de la ville.

### Application montre

`KliimaWatch` est une application watchOS autonome : elle interroge l'API
elle-même et se cale sur la position du poignet, sans passer par le téléphone.
Sur iOS 18, l'activité en direct déclare aussi la famille `.small` : dans
la pile intelligente de la montre, elle a sa propre vue (ville, température,
ciel, bornes du jour, ressenti, l'heure qui vient) au lieu du seul compact de
l'île dynamique.

Elle s'habille comme l'iPhone : le ciel sombre et animé derrière
(`CielVivant`, partagé), la température en grand et le nombre de sources
recoupées, le guetteur — la demi-heure en une phrase et les huit quarts en
barres, relus une minute après chaque quart —, la pluie qui vient et ce qu'il
faut emporter, les quatre heures suivantes (sur la largeur du cadran, rien ne
défile), puis le ressenti, le vent, l'UV et l'humidité. Les phrases du
guetteur viennent du même `VeilleTextes` que la carte de l'iPhone.

```bash
xcodebuild -project ios/Kliima.xcodeproj -scheme KliimaWatch \
  -destination 'platform=watchOS Simulator,name=Apple Watch Series 9 (45mm)' build
```

## Web

L'application complète : la pluie et ce qu'il faut emporter, le bandeau
horaire, la semaine, ce que dit chaque source, et les tuiles — ressenti,
humidité, vent, UV, pression, qualité de l'air, pollens.

```bash
cd rust/klima-web
trunk serve        # http://localhost:8081
trunk build --release
```

Rien ne défile de côté : le bandeau horaire est une grille qui se replie et
montre douze heures — une demi-journée —, les autres se dépliant d'un bouton.
Une carte ne prend pas tout l'écran.


La ville vit dans l'adresse et dans le navigateur ; la recherche de ville passe par
le géocodage Open-Meteo et le bouton « Me localiser » par la géolocalisation du
navigateur. Le fond suit le ciel : nuit, journée couverte ou journée dégagée.

## Site de présentation

La vitrine de Klima : ce que fait l'application, et une section « météo du jour »
qui la fait essayer sur sa propre ville — la journée en cours uniquement, la
semaine et le détail horaire restant l'affaire de l'application.

```bash
cd rust/klima-site
trunk serve        # http://localhost:8082
trunk build --release
```


Les seuils affichés dans la page sont lus dans les modules `seuils` du cœur : la
vitrine ne peut pas annoncer autre chose que ce que l'application applique.

### Marque et typographie

Le signe de Klima — un K dont la hampe porte une goutte et dont les bras sont
taillés en lames de feuille — tient à vingt-deux pixels comme sur une icône
d'application. Il sert de favicon (SVG et PNG), d'icône iOS et watchOS, et de
marque dans la barre et le pied de page. Un seul dessin, décliné depuis les
gabarits de `site/public/` et `/tmp/brand`.

L'écriture porte la marque : **Fraunces** pour tout ce qui s'annonce, la linéale
du système pour tout ce qui se lit. La police est **servie depuis le site**
(`site/public/fonts/`, licence OFL incluse) : aucun appel à un tiers, donc rien
à déclarer côté données personnelles, et un chargement de moins.

Les liens partagés affichent une image : `site/public/og.png`, 1200 × 630,
déclarée en `og:image` et `twitter:image`. Elle est rendue depuis un gabarit
HTML, comme les icônes — le même signe partout, sans retouche manuelle.

Les illustrations sont dessinées en SVG — rien à licencier, rien à charger, et
le trait reste net à toutes les tailles. Les animations sont en CSS et se
coupent d'elles-mêmes sous `prefers-reduced-motion`. Les apparitions au
défilement ne masquent jamais un contenu sans savoir pouvoir le ramener : sans
JavaScript ou sans `IntersectionObserver`, la page reste lisible.

### Publication sur GitHub Pages

`.github/workflows/pages.yml` construit les deux interfaces Yew et les publie à
chaque poussée sur la branche par défaut (et à la demande, via *Run workflow*) :
la vitrine à la racine, l'application sous `/app/`. Le déploiement échoue si les
tests du cœur échouent : rien d'incohérent n'est mis en ligne.

Une seule chose à faire côté dépôt, une fois : **Settings → Pages → Source →
GitHub Actions**. Sans cela le job `deploy` s'arrête faute d'environnement
`github-pages`.

Le site est construit avec des chemins d'actifs relatifs : il fonctionne sous le
sous-chemin d'un dépôt (`https://<compte>.github.io/<dépôt>/`) comme à la racine
d'un domaine personnalisé, sans rien reconfigurer.

## Cœur partagé

```bash
cd rust
cargo test                 # cœur, formats de fil, relais
cargo clippy --all-targets
```

`klima-core` n'a **aucune dépendance** — pas même une bibliothèque de dates. Ce
n'est pas de l'ascétisme : c'est ce qui lui permet de compiler pour un serveur
comme pour un navigateur, et de ne jamais dépendre d'une mise à jour qui change
un arrondi. Les horodatages y sont des millisecondes depuis l'époque, et ce sont
celles **de la ville** : une journée se découpe par division, pas en
demandant à un fuseau.

Lire du JSON demande une dépendance : c'est pourquoi `klima-api` existe à côté,
avec `serde_json`. Les règles restent pures d'un côté, le décodage du fil vit de
l'autre.

Le relais tourne sur Axum, les interfaces sur Yew et WebAssembly. Elles ne sont
pas dans l'espace de travail : elles se construisent pour le navigateur, et les
y laisser ferait compiler Yew à chaque `cargo test`.

## Données

[Open-Meteo](https://open-meteo.com/) — API libre, sans clé pour un usage non
commercial. Variables interrogées : température et ressenti, humidité et point
de rosée, pluie et probabilité, vent et rafales, indice UV, pression, et
`weather_code`, `is_day`, `sunrise`, `sunset` pour la présentation. La qualité
de l'air et les pollens viennent du service d'air d'Open-Meteo, qui redistribue
les prévisions européennes de [Copernicus](https://atmosphere.copernicus.eu/)
(CC BY 4.0) ; la mention s'affiche dès qu'une mesure d'air est montrée.

Le guetteur lit à part la série au quart d'heure (`minutely_15`, le quart
entamé et les dix suivants) : elle se relit tous les quarts d'heure, la
prévision horaire non. Le relais la garde dix minutes, pas une heure. Elle est
calculée par des modèles à maille fine en Europe centrale et en Amérique du
Nord, et interpolée depuis l'heure ailleurs.

L'API renvoie les horodatages en heure locale de la ville et les séries
horaires depuis minuit. Les clients recoupent la série à l'heure en cours, pour
que « maintenant » soit bien le premier élément affiché.

Côté Rust, l'heure locale est **conservée telle quelle** plutôt que ramenée en
instant absolu : « 21 h » veut alors dire 21 h là-bas, quel que soit le fuseau
du serveur qui calcule. Le décalage n'est pas perdu pour autant — la prévision
le porte, et `instant()` rend l'instant absolu pour qui en a besoin, un minuteur
d'écran verrouillé par exemple.
