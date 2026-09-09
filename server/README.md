# Relais Klima

Interroge les fournisseurs météo une fois pour tout le monde.

## Pourquoi il existe

Trois raisons, dans l'ordre d'importance.

**Le prix tient.** Sans relais, la facture Open-Meteo suit le nombre
d'utilisateurs : un appareil qui consulte sa parcelle quinze fois par jour
part quinze fois chez le fournisseur. Avec le relais, une cellule de grille
coûte vingt-quatre interrogations par jour, qu'elle soit ouverte par une
personne ou par mille. La facture suit le nombre de parcelles distinctes et
le rythme des modèles, pas la croissance.

**MET Norway reste sous son plafond.** Ses conditions plafonnent à vingt
requêtes par seconde *par application*, pas par appareil. Au-delà de quelques
milliers d'installations, l'appel direct depuis l'appareil dépasse ce plafond.

**La clé commerciale reste secrète.** Le plan gratuit d'Open-Meteo est réservé
à un usage non commercial : dès que Klima se vend, il faut un plan payant,
donc une clé. Une clé glissée dans un binaire distribué est une clé publiée.

## Ce qu'il n'est pas

Un point de défaillance unique. Quand le relais ne répond pas, les clients
retombent sur les fournisseurs qu'ils ont le droit d'appeler seuls — MET
Norway en natif, Bright Sky partout. On perd des sources, pas la météo.

## Lancer

```bash
bun install
bun run --cwd server start
```

| Variable          | Effet                                                                    |
| ----------------- | ------------------------------------------------------------------------ |
| `PORT`            | Port d'écoute. Par défaut `8787`.                                        |
| `OPEN_METEO_KEY`  | Clé du plan commercial. Absente : hôte public, usage non commercial.      |
| `KLIMA_ORIGINS`   | Origines autorisées, séparées par des virgules.                          |

Côté client, `VITE_KLIMA_RELAY` désigne l'origine du relais. Sans elle, les
appels partent en direct.

## Chemins

| Chemin                       | Cache             |
| ---------------------------- | ----------------- |
| `/v1/open-meteo/forecast`    | 1 h par cellule   |
| `/v1/open-meteo/search`      | 24 h par texte    |
| `/v1/met-norway/compact`     | 1 h par cellule   |
| `/v1/bright-sky/current`     | 1 h par cellule   |
| `/health`                    | —                 |

La cellule fait 0,02° de côté, soit environ deux kilomètres : la résolution
des modèles les plus fins qu'on interroge. Arrondir plus grossièrement ferait
perdre de la précision à un outil dont c'est l'argument ; le coût n'en
souffre pas, parce que le relais ne va chercher que les cellules qu'on lui
demande.

Quand un fournisseur ne répond plus, une prévision périmée dépanne encore
deux heures. Passé ce délai, l'erreur remonte — et jamais avec le message du
fournisseur, qui contiendrait l'URL, donc la clé.
