#!/usr/bin/env bash
#
# Construire la vitrine et l'application, et les poser là où le relais les
# servira.
#
# Le relais tourne déjà sur une adresse publique : lui faire servir les
# fichiers construits évite un second hébergement, et met l'application sur la
# même origine que ses appels — donc aucune origine à autoriser, et aucune
# adresse de relais à configurer.
set -euo pipefail

bun=node_modules/.bin/bun
if [ ! -x "$bun" ]; then
  echo "Bun est absent de node_modules : il est déclaré en dépendance de la" >&2
  echo "racine précisément pour être là. Vérifier l'installation." >&2
  exit 1
fi

if [ ! -x node_modules/.bin/vite ]; then
  echo "Vite est absent : les dépendances de développement n'ont pas été" >&2
  echo "installées. C'est ce que « include=dev » du .npmrc doit garantir," >&2
  echo "même avec NODE_ENV=production." >&2
  exit 1
fi

# L'application lit cette valeur à la construction et demande alors son relais
# à l'origine qui la sert.
export VITE_KLIMA_RELAY=meme-origine

"$bun" run --cwd site build
"$bun" run --cwd web build

# La même disposition que sur GitHub Pages : la vitrine à la racine,
# l'application sous /app/.
rm -rf server/public
cp -r site/dist server/public
cp -r web/dist server/public/app

echo "site assemblé dans server/public :"
ls server/public
