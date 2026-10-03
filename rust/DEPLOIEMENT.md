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

## Ce que la bascule demande, et pourquoi elle n'est pas automatique

Heroku déploie à chaque fusion sur `master`. Le `Procfile` et le buildpack
doivent donc changer **ensemble** : un `Procfile` qui lance un binaire Rust
alors que le buildpack Node est encore en place ne trouverait rien à lancer,
et l'application tomberait en boucle de redémarrage.

Le buildpack se règle dans l'interface d'Heroku, pas dans le dépôt. Les deux
gestes, dans cet ordre :

1. **Heroku → Settings → Buildpacks** : retirer `heroku/nodejs`, ajouter
   `https://github.com/emk/heroku-buildpack-rust`.
2. Fusionner le changement de `Procfile` (une ligne, préparée ci-dessous).

Tant que le premier geste n'est pas fait, le dépôt continue de livrer le
relais Bun, qui fonctionne : rien ne presse.

## Les fichiers, déjà en place

- `rust-toolchain.toml` — la version de Rust et la cible `wasm32-unknown-unknown`.
- `RustConfig` — dit au buildpack de compiler l'espace de travail du dossier
  `rust/`, et de construire les deux interfaces avant le relais.
- `rust/scripts/construire.sh` — télécharge Trunk (binaire publié, pas de
  compilation : une compilation de Trunk sur un dyno dépasserait le temps de
  construction), construit la vitrine et l'application, puis les assemble.

Le `Procfile` à poser, le jour de la bascule :

```
web: rust/target/release/klima-relay
```

## Variables de configuration

| Clé              | Valeur                                                            |
| ---------------- | ----------------------------------------------------------------- |
| `OPEN_METEO_KEY` | La clé du plan commercial, quand il y en aura une                 |
| `KLIMA_ORIGINS`  | Les origines admises — inutile si le site est servi par le relais |
| `KLIMA_PUBLIC`   | Où sont les fichiers du site. `server/public` par défaut          |

`OPEN_METEO_KEY` peut rester vide : le relais tourne alors sur le plan gratuit
d'Open-Meteo, réservé à l'usage non commercial. Le jour où Klima se vend,
cette clé n'est plus facultative — et elle vit ici, jamais dans un binaire
distribué.

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
