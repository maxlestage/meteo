# Outils du projet Xcode

Trois scripts Python sans dépendances. Ils produisent des fichiers **versionnés**
que Xcode sait ouvrir et modifier directement : le générateur est une commodité,
pas un passage obligé. On peut très bien éditer le `project.pbxproj` à la main —
mais ajouter un fichier veut alors dire retoucher une demi-douzaine de sections
dans soixante kilo-octets de plist, avec des identifiants à inventer soi-même.

## Ajouter un fichier iOS

1. Créer le fichier sous `ios/Kliima/…`.
2. L'ajouter à la liste qui convient en tête de `gen_pbxproj.py`
   (`MODELS`, `SERVICES`, `VIEWS`, `TEST_FILES`…).
3. Régénérer, puis revalider :

```bash
python3 ios/Tools/gen_pbxproj.py ios/Kliima.xcodeproj/project.pbxproj
python3 ios/Tools/validate_pbxproj.py ios/Kliima.xcodeproj/project.pbxproj
```

Les identifiants du plist sont dérivés d'un hachage de leur clé : deux
générations successives donnent le même fichier, et le diff ne montre que ce
qui a réellement changé.

## Ajouter un texte traduit

Les catalogues `.xcstrings` sont du JSON que personne ne relit avec plaisir. La
table lisible est en tête de `gen_xcstrings.py`, une ligne par clé :
`"clé": (français, anglais, espagnol)`.

```bash
python3 ios/Tools/gen_xcstrings.py ios/Kliima/Resources
```

Les trous des motifs sont **positionnels** (`%1$@`), comme `String(format:)`
l'attend — pas `{nom}` comme du côté TypeScript. Un test le verrouille.

## Ce que le validateur vérifie

Il analyse le plist avec son propre parseur OpenStep, puis contrôle que chaque
référence pointe sur un objet existant, que chaque fichier référencé est sur le
disque, que chaque cible a son type de produit, son identifiant de paquet et ses
sources, et que les phases d'intégration et les dépendances entre cibles sont
cohérentes. Il sort en erreur si quelque chose cloche — utilisable tel quel dans
une vérification automatique.
