# Klima — conventions du dépôt

## Workflow

Commiter, ouvrir la pull request et **fusionner sur `master` sans demander**.
Maxime Nathan Lestage a donné cette autorisation permanente : ne pas
redemander à chaque changement. Livrer, puis rendre compte.

Rester sur la branche `claude/weather-app-ios-web-rempob`. Si sa pull request
est déjà fusionnée, repartir de `master` sous le même nom et ouvrir une
nouvelle pull request.

## Ce qui ne se négocie pas

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
- **Une seule source pour les règles agronomiques.** Les seuils vivent dans
  `core/src/agro.ts` (`AgroThresholds`) et dans son miroir Swift
  `ios/Kliima/Models/AgroIndicators.swift`. Toute règle ajoutée d'un côté se
  porte de l'autre, avec les mêmes cas de test.
- **Le domaine ne fabrique pas de phrases.** Il renvoie des états et des motifs
  structurés ; l'interface les traduit. Trois langues : français, anglais,
  espagnol, avec des catalogues dont les clés sont vérifiées par les tests.
- **Un défilement horizontal ne se cache pas.** Le bandeau horaire d'iOS
  glisse de côté — la grille repliable essayée entre-temps montrait tout d'un
  coup, mais prenait la moitié de l'écran et finissait sur une rangée
  ébréchée ; Maxime Nathan Lestage a tranché pour le bandeau. Ce qui ne revient
  pas, c'est `showsIndicators: false` : la première version défilait sans rien
  dire, on voyait six heures et il fallait deviner que les autres existaient.
  L'indicateur reste visible, et lui seul défile de côté. Un test à la racine
  le vérifie.
- **Le web, lui, ne défile pas de côté.** Aucune règle CSS ne rend un bloc
  défilable à l'horizontale ; le bandeau y est une grille qui se replie, et
  montre douze heures — une demi-journée — les autres se dépliant d'un bouton.
  Une carte ne prend pas tout l'écran.
- **On part de là où est la personne.** Une application météo qui s'ouvre sur
  une ville qu'on n'a pas choisie demande un geste avant d'être utile. La
  position n'est donc demandée qu'à défaut — jamais par-dessus une parcelle
  déjà choisie ni par-dessus un lien partagé — elle ne bloque pas l'affichage,
  et un refus ne dit rien. Les coordonnées sont arrondies à la maille avant de
  devenir une parcelle : une parcelle finit dans l'adresse et dans le groupe
  partagé, elle n'a pas à dire à deux mètres près où se tient quelqu'un. La
  règle vit dans `core/src/position.ts` et dans son miroir
  `ios/Kliima/Models/Position.swift`.
- **Les heures sont celles de la parcelle**, pas celles du lecteur. Les nombres
  et les dates suivent en revanche la langue de l'utilisateur.
- **Ne pas publier de lien vers le code source** sur le site de présentation.
- **Respecter les conditions des fournisseurs météo.** MET Norway exige un
  `User-Agent` identifiant. La règle n'est donc pas « natif seulement » mais
  **« seulement là où l'on peut se nommer »** : en appel direct, le natif ; par
  le relais, le serveur, qui pose l'en-tête pour tout le monde. Jamais depuis
  un navigateur en direct. Les mentions de licence s'affichent dès qu'une
  source est utilisée.
- **Le plan gratuit d'Open-Meteo est réservé à un usage non commercial.** Le
  jour où Klima se vend, tout le trafic passe par un plan payant, donc par une
  clé — qui vit dans `server/` et nulle part ailleurs. Une clé dans un binaire
  distribué est une clé publiée.

## Vérifications avant de livrer

```bash
bun test                       # cœur partagé, relais, web et site
cd core && bun run typecheck   # idem dans server/, web/ et site/
cd web && bun run build        # idem dans site/
```

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
