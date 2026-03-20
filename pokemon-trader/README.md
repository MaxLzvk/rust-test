# Pokemon Trader

Application web d'echange de Pokemon entre dresseurs sur un reseau local.

## Fonctionnalites

- Attraper des Pokemon aleatoires (donnees depuis [PokeAPI](https://pokeapi.co/))
- Voir sa collection avec sprites, types et stats
- Relacher un Pokemon de sa collection
- Echanger des Pokemon avec d'autres dresseurs via leur adresse IP

## Compilation

```bash
go build -o pokemon-trader .
```

## Lancement

```bash
./pokemon-trader
```

### Arguments

| Argument | Description | Defaut |
|----------|-------------|--------|
| `-name`  | Votre nom de dresseur | Ash |
| `-port`  | Port d'ecoute du serveur | 3000 |

```bash
./pokemon-trader -name "Sacha" -port 8080
```

## Docker

```bash
docker build -t pokemon-trader .
docker run -p 3000:3000 pokemon-trader -name "Sacha"
```
