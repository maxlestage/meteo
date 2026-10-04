# Mettre le relais Rust en ligne

Le relais est un seul binaire : il écoute, interroge les fournisseurs une fois
pour tout le monde, rend la réponse, et sait aussi servir la vitrine et
l'application depuis la même origine. Pas de base de données, rien à part un
endroit qui reste allumé.

| Chemin    | Ce qu'on y trouve                       |
| --------- | --------------------------------------- |
| `/`       | La vitrine, **si** `KLIMA_PUBLIC` pointe dessus |
| `/app/`   | L'application, à la même condition      |
| `/health` | L'état du relais                        |
| `/v1/…`   | L'API : les fournisseurs, vus du relais |

L'API garde la priorité : elle est tout entière sous `/v1/`, et le partage est
vérifié par des tests.

**Sur Heroku, le relais sert tout** : la vitrine à `/`, l'application à
`/app/`, l'API sous `/v1/` — une seule adresse, et l'application appelle son
relais sur la même origine, sans réglage de partage.

Le buildpack Rust ne sait faire qu'une chose, `cargo build --release`, et n'a
aucun crochet pour lancer autre chose. C'est donc la compilation du relais qui
construit le site : `rust/klima-relay/build.rs` appelle `construire.sh` — le
même script qu'en local et que pour GitHub Pages. Il ne s'éveille que chez
Heroku (`STACK=heroku-…`, que la plateforme pose pendant la construction) ou
sur demande (`KLIMA_CONSTRUIRE_SITE=1`) ; en développement, en CI et sous
`cargo test`, il ne fait rien. Le cargo imbriqué reçoit son propre dossier de
compilation, voisin de celui du buildpack — sans quoi il attendrait pour
toujours le verrou que tient son parent — et ce dossier vit dans le cache, comme
Trunk, téléchargé une fois. Si le site ne se construit pas, le déploiement
échoue et Heroku garde la version en service : mieux vaut cela que remplacer
une version qui servait le site par une qui ne le sert plus.

Répété hors d'Heroku, dans un dépôt copié à neuf, sans Trunk préinstallé :
1 min 54 s pour le relais et les deux interfaces, puis la commande exacte du
`Procfile` sert `/`, `/app/`, `/health` et `/v1/plan`.

## Le geste qui reste, et il est dans l'interface d'Heroku

**Heroku → Settings → Buildpacks** : retirer `heroku/nodejs`, ajouter
`https://github.com/emk/heroku-buildpack-rust`.

Le buildpack se règle là et pas dans le dépôt. Tant qu'il n'est pas changé,
les constructions échouent — le buildpack Node ne trouve plus ni `package.json`
ni rien à installer. **Un échec de construction ne remplace pas la version en
service** — mais cela ne protège que ce qui tournait déjà : si la version en
service est elle-même un `Procfile` qui ne trouve pas son binaire, le dyno
démarre, meurt, et le routeur répond 503 sur tous les chemins. C'est la
différence entre « rien ne se déploie » et « rien ne sert », et seule la
seconde se voit du dehors.

## Les fichiers, déjà en place

- `rust-toolchain.toml` — la version de Rust et la cible `wasm32-unknown-unknown`.
- `RustConfig` — dit au buildpack de compiler l'espace de travail du dossier
  `rust/`, et sur quelle version. **Attention** : ce fichier est sourcé par le
  `bin/compile` du buildpack, qui ne lit que les variables qu'il déclare
  lui-même. Une clé inventée ne fait rien et ne le dit pas ; une valeur non
  numérique dans `RUST_SKIP_BUILD` fait *sauter* la compilation, en silence et
  avec un code de sortie nul. `rust/klima-relay/tests/deploiement.rs` vérifie
  donc les clés, la version, et que le `Procfile` vise bien là où le buildpack
  pose le binaire.
- `rust/scripts/construire.sh` — télécharge Trunk (binaire publié, pas de
  compilation : une compilation de Trunk sur un dyno dépasserait le temps de
  construction), construit la vitrine et l'application, puis les assemble. Il
  sert en local, à GitHub Pages, et chez Heroku par `klima-relay/build.rs`.

Le `Procfile` est déjà posé :

```
web: rust/target/release/klima-relay
```

## Variables de configuration

| Clé              | Valeur                                                            |
| ---------------- | ----------------------------------------------------------------- |
| `OPEN_METEO_KEY` | La clé du plan commercial, quand il y en aura une                 |
| `KLIMA_ORIGINS`  | Les origines admises — inutile si le site est servi par le relais |
| `KLIMA_PUBLIC`   | Où sont les fichiers du site. `public` par défaut — rien à régler |
| `KLIMA_PRO`      | Le palier accordé pendant l'essai — voir plus bas                 |
| `KLIMA_SESSION_SECRET` | Le secret qui signe les sessions des comptes — voir plus bas |

`OPEN_METEO_KEY` peut rester vide : le relais tourne alors sur le plan gratuit
d'Open-Meteo, réservé à l'usage non commercial. Le jour où Klima se vend,
cette clé n'est plus facultative — et elle vit ici, jamais dans un binaire
distribué.

## Débloquer le palier payant pendant l'essai

Pendant l'essai, personne n'achète : les testeurs doivent voir le palier payant
sans passer par la boutique. `KLIMA_PRO` le décide, **côté serveur** :

| Valeur                    | Ce que le relais répond                      |
| ------------------------- | -------------------------------------------- |
| absente, vide, `0`, `non` | rien : StoreKit décide seul                  |
| `tous`, `1`, `oui`        | le palier payant, à qui demande              |
| une valeur contenant `@`  | le palier payant, aux adresses énumérées     |
| n'importe quoi d'autre    | le palier payant, à qui présente ce code     |

C'est là que ça doit vivre, et pas ailleurs : une valeur glissée dans une
application distribuée est une valeur publiée, qu'on ne retire qu'en publiant
une nouvelle version. Celle-là s'enlève en une commande.

### Par comptes — la forme du système

Chaque testeur a son compte, et rien à taper : sur iPhone, l'écran
d'abonnement propose **Se connecter avec Apple**. Apple prouve l'adresse, le
relais la compare à `KLIMA_PRO`. Deux variables, une fois pour toutes :

```bash
heroku config:set KLIMA_SESSION_SECRET="$(openssl rand -hex 32)" -a VOTRE-APP
heroku config:set KLIMA_PRO='max@ferme.fr, ana@vina.es' -a VOTRE-APP
curl https://VOTRE-APP.herokuapp.com/health
# … "pro":"sur liste (2)","comptes":"activés"
```

Ce qui se passe, dans l'ordre :

1. Le testeur touche le bouton d'Apple. Apple remet à l'application un jeton
   signé qui dit : cette personne contrôle cette adresse.
2. L'application l'envoie à `POST /v1/session`. Le relais vérifie la signature
   avec les clés publiques d'Apple, l'émetteur, le destinataire
   (`com.kliima.app`) et l'échéance, puis rend une **session** qu'il signe
   lui-même avec `KLIMA_SESSION_SECRET`. Elle vaut six mois.
3. L'application la garde dans le trousseau et la présente à chaque
   `GET /v1/plan`, dans l'en-tête `Authorization`. Le relais en relit
   l'adresse et la compare à la liste **du moment**.

La session prouve une identité ; elle n'accorde rien. D'où les deux gestes :

| Pour…                              | Faire                                                     |
| ---------------------------------- | --------------------------------------------------------- |
| inviter quelqu'un                  | ajouter son adresse à `KLIMA_PRO`                         |
| retirer quelqu'un                  | l'enlever de `KLIMA_PRO` — effet à la question suivante   |
| déconnecter tout le monde          | changer `KLIMA_SESSION_SECRET` — chacun se reconnecte d'une tape |

**L'adresse relais d'Apple.** Qui choisit « Masquer mon adresse » reçoit une
adresse en `@privaterelay.appleid.com`, que personne ne devinerait. L'écran
d'abonnement l'affiche telle qu'Apple l'a prouvée, avec « ce compte n'est pas
encore invité » : c'est celle-là qu'il faut ajouter à la liste. Montrer sa
propre adresse à quelqu'un qui vient de la prouver ne révèle rien de la liste.

**Sans secret, pas de comptes.** Le relais le dit — `"comptes":"désactivés"`
dans `/health`, 503 sur `/v1/session` — plutôt que de tirer un secret au hasard
à chaque démarrage : Heroku recycle ses dynos chaque jour, et tout le monde
serait déconnecté chaque matin. Le secret doit faire 32 octets au moins ;
`openssl rand -hex 32` en donne 64 caractères. Il vit ici et nulle part
ailleurs, comme la clé d'Open-Meteo.

**Côté Apple**, la capacité « Sign in with Apple » doit être active sur
l'identifiant `com.kliima.app`. L'archivage de la CI l'active seul
(`-allowProvisioningUpdates`) ; si Xcode réclame tout de même un profil qui la
porte, la case est dans Certificates, Identifiers & Profiles › Identifiers ›
`com.kliima.app`.

**Ce que le relais vérifie, et que les tests fixent** : un jeton expiré, émis
pour une autre application, par un autre émetteur, à clé inconnue, retouché, ou
annonçant HS256 pour se faire vérifier avec la clé publique — tous refusés, sans
que la réponse dise lequel. Le motif va au journal. Les clés d'Apple sont lues
une fois par heure ; une clé inconnue relance la lecture au plus une fois toutes
les cinq minutes, pour que personne ne fasse marteler Apple par le relais.

### Par liste d'adresses, sans compte — ce que fait encore le site

```bash
heroku config:set KLIMA_PRO='max@ferme.fr, ana@vina.es' -a VOTRE-APP
curl 'https://VOTRE-APP.herokuapp.com/v1/plan?courriel=max@ferme.fr'
# {"plan":"pro"}
curl 'https://VOTRE-APP.herokuapp.com/v1/plan?courriel=jo@farm.uk'
# {"plan":"libre"}
```

Virgule, point-virgule, espace et retour à la ligne séparent indifféremment —
une liste se colle depuis un courriel, un tableur ou une note, et chacun la
sépare à sa façon. La casse et les blancs autour ne comptent pas. Une entrée
sans arobase est traitée comme une faute de frappe et tombe : la garder
ouvrirait le palier à qui taperait ce mot-là. `/health` dit alors `sur liste
(2)` — le compte, jamais les adresses — et c'est ce qui rend une virgule
oubliée visible du dehors.

C'est la seule forme qui n'oblige à **rien embarquer** : le testeur saisit son
adresse dans l'écran d'abonnement, elle ne voyage pas dans le binaire, et on
retire quelqu'un de l'essai sans toucher aux autres ni republier.

**Ce qu'une liste n'est pas** : une preuve d'identité. Personne ne vérifie que
celui qui présente une adresse la relève. Quiconque connaît une adresse invitée
obtient le palier. Pour un essai fermé c'est le bon compromis — plus étroit que
`tous`, plus révocable qu'un code partagé. Vendre un abonnement reste le
travail de StoreKit, et il reste entier.

**Une adresse à étiquette doit être encodée** dans l'URL : une query se lit en
form-urlencoded, où `+` vaut une espace. `max+ferme@ferme.fr` passé tel quel
arrive avec un trou au milieu. L'application encode ; à la main, écrivez
`?courriel=max%2Bferme@ferme.fr`.

### Par `tous` — le plus simple, et le plus large

```bash
heroku config:set KLIMA_PRO=tous -a VOTRE-APP
curl https://VOTRE-APP.herokuapp.com/v1/plan
# {"plan":"pro"}

heroku config:unset KLIMA_PRO -a VOTRE-APP   # et le palier redescend
```

**Soyez franc sur ce que fait `tous`** : un relais public qui accorde le palier
à qui demande l'accorde à tout le monde. C'est ce qu'on veut tant que le relais
n'est connu que des testeurs qu'on a invités.

C'est aussi la forme qui ne demande **rien à saisir** : les deux interfaces
interrogent `/v1/plan` au lancement, sans rien présenter, et le palier s'ouvre
seul. La liste d'adresses, elle, coûte une saisie par appareil — une seule, elle
est mémorisée — et c'est ce qu'on paie pour que le site public n'ouvre pas le
registre à qui passe.

### Par code partagé

Un code long et aléatoire marche aussi — le relais ne le répète jamais, ni dans
sa réponse ni dans `/health`, et la comparaison est à durée constante pour
qu'on ne le devine pas lettre par lettre en chronométrant. Mais il doit arriver
jusqu'au téléphone, donc voyager dans le binaire, donc être publié. La liste
d'adresses n'a pas ce défaut : préférez-la.

Côté application, deux clés d'`Info.plist` : `KliimaRelay` (l'adresse du relais
— sans elle, aucun appel n'est fait, et l'écran d'abonnement n'affiche même pas
le bloc du compte) et `KliimaProCode`, qui ne sert que si le relais exige un
code. L'accord **s'ajoute** à ce que dit StoreKit : un relais muet ne fait pas
perdre un abonnement réel, et une boutique vide n'annule pas l'accord du relais.

**Le site n'a pas de comptes.** « Se connecter avec Apple » sur le web demande
un identifiant de services et un domaine déclarés chez Apple ; en attendant, le
site garde le champ d'adresse, et `?courriel=` reste accepté par le relais. Ce
chemin-là n'est pas une preuve : quiconque connaît une adresse invitée obtient
le palier *sur le site* — c'est-à-dire le registre, un CSV de météo publique.
Il ne donne rien sur iPhone, qui ne présente plus que sa session.

`KliimaRelay` est renseigné : c'est une adresse publique, elle n'a rien à cacher.
`KliimaProCode` reste vide dans le dépôt et le restera, et un test le vérifie.
Un code écrit ici serait lisible par quiconque lit le dépôt, bien avant d'être
extrait du binaire. Les comptes rendent la question sans objet.

## Vérifier

```bash
curl https://VOTRE-APP.herokuapp.com/health
# {"statut":"ok","cellules":0,"interrogations":0,"cleOpenMeteo":"absente"}
```

Puis `/` pour la vitrine et `/app/` pour l'application. Rien à configurer pour
que l'application trouve le relais : elle est construite avec
`KLIMA_RELAY=meme-origine` et le demande à l'hôte qui la sert — une adresse qui
n'est connue qu'à l'affichage, puisqu'elle dépend du nom de domaine.

## Ce qu'un dyno change au cache

Le cache vit en mémoire. C'est ce qui fait tenir la promesse des vingt-quatre
appels par jour et par cellule — mais elle suppose un processus qui reste
debout.

- Un dyno **Eco** s'endort après trente minutes sans visite. Au réveil, le
  cache est vide et la première requête paie l'attente.
- Heroku redémarre les dynos une fois par jour de toute façon.

Contrairement au relais Bun, celui-ci balaie son cache toutes les heures : les
entrées que même le mode dépannage ne servirait plus sont oubliées, et la
mémoire d'un processus qui tourne des semaines ne monte plus indéfiniment.
