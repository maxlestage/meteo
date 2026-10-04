# Mettre le relais Rust en ligne

Le relais est un seul binaire : il écoute, interroge les fournisseurs une fois
pour tout le monde, rend la réponse, et sert la vitrine et l'application
depuis la même origine. Pas de base de données, rien à part un endroit qui
reste allumé.

| Chemin    | Ce qu'on y trouve                       |
| --------- | --------------------------------------- |
| `/`       | La vitrine                              |
| `/app/`   | L'application                           |
| `/health` | L'état du relais                        |
| `/v1/…`   | L'API : les fournisseurs, vus du relais |

L'API garde la priorité : elle est tout entière sous `/v1/`, et le partage est
vérifié par des tests.

## Le geste qui reste, et il est dans l'interface d'Heroku

**Heroku → Settings → Buildpacks** : retirer `heroku/nodejs`, ajouter
`https://github.com/emk/heroku-buildpack-rust`.

Le buildpack se règle là et pas dans le dépôt. Tant qu'il n'est pas changé,
les constructions échouent — le buildpack Node ne trouve plus ni `package.json`
ni rien à installer. **Un échec de construction ne remplace pas la version en
service** : ce qui tourne continue de tourner, simplement plus rien ne se
déploie jusqu'à ce que le buildpack soit le bon.

## Les fichiers, déjà en place

- `rust-toolchain.toml` — la version de Rust et la cible `wasm32-unknown-unknown`.
- `RustConfig` — dit au buildpack de compiler l'espace de travail du dossier
  `rust/`, et de construire les deux interfaces avant le relais.
- `rust/scripts/construire.sh` — télécharge Trunk (binaire publié, pas de
  compilation : une compilation de Trunk sur un dyno dépasserait le temps de
  construction), construit la vitrine et l'application, puis les assemble.

Le `Procfile` est déjà posé :

```
web: rust/target/release/klima-relay
```

## Variables de configuration

| Clé              | Valeur                                                            |
| ---------------- | ----------------------------------------------------------------- |
| `OPEN_METEO_KEY` | La clé du plan commercial, quand il y en aura une                 |
| `KLIMA_ORIGINS`  | Les origines admises — inutile si le site est servi par le relais |
| `KLIMA_PUBLIC`   | Où sont les fichiers du site. `public` par défaut                 |
| `KLIMA_PRO`      | Le palier accordé pendant l'essai — voir plus bas                 |

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

### Par liste d'adresses — la forme à préférer

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

### Par code partagé

Un code long et aléatoire marche aussi — le relais ne le répète jamais, ni dans
sa réponse ni dans `/health`, et la comparaison est à durée constante pour
qu'on ne le devine pas lettre par lettre en chronométrant. Mais il doit arriver
jusqu'au téléphone, donc voyager dans le binaire, donc être publié. La liste
d'adresses n'a pas ce défaut : préférez-la.

Côté application, deux clés d'`Info.plist` : `KliimaRelay` (l'adresse du relais
— sans elle, aucun appel n'est fait, et l'écran d'abonnement n'affiche même pas
le champ d'accès de test) et `KliimaProCode`, qui ne sert que si le relais exige
un code. L'adresse, elle, n'est pas dans `Info.plist` : elle est saisie par le
testeur et rangée avec les réglages. L'accord **s'ajoute**
à ce que dit StoreKit : un relais muet ne fait pas perdre un abonnement réel, et
une boutique vide n'annule pas l'accord du relais.

Le palier payant n'existe que sur iPhone : l'application web est au palier libre
en dur et n'interroge pas `/v1/plan`. Une invitation ne concerne donc que le
téléphone, et `tous` n'ouvre rien de payant sur le site — il n'y a rien à y
ouvrir.

`KliimaRelay` est renseigné : c'est une adresse publique, elle n'a rien à cacher.
`KliimaProCode` reste vide dans le dépôt et le restera. Un code écrit ici serait
lisible par quiconque lit le dépôt, bien avant d'être extrait du binaire — ce qui
ne laisserait plus qu'à le changer des deux côtés. Si une version d'essai doit
présenter un code, il se pose dans Xcode au moment de l'archivage, sur une copie
locale du fichier, et il ne revient pas dans un commit. Tant que `KLIMA_PRO` vaut
`tous`, la question ne se pose pas : le relais accorde le palier sans qu'on lui
présente quoi que ce soit, et l'application n'a aucun code à porter.

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
