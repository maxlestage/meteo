# Envoyer une version sur TestFlight

Le dépôt ne peut pas compiler l'application : il faut un Mac avec Xcode, un
certificat de distribution et un compte développeur. `.github/workflows/testflight.yml`
fait le travail sur un exécuteur macOS de GitHub, sur déclenchement manuel.

**Ce workflow n'a jamais tourné.** Il a été écrit et relu, sa syntaxe est
vérifiée, mais aucune exécution ne l'a confirmé. La première demandera
probablement un ou deux ajustements — c'est la nature de la chose.

## À faire une fois, chez Apple

1. **Compte développeur** (99 $/an) → noter le *Team ID* (dix caractères).
2. **Enregistrer les cinq identifiants** dans Certificates, Identifiers & Profiles :

   ```
   com.kliima.app
   com.kliima.app.widgets
   com.kliima.app.watchkitapp
   com.kliima.app.watchkitapp.complications
   com.kliima.app.tests
   ```

   Sur `com.kliima.app` : activer **App Groups**, **Push Notifications** et
   **Background Modes**. Sur les autres : **App Groups**.
3. **Créer le groupe** `group.com.kliima.app` et le rattacher aux quatre
   premiers identifiants.
4. **Créer l'application** dans App Store Connect sur `com.kliima.app`.
   Le nom du magasin s'y règle séparément du nom affiché sous l'icône — si le
   triangle de « Kliima ‣ » y était refusé, seule la fiche changerait.
5. **Créer l'abonnement** `com.kliima.app.pro.mensuel`, mensuel, au palier
   0,99 €. Sans lui, l'écran d'achat s'affichera sans prix.
6. **Une clé App Store Connect** (Utilisateurs et accès → Intégrations), rôle
   *App Manager*. Le fichier `.p8` ne se télécharge qu'une fois.
7. **Un certificat de distribution Apple**, exporté en `.p12` avec sa clé
   privée et un mot de passe.

## Les six secrets du dépôt

Réglages → Secrets and variables → Actions.

| Secret                 | Contenu                                              |
| ---------------------- | ---------------------------------------------------- |
| `APPLE_TEAM_ID`        | Le Team ID, dix caractères                           |
| `APPLE_CERT_P12`       | Le `.p12` encodé en base64                           |
| `APPLE_CERT_PASSWORD`  | Le mot de passe du `.p12`                            |
| `ASC_KEY_ID`           | L'identifiant de la clé (dix caractères)             |
| `ASC_ISSUER_ID`        | L'identifiant d'émetteur (un UUID)                   |
| `ASC_KEY_P8`           | Le `.p8` encodé en base64                            |

```bash
base64 -i Distribution.p12 | pbcopy
base64 -i AuthKey_XXXXXXXXXX.p8 | pbcopy
```

Le workflow s'arrête à la première étape si l'un manque, plutôt qu'après vingt
minutes de compilation.

## Un préalable : les exécuteurs macOS

Sur ce dépôt, **le workflow ne démarre pas**. Deux déclenchements, l'un sur
`macos-15`, l'autre sur `macos-latest` : échec en sept secondes, sans un seul
journal — l'archive que GitHub renvoie est un zip vide. Un job qui meurt ainsi
n'a exécuté aucune étape ; l'exécuteur n'a jamais été alloué.

Le diagnostic tient par comparaison : sur ce même dépôt, `pages.yml` tourne sur
`ubuntu-latest` et réussit. Actions fonctionne, Linux fonctionne, macOS non. Or
le dépôt est **privé**, et les exécuteurs macOS y sont facturés au décuple des
minutes Linux, sous condition d'une limite de dépense non nulle. Aucun message
d'erreur ne le dit — c'est une inférence, mais toutes les indications
convergent.

Deux façons d'en sortir, aucune ne se règle dans le code :

1. **Relever la limite de dépense** du compte (Réglages → Facturation →
   Spending limit). Le workflow part alors tel quel.
2. **Compiler depuis un Mac**, ci-dessous. Rien à payer, rien à configurer
   côté GitHub.

## Compiler depuis un Mac, à la main

Les mêmes étapes que le workflow, sans exécuteur. Depuis la racine du dépôt,
avec Xcode installé et l'équipe de signature renseignée :

```bash
# 1. Les tests, comme le ferait l'intégration continue
bun install && bun test
python3 ios/Tools/validate_pbxproj.py ios/Kliima.xcodeproj/project.pbxproj

# 2. Les tests iOS — la première fois que SwiftUI, StoreKit et WidgetKit
#    seront compilés pour de bon
xcodebuild test -project ios/Kliima.xcodeproj -scheme Kliima \
  -destination 'platform=iOS Simulator,name=iPhone 16'

# 3. L'archive. Le numéro de build doit croître à chaque envoi.
xcodebuild archive -project ios/Kliima.xcodeproj -scheme Kliima \
  -destination 'generic/platform=iOS' \
  -archivePath build/Kliima.xcarchive \
  -allowProvisioningUpdates \
  DEVELOPMENT_TEAM=VOTRE_TEAM_ID \
  CURRENT_PROJECT_VERSION=1
```

Puis, dans Xcode : **Window → Organizer**, sélectionner l'archive,
*Distribute App* → *TestFlight & App Store*. Xcode se charge de la signature,
de l'export et de l'envoi — c'est le chemin le plus court, et il évite d'avoir
à fabriquer un `ExportOptions.plist` à la main.

## Lancer le workflow

Une fois les exécuteurs macOS disponibles : Actions → TestFlight →
*Run workflow*. Il enchaîne : tests du cœur partagé,
validation du projet, **tests iOS sur simulateur**, archive, export, envoi.

Les tests sur simulateur méritent qu'on s'y arrête : c'est la première fois que
le code SwiftUI, StoreKit et WidgetKit sera compilé pour de bon. Jusqu'ici il
n'a été que vérifié syntaxiquement, et seul le domaine pur tourne sur le banc
d'essai Linux. **C'est là que se révéleront les erreurs qu'aucune vérification
n'a pu voir.**

Le numéro de build vient du numéro d'exécution : il croît tout seul, comme App
Store Connect l'exige.

## Ce qui a déjà été réglé dans le dépôt

- **Conformité au chiffrement.** `ITSAppUsesNonExemptEncryption` vaut `false`
  dans les deux applications : Kliima ne chiffre rien lui-même, il appelle des
  services en HTTPS. Sans cette clé, chaque build reste bloqué en *Missing
  Compliance* jusqu'à ce qu'on réponde à la question — à chaque envoi.
- **Icônes sans couche alpha.** Vérifié : les deux PNG 1024 sont en RVB. Une
  couche alpha est un motif de rejet automatique.
- **Équipe de signature.** Le réglage existe, vide : Xcode la demande en local,
  le workflow la passe en ligne de commande.

## Ce qui reste hors de portée d'ici

Le relais (`server/`) n'est pas hébergé : sans lui, l'application retombe sur
les fournisseurs qu'elle peut appeler seule. Pour une première version de test,
c'est suffisant — mais le recoupement complet et la clé commerciale
d'Open-Meteo attendent un hébergement.
