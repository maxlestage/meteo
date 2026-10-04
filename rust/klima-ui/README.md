# Ce que les deux interfaces partagent

La vitrine et l'application affichent le même signe, parlent les mêmes langues,
tiennent leur parcelle dans l'adresse de la même façon et appellent les
fournisseurs par le même chemin. Deux copies de tout cela divergeraient : la
première correction n'en toucherait qu'une.

| Module       | Ce qu'il tient                                            |
| ------------ | --------------------------------------------------------- |
| `i18n`       | La langue, son contexte, sa mémoire d'une visite à l'autre |
| `dates`      | Le seul endroit où Klima appelle `Intl`                    |
| `reseau`     | Les appels qui partent du navigateur                       |
| `horloge`    | Maintenant, à la parcelle                                  |
| `storage`    | Le stockage local, et la permission de ne pas y arriver    |
| `composants` | La marque, le pictogramme météo, le sélecteur de langue    |
| `crochets`   | La parcelle dans l'adresse, la position, la prévision      |

Les textes, les feuilles de style et les composants propres à chaque interface
restent chez elle.
