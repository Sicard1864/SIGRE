# SIGRE — Système Informatique de Gestion de Révisions Espacées

[![CI](https://github.com/Sicard1864/SIGRE/actions/workflows/ci.yml/badge.svg)](https://github.com/Sicard1864/SIGRE/actions/workflows/ci.yml)

Application web d'apprentissage combinant **répétition espacée** (SM-2 / FSRS), **classement Elo par tags** et **gamification**. Gratuite, open-source et communautaire : renforcer l'intelligence humaine face à l'IA, réduire les inégalités éducatives, optimiser la rétention des connaissances.

## Fonctionnalités

- Répétition espacée (algorithme SM-2, objectifs FSRS) sur chaque question
- Classement Elo par tag : niveau de l'utilisateur, difficulté des questions et exercices
- Exercices variés : QCM, QCU, flashcards, vrai/faux, texte à trous, classement, open_ended...
- Défis quotidiens, notifications de révision, notifications
- Groupes d'étude, devoirs avec objectif minimal à atteindre (17/20), pas de classement par moyenne
- Comptes standards, anonymes (essai) et organisations (établissements, associations)

## Stack

| Couche | Technologie | Dossier |
|---|---|---|
| Backend | Rust (Axum + SQLx) | `back/` |
| Frontend | React + TypeScript (Vite) | `front/` |
| Base de données | PostgreSQL 15+ | migrée par SQLx |

## Prérequis

- [Rust](https://rustup.rs) (rustup, toolchain stable)
  — sous Windows, le linker MSVC est requis (installé par rustup-init, ou [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/))
- [Node.js](https://nodejs.org) LTS
- [PostgreSQL](https://www.postgresql.org/download/) 15 minimum (`UNIQUE NULLS NOT DISTINCT` utilisé dans le schéma)
- [SQLx CLI](https://github.com/launchbadge/sqlx) : `cargo install sqlx-cli --no-default-features --features postgres`

## Installation

```bash
git clone git@github.com:Sicard1864/SIGRE.git
cd SIGRE
```

### 1. Base de données

Crée la base (adapte le nom d'utilisateur) :

```bash
psql -U postgres -h localhost
CREATE DATABASE revisions;
\q
```

### 2. Configuration du backend

Crée le fichier d'environnement depuis l'exemple :

```bash
cd back
cp .env.example .env      # sous Windows : Copy-Item .env.example .env
```

Puis édite `back/.env` et remplace le mot de passe :

```
DATABASE_URL=postgres://postgres:TON_MOT_DE_PASSE@localhost:5432/revisions
```

> ⚠️ `.env` ne doit **jamais** être commité (il est ignoré par `.gitignore`). Seul `.env.example` est versionné.

Applique les migrations :

```bash
sqlx migrate run
```

### 3. Lancer le backend (port 3000)

```bash
cargo run
```

Vérification : <http://127.0.0.1:3000/health> doit répondre `OK db=1`.

### 4. Lancer le frontend (port 5173)

```bash
cd ../front
npm install
npm run dev
```

L'application : <http://localhost:5173>.

### 5. Tout lancer avec Docker (optionnel)

Avec [Docker Desktop](https://www.docker.com/products/docker-desktop/) installé, une seule commande démarre les 3 serveurs (base incluse, migrations appliquées automatiquement au démarrage du back) :

```bash
docker compose up --build
```

- API : <http://localhost:3000/health>
- Application : <http://localhost:8080>

## Tests

```bash
# Backend
cd back && cargo test

# Frontend (lint + build TypeScript ; tests unitaires front à venir)
cd front && npm run lint && npm run build
```

## Structure du dépôt

```
SIGRE/
├── doc/               documentation (architecture, domaine, conventions, navigation)
├── back/              API Rust (Axum + SQLx)
│   ├── migrations/    migrations SQL versionnées (SQLx)
│   └── src/
├── front/             application React + TypeScript (Vite)
├── TODO.md            feuille de route
├── TIL.md             notes d'apprentissage du projet
└── LICENSE            GPL-3.0
```

## Documentation

Toute la documentation est dans [`doc/`](doc/) :

- [Architecture](doc/architecture.md) et [modèle de domaine](doc/domain-model.md)
- [Cahier des charges](doc/about-project.md) et [navigation](doc/navigation.md)
- Conventions de code : [Rust](doc/style-rust.md), [React](doc/style-react.md), [SQL](doc/style-sql.md), [API](doc/style-api.md)
- [Workflow Git](doc/git-workflow.md)

## Versions testées

Développement et tests effectués avec : Rust 1.100.0 (édition 2024) · Node.js 26.13.1 LTS · PostgreSQL 16.11 · Windows 10/11.

Aucun blocage connu sur d'autres configurations récentes (Linux, PostgreSQL 17+).

## Contribuer

Les contributions sont les bienvenues ! Avant d'ouvrir une pull request :

1. Lis le [workflow Git](doc/git-workflow.md) (nommage des branches et des commits) et le [guide API](doc/style-api.md).
2. Respecte les conventions de style (Rust, React, SQL) documentées ci-dessus.
3. Toute PR doit compiler et passer les tests : `cargo test` côté back, `npm test` côté front.
4. En soumettant une contribution, tu acceptes qu'elle soit publiée sous GPL-3.0.

## Licence

Projet sous licence [GPL-3.0](LICENSE) — © 2026 Arthur Sicard.

Toute redistribution ou modification doit conserver la même licence et rester open-source.