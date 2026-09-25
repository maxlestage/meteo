# Envoyer une version sur TestFlight

Le dépôt ne peut pas compiler l'application : il faut un Mac avec Xcode et un
compte développeur. `.github/workflows/testflight.yml` fait le travail sur un
exécuteur macOS de GitHub, sur déclenchement manuel.

**Où en est-on.** Le workflow tourne. Les exécutions nº 8 et nº 9 ont compilé
les cinq cibles et passé les 126 tests sur simulateur (Xcode 26.6, environ cinq
minutes). Il bute maintenant sur l'archive, pour une raison qui n'est pas dans
le code — voir « Le groupe d'applications », plus bas.

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
   premiers identifiants. Cette étape ne s'automatise pas ; elle a sa propre
   section plus bas, parce qu'elle est la seule à arrêter le workflow.
4. **Créer l'application** dans App Store Connect sur `com.kliima.app`.
   Le nom du magasin s'y règle séparément du nom affiché sous l'icône — si le
   triangle de « Kliima ‣ » y était refusé, seule la fiche changerait.
5. **Créer l'abonnement** `com.kliima.app.pro.mensuel`, mensuel, au palier
   0,99 €. Sans lui, l'écran d'achat s'affichera sans prix.
6. **Une clé App Store Connect** (Utilisateurs et accès → Intégrations), rôle
   *App Manager*. Le fichier `.p8` ne se télécharge qu'une fois.

## Quatre secrets, pas six

Réglages → Secrets and variables → Actions.

| Secret            | Contenu                                   |
| ----------------- | ----------------------------------------- |
| `APPLE_TEAM_ID`   | Le Team ID, dix caractères                |
| `ASC_KEY_ID`      | L'identifiant de la clé (dix caractères)  |
| `ASC_ISSUER_ID`   | L'identifiant d'émetteur (un UUID)        |
| `ASC_KEY_P8`      | Le fichier `.p8`, tel quel ou en base64   |

Le `.p8` s'accepte sous ses deux formes : collez le contenu du fichier
directement, ou son encodage — `base64 -i AuthKey_XXXXXXXXXX.p8 | pbcopy`. Le
workflow vérifie qu'il y lit bien une clé PEM et le dit tout de suite si ce
n'est pas le cas.

**Pas besoin de certificat de distribution.** L'exécution nº 8 l'a établi :
avec `-allowProvisioningUpdates` et la clé App Store Connect, Xcode obtient de
quoi signer sans qu'on lui fournisse de `.p12`. Vous pouvez tout de même en
poser un — `APPLE_CERT_P12` et `APPLE_CERT_PASSWORD` restent reconnus — mais
ça n'apporte rien et demande un Mac.

**Le Team ID, en revanche, est indispensable.** Sans lui, les quatre cibles
échouent sur « Signing requires a development team » : la clé authentifie,
elle ne désigne pas l'équipe.

Le workflow s'arrête à la première étape si l'un des quatre manque, plutôt
qu'après cinq minutes de compilation.

## Le groupe d'applications, à la main et seulement à la main

C'est le point de blocage actuel. L'exécution nº 9 échoue à l'archive, sur les
quatre cibles :

```
Provisioning profile "iOS Team Provisioning Profile: com.kliima.app" doesn't
match the entitlements file's value for the com.apple.security.application-groups
entitlement.
```

Ce n'est pas un défaut du projet : les quatre fichiers `.entitlements` déclarent
le même groupe, `group.com.kliima.app`, et rien d'autre. Le manque est chez
Apple.

La raison tient en une phrase : **l'API App Store Connect n'expose pas les
groupes d'applications.** Avec `-allowProvisioningUpdates`, Xcode sait créer les
identifiants d'application et y activer la capacité *App Groups* ; il ne sait
pas rattacher *tel* groupe à *tel* identifiant, faute d'une ressource
`/v1/appGroups` à appeler. Le profil qu'il régénère ne porte donc aucun groupe,
et l'écart avec le fichier d'habilitations arrête la signature.

### En attendant : archiver sans le groupe

Le workflow n'attend plus. *Run workflow* laisse la case **« Le groupe
group.com.kliima.app est enregistré »** décochée par défaut, et archive alors
en passant `CODE_SIGN_ENTITLEMENTS=` : plus d'habilitations, donc plus d'écart
avec le profil, donc une archive qui part.

Ce que ça coûte, et il faut le savoir avant d'installer la version : le widget
d'écran d'accueil et la montre ne liront pas la parcelle choisie dans
l'application. `SharedStore` retombe sur les réglages locaux — c'est prévu, ça
ne plante pas — et ils affichent donc la parcelle par défaut. Tout le reste de
l'application fonctionne.

Le jour où le groupe existe, cocher la case au lancement : les habilitations
reprennent leur place et le partage revient, sans rien changer au dépôt.

### Le rattachement, lui, se fait chez Apple

Une seule fois, dans
[Certificates, Identifiers & Profiles](https://developer.apple.com/account/resources/identifiers/list/applicationGroup) :

1. **Identifiers → App Groups → +** : enregistrer `group.com.kliima.app`.
2. Pour chacun des quatre identifiants — `com.kliima.app`,
   `com.kliima.app.widgets`, `com.kliima.app.watchkitapp`,
   `com.kliima.app.watchkitapp.complications` — ouvrir la fiche, cocher
   **App Groups**, *Configure*, et sélectionner le groupe.

Les identifiants existent déjà : Xcode les a créés à la première archive. Les
modifier invalide les profils en cours, mais c'est sans conséquence ici — le
workflow en régénère à chaque exécution.

## Ce que le dépôt ne peut pas faire à votre place

Deux choses, et seulement deux. Ni l'une ni l'autre n'a d'API : l'App Store
Connect API lit les applications, elle ne les crée pas, et elle ignore les
groupes d'applications.

1. **La fiche de l'application**, dans App Store Connect → Mes apps → **+**,
   sur `com.kliima.app`. Sans elle, l'envoi est refusé : TestFlight n'accepte
   pas un paquet dont la fiche n'existe pas.
2. **Le groupe d'applications**, ci-dessus — et lui peut attendre, puisque
   l'archive part sans.

Et une troisième, plus tard : l'abonnement `com.kliima.app.pro.mensuel` au
palier 0,99 €, sans quoi l'écran d'achat s'affiche sans prix. Une version de
test s'installe très bien sans lui.

## Savoir où on en est sans rien compiler

Actions → **État Apple** → *Run workflow*. Vingt secondes, sur un exécuteur
Linux, et le résumé dit ce que le compte contient : la fiche de l'application,
chacun des cinq identifiants et ses capacités, l'abonnement. Il ne modifie
rien — que des lectures.

C'est la première chose à lancer quand une signature échoue : le message
d'Apple nomme rarement la cause, cette page si.

## Compiler depuis un Mac, à la main

Les mêmes étapes que le workflow, si vous préférez garder la main. Depuis la
racine du dépôt, avec Xcode installé et l'équipe de signature renseignée :

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

Actions → TestFlight → *Run workflow*. Il enchaîne : tests du cœur partagé,
validation du projet, **tests iOS sur simulateur**, archive, export, envoi.

Les tests sur simulateur ont déjà servi : c'est là que le code SwiftUI,
StoreKit et WidgetKit a été compilé pour de bon, après n'avoir été que vérifié
syntaxiquement et éprouvé, pour le seul domaine pur, sur un banc d'essai Linux.
**126 tests, aucun échec** — la traduction depuis le cœur partagé tient.

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
