# Mettre le relais en ligne

Le relais est un seul processus : il écoute, interroge les fournisseurs une
fois pour tout le monde, et rend la réponse. Il n'a pas de base de données. Ce
qu'il lui faut, c'est un endroit qui reste allumé.

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

`heroku-postbuild` ne fait rien, volontairement : sans lui, Heroku lancerait le
`build` de la racine, qui construit aussi la vitrine et l'application web. Le
relais, lui, s'exécute depuis ses sources.

## Les étapes

1. Créer l'application sur Heroku, puis la relier au dépôt GitHub
   (*Deploy* → *GitHub* → *Enable Automatic Deploys*), ou pousser à la main.
2. **Variables de configuration** (*Settings* → *Config Vars*) :

   | Clé               | Valeur                                              |
   | ----------------- | --------------------------------------------------- |
   | `KLIMA_ORIGINS`   | `https://maxlestage.github.io` — les origines admises |
   | `OPEN_METEO_KEY`  | La clé du plan commercial, quand il y en aura une    |

   `OPEN_METEO_KEY` peut rester vide pour l'instant : le relais tourne alors
   sur le plan gratuit d'Open-Meteo, réservé à l'usage non commercial. Le jour
   où Klima se vend, cette clé n'est plus facultative — et elle vit ici, jamais
   dans un binaire distribué.
3. Vérifier :

   ```bash
   curl https://VOTRE-APP.herokuapp.com/health
   # {"statut":"ok","cellules":0,"interrogations":0,"cleOpenMeteo":"absente"}
   ```

4. **Faire passer le web par le relais.** Le site et l'application lisent
   `VITE_KLIMA_RELAY` à la construction. Poser l'adresse dans GitHub →
   *Settings* → *Secrets and variables* → *Actions* → *Variables* → `KLIMA_RELAY`,
   puis relancer la publication. Sans cette variable, les deux appellent les
   fournisseurs en direct, comme aujourd'hui.

## Ce qu'un dyno change au cache

Le cache vit en mémoire. C'est ce qui fait tenir la promesse des 24 appels par
jour et par cellule — mais elle suppose un processus qui reste debout.

- Un dyno **Eco** s'endort après trente minutes sans visite. Au réveil, le
  cache est vide et la première requête paie l'attente.
- Heroku redémarre les dynos une fois par jour de toute façon. Le cache
  repart de zéro à chaque fois ; ce n'est pas grave, c'est juste à savoir.

Pour un relais qui sert vraiment, un dyno **Basic** qui ne dort pas vaut mieux
que la mise en cache la plus fine.
