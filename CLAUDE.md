# Klima — conventions du dépôt

## Workflow

Commiter, ouvrir la pull request et **fusionner sur `master` sans demander**.
Maxime Nathan Lestage a donné cette autorisation permanente : ne pas
redemander à chaque changement. Livrer, puis rendre compte.

Rester sur la branche `claude/weather-app-ios-web-rempob`. Si sa pull request
est déjà fusionnée, repartir de `master` sous le même nom et ouvrir une
nouvelle pull request.

## Ce qui ne se négocie pas

- **Klima est la météo d'une ville.** Plus rien d'agricole : ni sol, ni
  traitement, ni registre. Ce que l'application dit, c'est ce qu'on regarde
  avant de sortir — la pluie qui vient, ce qu'il faut emporter, le ressenti,
  l'UV, l'air et les pollens, l'accord des sources. Les identifiants internes
  gardent leur nom d'époque (`Parcelle`, `parcelle_url`, `AgroFormat`…) : ce
  sont des noms de code, l'interface dit « ville ».

- **Toutes les sources font la prévision.** Elle n'est pas celle d'un modèle :
  sept modèles d'Open-Meteo, MET Norway et une station votent heure par heure
  (`klima-core/src/fusion.rs` et son miroir `ios/Kliima/Models/Fusion.swift`) —
  médiane pour ce qui se mesure, part des sources qui mouillent pour le risque,
  majorité pour le temps qu'il fait. Un modèle régional « sans couture » n'y
  entre pas : hors de son domaine il retombe sur l'ECMWF, qui voterait deux
  fois. Une source muette est écartée, jamais comptée pour zéro.

- **La météo est poussée, en direct.** Avec un relais, chaque interface ouvre
  `/v1/direct` (WebSocket, `klima-relay/src/direct.rs`) : le relais pousse les
  réponses brutes des fournisseurs dès qu'elles changent, lues par les mêmes
  caches que les requêtes — le direct ne coûte pas une interrogation de plus.
  Il s'ajoute aux requêtes sans les remplacer : la première prévision vient par
  HTTP, et une connexion tombée se rouvre seule (`crochets/direct.rs`,
  `DirectRelais.swift`, même protocole, mêmes cas de test).

- **Le guetteur relit, il ne garde pas.** La demi-heure en cours et les deux
  heures à venir viennent de la série au quart d'heure (`minutely_15`), dans
  un appel à part de la prévision horaire : `klima-core/src/veille.rs` et son
  miroir `ios/Kliima/Models/Veille.swift`. Les interfaces relisent une minute
  après chaque quart (`prochaine_lecture`) ; le relais garde ces réponses dix
  minutes, pas une heure. Sans série qui couvre le quart en cours, le
  guetteur se tait : un « sec » dit sans avoir regardé est un mensonge.

- **Ce qui tombe se voit, ça ne se vote pas.** Quelqu'un sous la pluie a lu
  « sec » : la majorité des modèles avait effacé l'averse. Pour l'instant
  présent, ce qu'un aéroport proche voit tomber (METAR de la NOAA, domaine
  public : `klima-core/src/ciel.rs`, miroir `ios/Kliima/Models/Ciel.swift`)
  l'emporte, puis ce que la prévision de base fait tomber ; le vote ne fait
  que peindre le ciel (`fusion::code_present`). Quand quelque chose tombe,
  l'heure en cours et le quart en cours du guetteur le disent aussi. La règle
  est à sens unique : un ciel sec observé ne retire jamais une pluie prévue.
  Le guetteur lit aussi la **tendance** du bulletin (`NOSIG`, `TEMPO`,
  `BECMG`, deux heures) et la force du signe (`+TSRA` est un orage fort) :
  ce qu'on voit tombe la demi-heure, deux heures sous `NOSIG`. Quand les
  modèles n'ont pas vu ce qui tombe (`veille::aveugle`), il ne donne pas de
  fin — une heure inventée est un mensonge — et dit ce que l'aéroport annonce.

- **Le radar voit ce que les modèles ratent.** La mosaïque européenne OPERA
  d'EUMETNET (radars de Météo-France et voisins, au kilomètre, toutes les
  cinq minutes, seau public sans clé, CC BY 4.0) est lue par le relais et
  nulle part ailleurs (`klima-relay/src/radar.rs`, 3 Mo l'image) ; la partie
  pure — projection, dBZ → mm/h, déplacement, extrapolation — vit dans
  `klima-core/src/radar.rs`. Les interfaces reçoivent `/v1/radar` (et le
  sujet `radar` du direct) et refont l'heure qui vient du guetteur
  (`veille::radariser`, miroir `Veille.radariser`) : le premier quart est au
  radar, le huitième pour un huitième. Quand le radar voit la ville, il fait
  foi pour ce qui tombe maintenant, avant l'aéroport. La mention de la source
  s'affiche dès qu'il sert. La montre, sans relais, ne l'a pas.

- **Deux noms, et ce n'est pas une coquille.** Le nouveau nom ne dépasse pas
  de `ios/`. L'application iPhone s'appelle **Kliima ‣** — le triangle
  (U+2023) fait partie du nom, ce n'est pas de la décoration. Tout le reste —
  le dépôt, le site, l'application web, les catalogues partagés — reste
  **Klima**. Les libellés partagés (`plan.libre`, `plan.pro`) disent donc
  Klima ; seul le catalogue iOS dit Kliima ‣. La divergence est voulue : ne
  pas « corriger » l'un d'après l'autre.
- **Le symbole ne vit que dans le nom affiché.** `CFBundleDisplayName` et les
  textes où l'application se nomme le portent. Il ne peut pas aller ailleurs :
  un identifiant de paquet n'accepte que lettres, chiffres, tirets et points ;
  le nom du bundle `.app` vient de `PRODUCT_NAME` ; et un caractère non ASCII
  dans un nom de cible ou de dossier casserait les chemins de compilation. Les
  cibles, les dossiers et les identifiants restent donc `Kliima` tout court.
  Un test vérifie que le nom affiché n'a pas perdu son triangle.
- **Une seule source pour les règles de la ville.** Les seuils vivent dans
  `rust/klima-core/src/ville.rs`, `air.rs` et `alerts.rs` (modules `seuils`)
  et dans leurs miroirs Swift `ios/Kliima/Models/Ville.swift`, `Air.swift` et
  `Alerts.swift` ; les villes enregistrées dans `villes.rs` et `Villes.swift`
  (une au palier libre, autant qu'on veut en Pro, une par maille). Toute règle ajoutée d'un côté se porte de l'autre, avec les
  mêmes cas de test.
- **Le domaine ne fabrique pas de phrases.** Il renvoie des états et des motifs
  structurés ; l'interface les traduit. Trois langues : français, anglais,
  espagnol, avec des catalogues dont les clés sont vérifiées par les tests.
- **Un défilement horizontal ne se cache pas.** Le bandeau horaire d'iOS
  glisse de côté — la grille repliable essayée entre-temps montrait tout d'un
  coup, mais prenait la moitié de l'écran et finissait sur une rangée
  ébréchée ; Maxime Nathan Lestage a tranché pour le bandeau. Ce qui ne revient
  pas, c'est `showsIndicators: false` : la première version défilait sans rien
  dire, on voyait six heures et il fallait deviner que les autres existaient.
  L'indicateur reste visible, et lui seul défile de côté.
  `rust/klima-web/tests/defilement.rs` le vérifie : il porte sur les deux
  plateformes à la fois, et lit les vues SwiftUI.
- **Le web, lui, ne défile pas de côté.** Aucune règle CSS ne rend un bloc
  défilable à l'horizontale ; le bandeau y est une grille qui se replie, et
  montre douze heures — une demi-journée — les autres se dépliant d'un bouton.
  Une carte ne prend pas tout l'écran.
- **On part de là où est la personne.** Une application météo qui s'ouvre sur
  une ville qu'on n'a pas choisie demande un geste avant d'être utile. La
  position n'est donc demandée qu'à défaut — jamais par-dessus une ville
  déjà choisie ni par-dessus un lien partagé — elle ne bloque pas l'affichage,
  et un refus ne dit rien. Les coordonnées sont arrondies à la maille avant de
  devenir une ville : elle finit dans l'adresse et dans le groupe partagé,
  elle n'a pas à dire à deux mètres près où se tient quelqu'un. La
  règle vit dans `rust/klima-core/src/position.rs` et dans son miroir
  `ios/Kliima/Models/Position.swift`.
- **Les heures sont celles de la ville**, pas celles du lecteur. Les nombres
  et les dates suivent en revanche la langue de l'utilisateur.
- **Ne pas publier de lien vers le code source** sur le site de présentation.
- **Respecter les conditions des fournisseurs météo.** MET Norway exige un
  `User-Agent` identifiant. La règle n'est donc pas « natif seulement » mais
  **« seulement là où l'on peut se nommer »** : en appel direct, le natif ; par
  le relais, le serveur, qui pose l'en-tête pour tout le monde. Jamais depuis
  un navigateur en direct. Les mentions de licence s'affichent dès qu'une
  source est utilisée — Copernicus compris, dès qu'une mesure d'air est
  montrée.
- **Ce que le déploiement accorde vit sur le serveur.** Le palier payant
  s'ouvre pendant l'essai par la variable `KLIMA_PRO` du relais, jamais par une
  valeur glissée dans l'application : une valeur distribuée est une valeur
  publiée, qu'on ne retire qu'en publiant une version. L'accord **s'ajoute** à
  ce que dit StoreKit, il ne le remplace pas — un relais muet ne fait pas
  perdre un abonnement réel. Le relais ne répète jamais ce qu'il attend, ni
  dans sa réponse ni dans `/health` : d'un refus on ne peut pas déduire qu'une
  adresse est inconnue, sinon la liste s'énumère une adresse à la fois.
- **Un compte par personne, prouvé par Apple.** Maxime Nathan Lestage a
  tranché : pas de `tous`, des comptes séparés. Sur iPhone, « Se connecter
  avec Apple » prouve l'adresse ; le relais vérifie le jeton (signature RS256
  avec les clés publiques d'Apple, émetteur, destinataire `com.kliima.app`,
  échéance) et rend une session qu'il signe avec `KLIMA_SESSION_SECRET`. La
  session prouve une identité, elle n'accorde rien : c'est `KLIMA_PRO`, liste
  d'adresses, qui décide à chaque question — retirer quelqu'un de la liste lui
  retire l'accès tout de suite. Pas de mot de passe, pas de base de données.
  Le secret vit dans le relais et nulle part ailleurs ; sans lui, les comptes
  sont désactivés et le relais le dit. La session va dans le trousseau, et
  dans l'en-tête `Authorization`, jamais dans une adresse. `KliimaProCode`
  reste vide dans le dépôt, et un test le vérifie. Le site, lui, n'a pas
  encore de comptes : son champ d'adresse n'est pas une preuve, et la
  documentation le dit.
- **L'île dynamique se tient à l'heure par le relais.** L'iPhone calcule
  l'heure suivante d'avance et pose la péremption au début de l'heure qui
  vient : l'île bascule seule, même sans réseau. Le relais, s'il a une clé
  APNs (`KLIMA_APNS_KEY`, `_KEY_ID`, `_TEAM_ID` — dans le relais et nulle part
  ailleurs), pousse ce qui change ; il ne garde que le jeton et la maille, en
  mémoire. Le `content-state` qu'il produit (`klima-relay/src/iles.rs`) et
  celui que décode `WeatherActivityAttributes.swift` sont un seul contrat,
  vérifié des deux côtés : les dates y sont en secondes depuis 2001.
- **Le plan gratuit d'Open-Meteo est réservé à un usage non commercial.** Le
  jour où Klima se vend, tout le trafic passe par un plan payant, donc par une
  clé — qui vit dans le relais et nulle part ailleurs. Une clé dans un binaire
  distribué est une clé publiée.

## Vérifications avant de livrer

```bash
cd rust && cargo test          # cœur, formats de fil, relais
cd rust && cargo clippy --all-targets
cd rust && cargo test --manifest-path klima-web/Cargo.toml   # idem klima-site/
cd rust && trunk build --config klima-web/Trunk.toml         # idem klima-site/
```

Les interfaces en Yew ne sont pas dans l'espace de travail : elles se
construisent pour le navigateur, et les laisser dedans ferait compiler Yew à
chaque `cargo test`. Leurs propres tests tiennent les catalogues — et, pour
`klima-web`, le garde-fou du défilement horizontal, qui lit aussi les vues
SwiftUI.

Le `project.pbxproj` est versionné et ouvrable tel quel par Xcode ; les outils
de `ios/Tools/` le régénèrent depuis une liste de fichiers lisible, ce qui évite
d'inventer des identifiants à la main. Après tout ajout de fichier iOS :

```bash
python3 ios/Tools/gen_pbxproj.py ios/Kliima.xcodeproj/project.pbxproj
python3 ios/Tools/validate_pbxproj.py ios/Kliima.xcodeproj/project.pbxproj
```

Même chose pour les textes traduits d'iOS : la table lisible est en tête de
`ios/Tools/gen_xcstrings.py`, et les trous y sont positionnels (`%1$@`).

## Marque

Le signe, le favicon, l'icône d'application et l'image de partage sont rendus
depuis les gabarits de `site/public/` et `/tmp/brand` — un seul dessin, décliné.
La police d'affichage est auto-hébergée : ne jamais la remplacer par un appel à
un service tiers.

## Crédits

Le site mentionne **Maxime Nathan Lestage** comme concepteur et développeur.
