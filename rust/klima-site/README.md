# La vitrine, en Yew

Ce qui se voit avant d'ouvrir l'application : ce que Klima calcule, sur quelles
règles, à partir de quelles sources — et la météo du jour sur la commune du
visiteur, pour que la démonstration ne soit pas une capture d'écran.

## Ce qui tient dans la page

- **La météo du jour, et rien d'autre.** La semaine et le détail heure par
  heure sont dans l'application ; les montrer ici n'ajouterait pas un argument,
  seulement du défilement.
- **Les seuils cités viennent du code.** `Fonctions` lit `AgroThresholds` et
  les met en forme dans la langue courante : la page ne peut pas annoncer autre
  chose que ce que l'application applique.
- **Le recoupement est montré, pas affirmé.** Chaque institut porte son point
  sur l'axe des températures. Dire « nous recoupons cinq instituts » n'engage à
  rien ; les montrer en désaccord de 1,9 °C, si.
- **Aucun lien vers le code source.** Un test du catalogue le vérifie.

## Construire

```sh
trunk serve                      # développement, sur :8082
KLIMA_RELAY=meme-origine trunk build --release
```

Sans `KLIMA_RELAY`, la page interroge les fournisseurs en direct — c'est le
mode de développement, et il reste sur le plan gratuit d'Open-Meteo, réservé à
un usage non commercial.
