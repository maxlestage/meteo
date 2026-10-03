# L'application web, en Yew

La même chose que sur iPhone, dans un navigateur : la journée en cours, la
semaine, les fenêtres de traitement et les indicateurs agronomiques de la
parcelle.

## Ce qui ne se négocie pas

- **Rien ne défile de côté.** Le bandeau horaire est une grille qui se replie
  et montre douze heures — une demi-journée ; les autres se déplient d'un
  bouton. Seule l'application iOS a un bandeau qui glisse.
- **Une carte ne prend pas tout l'écran.**
- **On part de là où est la personne.** La position n'est demandée qu'à
  défaut — jamais par-dessus une parcelle déjà choisie ni par-dessus un lien
  partagé —, elle ne bloque pas l'affichage, et un refus ne dit rien.
- **La parcelle vit dans l'adresse.** Le bouton retour la défait, et l'adresse
  envoyée à quelqu'un lui montre bien la parcelle qu'on a regardée.

## Construire

```sh
trunk serve                      # développement, sur :8081
KLIMA_RELAY=meme-origine trunk build --release
```

L'application est publiée sous `/app/` de la vitrine : les chemins produits
sont relatifs, il n'y a donc rien à reconfigurer pour la servir d'un
sous-chemin.
