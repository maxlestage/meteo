#!/usr/bin/env bash
#
# Construire les deux interfaces et les poser là où le relais les servira.
#
# Trunk est téléchargé déjà compilé : le compiler ici prendrait plus que le
# temps de construction d'un dyno, pour un outil dont on n'utilise que la
# sortie.
set -euo pipefail

VERSION_TRUNK="${VERSION_TRUNK:-0.21.14}"
RACINE="$(cd "$(dirname "$0")/.." && pwd)"
DEHORS="${1:-$RACINE/../server/public}"

cd "$RACINE"

if ! command -v trunk > /dev/null; then
  echo "— Trunk $VERSION_TRUNK"
  mkdir -p "$HOME/.local/bin"
  url="https://github.com/trunk-rs/trunk/releases/download/v$VERSION_TRUNK/trunk-x86_64-unknown-linux-gnu.tar.gz"
  curl --fail --location --silent "$url" | tar -xz -C "$HOME/.local/bin"
  export PATH="$HOME/.local/bin:$PATH"
fi

rustup target add wasm32-unknown-unknown

# Les interfaces demandent leur relais à l'hôte qui les sert : une adresse qui
# n'est connue qu'à l'affichage, puisqu'elle dépend du nom de domaine.
export KLIMA_RELAY=meme-origine

echo "— la vitrine"
trunk build --release --config klima-site/Trunk.toml

echo "— l'application"
trunk build --release --config klima-web/Trunk.toml

echo "— assemblage dans $DEHORS"
rm -rf "$DEHORS"
mkdir -p "$DEHORS"
cp -r klima-site/dist/. "$DEHORS/"
mkdir -p "$DEHORS/app"
cp -r klima-web/dist/. "$DEHORS/app/"
ls "$DEHORS"
