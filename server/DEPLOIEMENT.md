# Mettre le relais en ligne

Le relais est un seul processus : il écoute, interroge les fournisseurs une
fois pour tout le monde, et rend la réponse. Il n'a pas de base de données. Ce
qu'il lui faut, c'est un endroit qui reste allumé.

Il sert aussi le site. Puisqu'il tourne déjà sur une adresse publique, lui
faire rendre les fichiers construits évite un second hébergement — et met
l'application sur la même origine que ses appels, donc aucune origine à
autoriser et aucune adresse de relais à configurer.

| Chemin       | Ce qu'on y trouve                        |
| ------------ | ---------------------------------------- |
| `/`          | La vitrine                               |
| `/app/`      | L'application                            |
| `/health`    | L'état du relais                         |
| `/v1/…`      | L'API : les fournisseurs, vus du relais  |

L'API garde la priorité : elle est toute entière sous `/v1/`, et le partage est
vérifié par des tests. Un chemin qui ne désigne aucun fichier répond 404 — pas
une erreur de coordonnées.

## Pourquoi le premier essai sur Heroku a échoué

```
npm error code EUNSUPPORTEDPROTOCOL
npm error Unsupported URL Type "workspace:": workspace:*
```

Deux manques, tous deux dans le dépôt :

1. **`workspace:*` n'est pas de l'npm.** C'est la notation de Bun et de pnpm.
   npm gère les espaces de travail, mais il veut une plage de versions
   ordinaire ; il relie ensuite le paquet local parce que son nom figure dans
   `workspaces`. Les trois dépendances disent donc `"*"` désormais — Bun
   résout pareil, par le nom.
2. **Aucun `Procfile`.** Même installé, rien n'aurait dit quoi lancer.

## Comment le relais trouve Bun sur un dyno

Il tourne sur Bun — `Bun.serve`, et du TypeScript exécuté sans compilation. Le
buildpack Node de Heroku n'installe que Node.

Plutôt qu'un buildpack tiers à ajouter dans les réglages, Bun est déclaré comme
une dépendance ordinaire : il est publié sur npm, et le buildpack l'installe
donc lui-même, avec le reste. Le `Procfile` l'appelle là où npm le pose :

```
web: node_modules/.bin/bun run server/src/index.ts
```

Rien à configurer côté Heroku. Le prix de ce choix est un binaire d'une
trentaine de mégaoctets installé à chaque `install`, y compris en local où Bun
est déjà là. C'est le coût d'un déploiement qui part d'une simple poussée.

`heroku-postbuild` construit la vitrine et l'application, et les pose dans
`server/public` — la même disposition que sur GitHub Pages. Il refuse
bruyamment si Vite manque plutôt que de livrer un site vide.

Vite manquerait, d'ailleurs, sans le `.npmrc` de la racine : Heroku construit
avec `NODE_ENV=production`, et npm saute alors les dépendances de
développement. `include=dev` les rétablit ; elles sont élaguées après la
construction, donc ce qui tourne ne grossit pas.

## Les étapes

1. Créer l'application sur Heroku, puis la relier au dépôt GitHub
   (*Deploy* → *GitHub* → *Enable Automatic Deploys*), ou pousser à la main.
2. **Variables de configuration** (*Settings* → *Config Vars*) :

   | Clé               | Valeur                                              |
   | ----------------- | --------------------------------------------------- |
   | `OPEN_METEO_KEY`  | La clé du plan commercial, quand il y en aura une    |
   | `KLIMA_ORIGINS`   | Les origines admises — inutile si le site est servi par le relais |

   `OPEN_METEO_KEY` peut rester vide pour l'instant : le relais tourne alors
   sur le plan gratuit d'Open-Meteo, réservé à l'usage non commercial. Le jour
   où Klima se vend, cette clé n'est plus facultative — et elle vit ici, jamais
   dans un binaire distribué.
3. Vérifier :

   ```bash
   curl https://VOTRE-APP.herokuapp.com/health
   # {"statut":"ok","cellules":0,"interrogations":0,"cleOpenMeteo":"absente"}
   ```

4. Ouvrir `https://VOTRE-APP.herokuapp.com/` : la vitrine. Et `/app/` :
   l'application, qui passe déjà par le relais.

   Rien à configurer pour ça. La construction pose `VITE_KLIMA_RELAY` à
   `meme-origine`, et l'application demande alors son relais à l'hôte qui la
   sert — une adresse qui n'est connue qu'à l'affichage, puisqu'elle dépend du
   nom de domaine.

5. **Seulement si vous publiez aussi sur GitHub Pages** : là, le site et le
   relais ne sont pas sur la même origine. Il faut alors poser l'adresse du
   relais dans GitHub → *Settings* → *Secrets and variables* → *Actions* →
   *Variables* → `KLIMA_RELAY`, et ajouter `https://maxlestage.github.io` aux
   `KLIMA_ORIGINS` du relais.

## Ce qu'un dyno change au cache

Le cache vit en mémoire. C'est ce qui fait tenir la promesse des 24 appels par
jour et par cellule — mais elle suppose un processus qui reste debout.

- Un dyno **Eco** s'endort après trente minutes sans visite. Au réveil, le
  cache est vide et la première requête paie l'attente.
- Heroku redémarre les dynos une fois par jour de toute façon. Le cache
  repart de zéro à chaque fois ; ce n'est pas grave, c'est juste à savoir.

Pour un relais qui sert vraiment, un dyno **Basic** qui ne dort pas vaut mieux
que la mise en cache la plus fine.
