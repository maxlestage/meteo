# Klima — conventions du dépôt

## Workflow

Commiter, ouvrir la pull request et **fusionner sur `master` sans demander**.
Maxime Nathan Lestage a donné cette autorisation permanente : ne pas
redemander à chaque changement. Livrer, puis rendre compte.

Rester sur la branche `claude/weather-app-ios-web-rempob`. Si sa pull request
est déjà fusionnée, repartir de `master` sous le même nom et ouvrir une
nouvelle pull request.

## Ce qui ne se négocie pas

- **Une seule source pour les règles agronomiques.** Les seuils vivent dans
  `core/src/agro.ts` (`AgroThresholds`) et dans son miroir Swift
  `ios/Klima/Models/AgroIndicators.swift`. Toute règle ajoutée d'un côté se
  porte de l'autre, avec les mêmes cas de test.
- **Le domaine ne fabrique pas de phrases.** Il renvoie des états et des motifs
  structurés ; l'interface les traduit. Trois langues : français, anglais,
  espagnol, avec des catalogues dont les clés sont vérifiées par les tests.
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

Le projet Xcode est généré : après tout ajout de fichier iOS, régénérer
`ios/Klima.xcodeproj/project.pbxproj` et le revalider.

## Marque

Le signe, le favicon, l'icône d'application et l'image de partage sont rendus
depuis les gabarits de `site/public/` et `/tmp/brand` — un seul dessin, décliné.
La police d'affichage est auto-hébergée : ne jamais la remplacer par un appel à
un service tiers.

## Crédits

Le site mentionne **Maxime Nathan Lestage** comme concepteur et développeur.
