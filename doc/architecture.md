# Architecture — Plateforme d'apprentissage

> **Portée :** V1 (monolithe en 3 serveurs Docker : web, api, db).
> **Documents liés :** `DOMAIN_MODEL.md` (le *quoi* métier), `ABOUT_PROJECT.md` § 2 (le *pourquoi* des choix), `STYLE_RUST.md`, `STYLE_API.md`, `STYLE_SQL.md`, `STYLE_REACT.md`, `NAVIGATION.md` (le front, écran par écran), `GIT_WORKFLOW.md` (branches et CI).
> **Nom de code :** `sigre` est le préfixe des crates Rust (`sigre-domain`, `sigre-api`…) ; si le nom du projet change, un renommage se fait tôt, avant les premiers consommateurs.

## Sommaire

1. [Présentation](#1-présentation)
2. [Vue d'ensemble](#2-vue-densemble)
3. [Organisation du dépôt](#3-organisation-du-dépôt)
4. [Base de données](#4-base-de-données)
5. [Backend](#5-backend)
6. [API](#6-api)
7. [Frontend](#7-frontend)
8. [Tests](#8-tests)
9. [Configuration et secrets](#9-configuration-et-secrets)
10. [Docker et environnements](#10-docker-et-environnements)
11. [CI/CD](#11-cicd)
12. [Décisions d'architecture](#12-décisions-darchitecture)
13. [Questions ouvertes](#13-questions-ouvertes)
14. [Annexe — initialiser le dépôt](#14-annexe--initialiser-le-dépôt)

---

## 1. Présentation

Application web d'apprentissage : exercices, répétition espacée (SM-2), niveau et difficulté par domaine (Elo sur des tags), groupes et devoirs.

Technologies V1 (décision actée, `ABOUT_PROJECT.md` § 0 et § 2.2) :

- **Base de données :** PostgreSQL 16 (15 minimum, requis pour `UNIQUE NULLS NOT DISTINCT`), schéma géré par **migrations SQLx versionnées** (`STYLE_SQL.md` § 8) ;
- **Backend :** **Rust** (Axum sur Tokio, SQLx, serde, thiserror, tracing) — C#/.NET est écarté, la correspondance C# → Rust est dans `STYLE_RUST.md` § 18 ;
- **Frontend :** React + TypeScript (Vite, TanStack Query) ;
- **Tests :** `cargo test` (back), Vitest + Testing Library (front), Playwright (bout en bout) ;
- **Déploiement :** Docker, Docker Compose ;
- **Développement :** Windows + Docker Desktop (WSL 2) ; cible d'hébergement : Debian (VPS).

Le tout vit dans **un seul dépôt Git** (monorepo) mais en **trois tiers indépendants** : chacun se construit, se teste et se déploie seul.

---

## 2. Vue d'ensemble

```text
Navigateur
    │  HTTPS
    ▼
┌──────────────────┐   /api/*   ┌──────────────────┐   SQL   ┌──────────────────┐
│ web              │ ─────────► │ api              │ ──────► │ db               │
│ Nginx + build    │            │ Axum (Rust)      │         │ PostgreSQL       │
│ React            │            │ (monolithe)      │         │                  │
└──────────────────┘            └──────────────────┘         └──────────────────┘
                                                                      ▲
                                                             ┌──────────────────┐
                                                             │ migrate          │
                                                             │ sqlx-cli (1 coup)│
                                                             └──────────────────┘
```

- **web** sert le front et relaie `/api/*` vers **api** : une seule origine, donc pas de CORS à configurer (le CORS restreint de `STYLE_API.md` § 11 reste en place par principe, mais il n'a rien à autoriser en production).
- **migrate** applique les scripts de `db/migrations/` puis s'arrête ; **api** ne démarre qu'après.
- La base ne dépend pas du backend : elle peut être créée, migrée et interrogée sans lui (client SQL, futur worker, outil d'administration).

Les **3 serveurs** du CDC (front, back, bdd) sont les conteneurs `web`, `api`, `db` ; `migrate` est un job éphémère du même compose, pas un serveur. Le worker (défis quotidiens, rappels) sera un service supplémentaire du même compose, sans changer le modèle.

---

## 3. Organisation du dépôt

```text
sigre/
├── .github/
│   └── workflows/
│       ├── back.yml              fmt + clippy + sqlx prepare + tests + build + image
│       ├── front.yml             lint + types + tests + build + image
│       └── e2e.yml               Playwright sur la pile complète
├── docs/
│   ├── CONTEXT.md                porte d'entrée : cartographie des documents
│   ├── ABOUT_PROJECT.md          le cahier des charges
│   ├── ARCHITECTURE.md           ce document
│   ├── DOMAIN_MODEL.md          le modèle métier (le pourquoi des tables)
│   ├── NAVIGATION.md            écrans, droits, parcours
│   └── STYLE_*.md, GIT_WORKFLOW.md, TODO.md, TIL.md
├── db/
│   ├── migrations/
│   │   └── V0001__initial_schema.sql
│   ├── seeds/                    données de référence et de démo (pays, tags…)
│   └── README.md
├── backend/                      workspace Cargo
│   ├── Cargo.toml               [workspace] : membres, dépendances communes
│   ├── rust-toolchain.toml
│   ├── rustfmt.toml
│   ├── crates/
│   │   ├── domain/
│   │   ├── application/
│   │   ├── infrastructure/
│   │   └── api/
│   ├── .sqlx/                    cache hors ligne des requêtes vérifiées (versionné)
│   ├── migrations -> ../db/migrations   (ou variable SQLX_MIGRATIONS_DIR)
│   └── Dockerfile
├── frontend/
│   ├── src/
│   ├── public/
│   ├── vite.config.ts
│   ├── nginx.conf
│   └── Dockerfile
├── e2e/                          Playwright : teste les trois tiers ensemble
├── docker-compose.yml
├── .env.example                  variables attendues, sans valeur secrète
├── .gitattributes                fins de ligne LF (scripts et conteneurs Linux)
├── .gitignore
└── README.md
```

Règle : un dossier de premier niveau = un tier ou un outil transverse. Rien du backend ne lit `frontend/`, et inversement ; le seul contrat entre eux est l'API (§ 6). La documentation vit dans `docs/` et suit les mêmes commits que ce qu'elle décrit (`CONTEXT.md` donne la cartographie).

---

## 4. Base de données

### Source de vérité

Le schéma est défini par les scripts de `db/migrations/`, comme l'annonce `DOMAIN_MODEL.md`. **Il n'existe aucun ORM** : SQLx mappe des requêtes écrites à la main, vérifiées à la compilation. Le schéma n'est jamais dérivé du code Rust.

| Règle | Détail (`STYLE_SQL.md` § 8) |
|---|---|
| Nommage | `V<numéro sur 4 chiffres>__<description_snake_case>.sql`, générés par `sqlx migrate add` |
| Immuabilité | un script appliqué n'est jamais modifié : on en ajoute un nouveau |
| Contenu | DDL (tables, contraintes nommées, index, déclencheurs `set_updated_at`) ; les données de démo vont dans `seeds/` |
| Contrôle | la table `_sqlx_migrations` record ce qui est appliqué ; `sqlx migrate info` audite, la CI vérifie que le cache `.sqlx` suit le code |

### Ce que la base garantit, ce que l'application garantit

| Garanti par la base | Garanti par l'application |
|---|---|
| Clés primaires et étrangères | Calculs SM-2 et Elo |
| `CHECK` sur une ligne (arc exclusif de `elo_rating`, plancher d'`ease_factor`, `span_end > span_start`…) | Règles entre plusieurs lignes ou tables (un seul `owner` par groupe, cycles de tags…) |
| Unicité (index partiels, `UNIQUE NULLS NOT DISTINCT`, index sur expression `LOWER(username)`) | Droits d'accès |
| `updated_at` par déclencheur | Orchestration d'un cas d'usage (une transaction) |

Les deux se complètent : l'application vérifie pour renvoyer un message clair (le code d'erreur `STYLE_API.md` § 7), la base vérifie pour qu'aucun client ne puisse contourner la règle.

---

## 5. Backend

### 5.1 Couches

Quatre crates du workspace, une par couche. La règle de dépendance n'est pas une convention : **c'est le compilateur qui la fait respecter**, via les `[dependencies]` du `Cargo.toml` (une crate ne peut référencer que ce qu'elle déclare).

```text
api ─────────────► application ─────────────► domain
 │                      ▲
 │                      │ implémente les traits (ports)
 └──► infrastructure ───┘
      (construite dans build_state() au démarrage)
```

| Crate | Dépend de | Ne doit jamais dépendre de |
|---|---|---|
| `sigre-domain` | rien (time, newtypes) | axum, sqlx, tokio |
| `sigre-application` | `domain` | sqlx, axum |
| `sigre-infrastructure` | `application`, `domain` | `api` |
| `sigre-api` | les trois | — |

`api` référence `infrastructure` uniquement pour construire l'état au démarrage (`build_state()`). Les handlers ne parlent qu'aux services de l'`application` (`STYLE_RUST.md` § 13-14).

### 5.2 Organisation interne : par module

Dans chaque crate, les fichiers sont rangés par **module métier**, calqué sur les sections de `DOMAIN_MODEL.md` — pas par type (`models/`, `services/` deviendrait illisible à sept domaines) :

| Module | Tables (`DOMAIN_MODEL.md`) |
|---|---|
| `accounts` | `account`, `verified_identity`, `friendship`, `account_block`, `study_group`, `study_group_member` (§ 1) |
| `content` | `exercise`, `question`, `exercise_question`, `answer` (§ 2) |
| `tags` | `tag` et tables d'association (§ 3) |
| `practice` | `assignment`, `practice_session`, `attempt` (§ 4) |
| `learning` | `srs_state`, `elo_rating`, `elo_rating_history` (§ 5) |
| `moderation` | `rating`, `feedback` (§ 6) |
| `reference` | `country` (§ 0) |

### 5.3 `sigre-domain`

Cœur métier, sans aucune dépendance technique (`STYLE_RUST.md` § 8) :

```text
sigre-domain/src/
├── common/        AccountId, QuestionId (newtypes), DomainError
├── accounts/      Account, Friendship, StudyGroup, StudyGroupMember…
├── content/       Exercise, Question, Answer, QuestionType, PublicationStatus…
├── tags/          Tag
├── practice/      Assignment, PracticeSession, Attempt
├── learning/      SrsState, Sm2Scheduler, EloRating, EloCalculator
├── moderation/    Rating, Feedback
└── reference/     Country
```

Contient :

- les entités et leurs règles propres : champs privés, fabriques `Result` (`Friendship::request(...)` renvoie `Result<Friendship, FriendshipError>`, thiserror) ;
- les énumérations, avec la correspondance vers les valeurs SQL au même endroit (`impl TryFrom<&str>`, jamais d'ordinale) ;
- **les moteurs SM-2 et Elo, fonctions synchrones et pures** : mêmes entrées, même sortie, aucune base, aucune horloge — c'est la partie la plus précieuse à tester (§ 8), parce qu'un SM-2 faux ne plante pas : il programme silencieusement les révisions aux mauvaises dates.

```rust
srs_state.review(RecallGrade::Four, now);                  // ease_factor, interval_days, due_at
let p = elo_calculator::expected_score(account_rating, question_rating);
let (new_account, new_question) =
    elo_calculator::update(account_rating, question_rating, score_ratio, K);
```

### 5.4 `sigre-application`

Orchestre les cas d'usage : s'inscrire, créer une question, donner un devoir, soumettre une réponse…

```text
sigre-application/src/
├── ports/                  les traits que l'infrastructure implémente
│   ├── repositories.rs     AccountsRepo, PracticeRepo… (fn async autour d'un tx)
│   ├── clock.rs             Clock (l'heure injectée, pour tout tester)
│   └── security.rs          PasswordHasher, TokenService
├── accounts/               AccountService, AuthService
├── content/                ExerciseService, QuestionService
├── practice/               PracticeService, AssignmentService
├── learning/               ReviewQueueService
└── moderation/             RatingService, FeedbackService
```

Contient :

- les services applicatifs : une méthode = un cas d'usage, qui **ouvre et commit une transaction** (`STYLE_RUST.md` § 12) ;
- les **ports** (traits) que l'infrastructure implémente — repositories, horloge, hachage, jetons ;
- les validations qui demandent plusieurs entités ou un accès aux données (« la question appartient-elle à l'exercice de la séance ? ») ;
- les contrôles de droits (« ce compte est-il `owner` ou `admin` du groupe ? »).

**L'heure courante est injectée** (`now: OffsetDateTime` passé en paramètre, fourni par un port `Clock`) au lieu de lire l'horloge réelle : indispensable pour tester SM-2, les échéances de devoirs et les séries quotidiennes. Le Domain reçoit l'heure, ne la lit jamais (`STYLE_RUST.md` § 8).

### 5.5 `sigre-infrastructure`

```text
sigre-infrastructure/src/
├── persistence/
│   ├── accounts.rs       requêtes SQLx + structs de chargement (*Row) + try_into_domain()
│   ├── practice.rs
│   └── …                 un fichier par module, pas par table
├── security/              Argon2PasswordHasher, JwtTokenService
├── clock.rs               SystemClock (l'implémentation réelle du port)
└── state.rs               build_state(pool, config) -> AppState
```

Points de configuration SQLx :

- **requêtes vérifiées** par les macros `query!`/`query_as!`, cache hors ligne versionné dans `.sqlx/` (`cargo sqlx prepare`, `SQLX_OFFLINE=true` en CI : **la CI n'a pas de base pour compiler**) ;
- **structs de chargement distincts des entités** quand les champs diffèrent (`SrsStateRow` → `try_into_domain()`), le Domain garde ses champs privés ;
- **énumérations** converties via les `TryFrom` du Domain, jamais de `to_string()` dispersé ;
- **`updated_at` tenu par la base** (déclencheur) : SQLx ne l'écrit pas ;
- **transactions** : le service passe `&mut PgConnection` (ou `Transaction`) aux fonctions du repository, commit au niveau du cas d'usage ;
- **mot de passe** : Argon2id au format PHC, le poivre vient de la configuration (§ 9), jamais du code.

### 5.6 `sigre-api`

```text
sigre-api/src/
├── main.rs                 #[tokio::main], build_state(), Router, écoute
├── state.rs                AppState { pool, config, services } (STYLE_RUST.md § 13)
├── routes/                 un module par ressource (accounts.rs, practice_sessions.rs…)
├── dto/                    DTO serde par module, conversions From/TryFrom explicites
├── error.rs                ApiError -> ProblemDetails RFC 9457 (STYLE_API.md § 7)
├── middleware/             authentification (CurrentUser), X-Request-Id, tracing
└── config.rs               AppConfig désérialisée depuis l'environnement, validée au démarrage
```

Un handler reçoit la requête, appelle un service, traduit le résultat en HTTP (`STYLE_RUST.md` § 14). Pas de logique métier, pas de SQL. Les erreurs suivent `ProblemDetails` :

| Erreur | Code HTTP |
|---|---|
| validation d'un DTO entrant | 422 |
| non authentifié | 401 |
| droit insuffisant | 403 |
| ressource absente | 404 |
| règle métier violée (`DomainError`) | 409 ou 422 selon le code (`STYLE_API.md` § 7) |

### 5.7 Flux principal : soumettre une réponse

C'est le cas d'usage central (`DOMAIN_MODEL.md` § 7, `ABOUT_PROJECT.md` § 3.1) : il touche les trois couches métier (pratique, SRS, Elo) en une seule transaction.

```text
POST /api/v1/practice-sessions/{session_id}/attempts
    ↓
routes::practice_sessions::submit_attempt (handler)
    ├─ extracteurs : State, CurrentUser, Path, Json<SubmitAttemptRequest>
    └─ TryFrom<&SubmitAttemptRequest> -> SubmitAttemptCommand
    ↓
PracticeService::submit_attempt(account_id, session_id, command)
    ├─ ouvre la transaction (pool.begin())
    ├─ charge la séance, la question (avec ses tags), le srs_state, les elo_rating
    ├─ vérifie : séance du compte connecté, non terminée, question autorisée
    ├─ Attempt::new(...)                                domain
    ├─ srs_state.review(recall_grade, now)              domain (SM-2, synchrone)
    ├─ pour chaque tag de la question :
    │     crée account_tag + elo_rating s'ils manquent
    │     elo_calculator::update(...)                   domain (Elo, synchrone)
    │     insère deux lignes elo_rating_history
    └─ commit                                           une seule transaction
    ↓
201 Created  ->  AttemptResponse (score, prochaine révision, elo_deltas)
```

Si une étape échoue, la transaction est annulée : pas de tentative sans mise à jour SRS, pas d'Elo à moitié modifié.

### 5.8 Où placer la logique ?

| Type de logique | Couche |
|---|---|
| Routes, DTO, codes HTTP, ProblemDetails | api |
| Orchestration, transactions, droits, validations entre entités | application |
| Règles propres à une entité, SM-2, Elo | domain |
| SQLx, Argon2, JWT, horloge réelle | infrastructure |
| Contraintes d'intégrité (clés, `CHECK`, unicité) | base de données |

### 5.9 Principaux fichiers d'une fonctionnalité

Exemple avec la pratique :

| Fichier | Crate | Rôle |
|---|---|---|
| `practice/practice_session.rs`, `practice/attempt.rs` | domain | Entités et règles |
| `learning/srs_state.rs`, `learning/elo_calculator.rs` | domain | Algorithmes (purs, testés à part) |
| `ports/repositories.rs` (trait `PracticeRepo`) | application | Contrat d'accès aux données |
| `practice/practice_service.rs` | application | Cas d'usage + transaction |
| `persistence/practice.rs` | infrastructure | Requêtes SQLx vérifiées |
| `routes/practice_sessions.rs` | api | Handler HTTP |
| `dto/practice.rs` | api | DTO + conversions |
| `srs_state.rs` (mod `tests`), `practice_service.rs` (tests) | — | Vérifications |

---

## 6. API

Conventions détaillées dans `STYLE_API.md` :

- préfixe versionné `/api/v1` ; ressources au pluriel, `kebab-case` ; imbrication limitée à un niveau ;
- clés JSON en `snake_case`, identiques aux colonnes SQL ; identifiants `BIGINT` en chaînes dans le JSON ;
- erreurs `ProblemDetails` RFC 9457 avec un `code` stable (`friendship.not_pending`) ;
- pagination obligatoire sur toute collection (`page`, `per_page`), `Idempotency-Key` sur les créations sensibles ;
- santé : `GET /healthz` (hors versionnement) pour le compose et la supervision.

```text
POST   /api/v1/accounts
POST   /api/v1/auth/login
GET    /api/v1/questions?tag_id=12&page=1&per_page=20
POST   /api/v1/questions
POST   /api/v1/practice-sessions
POST   /api/v1/practice-sessions/{id}/attempts
PATCH  /api/v1/practice-sessions/{id}/finish
GET    /api/v1/me/review-queue
POST   /api/v1/study-groups/{id}/assignments
```

### Contrat entre front et back

Le backend publie sa description **OpenAPI générée depuis le code** (`utoipa` sur les handlers et DTO, servie sur `/api/v1/openapi.json`, `STYLE_API.md` § 12). Le frontend **génère ses types TypeScript** à partir de ce fichier (`openapi-typescript`) au lieu de les recopier à la main : si un DTO change côté Rust, la compilation TypeScript casse au lieu du navigateur de l'utilisateur. La couche `src/api/` du front (`STYLE_REACT.md` § 10) encapsule ces types ; aucun composant ne déclare un type de contrat à la main.

---

## 7. Frontend

SPA React + TypeScript construite avec Vite. L'organisation et les conventions de code sont dans `STYLE_REACT.md` ; les écrans, droits et parcours sont dans `NAVIGATION.md` (source de vérité des routes : `frontend/src/app/routes.tsx`).

```text
frontend/src/
├── app/            App.tsx, routes.tsx, providers (auth, TanStack Query)
├── features/       une brique métier par dossier
│   ├── practice/     le lecteur de questions (mode focus)
│   ├── questions/    éditeurs de question et d'exercice
│   ├── groups/       groupes, membres, devoirs
│   ├── auth/         connexion, inscription, session
│   └── dashboard/    Aujourd'hui, progression
├── api/            la seule couche qui parle au réseau (types générés depuis OpenAPI)
├── shared/         composants sans métier (bouton, champ, modale…)
└── main.tsx
```

Règle de dépendance, la même idée qu'au backend : `features` → `shared`, `api` est le seul point réseau, jamais l'inverse.

Le serveur de développement Vite relaie `/api` vers le backend local (`server.proxy`) ; en production, c'est Nginx (§ 10). Le code du front appelle toujours `/api/...`, sans adresse en dur.

---

## 8. Tests

### 8.1 Pyramide

```text
                  ▲  lent, fragile, rassurant
       ┌───────────────────┐
       │  e2e (Playwright) │   quelques parcours critiques
       ├───────────────────┤
       │  Intégration      │   API + vraie base PostgreSQL (testcontainers)
       ├───────────────────┤
       │  Unitaires        │   domain, application, composants React
       └───────────────────┘
                  ▼  rapide, ciblé, nombreux
```

| Quoi | Outils | Dépendances externes |
|---|---|---|
| Entités, SM-2, Elo (`#[cfg(test)]` dans le fichier) | `cargo test` | aucune |
| Services, droits, orchestration | `cargo test`, faux écrits à la main | aucune |
| Routes, requêtes SQLx, contraintes SQL | `cargo test` (dossier `tests/`), testcontainers-rs | Docker |
| Composants et hooks React | Vitest, Testing Library, MSW | aucune |
| Parcours complets | Playwright (`e2e/`) | pile Docker complète |

Côté back, les tests unitaires vivent **dans le fichier testé** (mod `#[cfg(test)]`, convention Rust) et les tests d'intégration dans `crates/api/tests/`. Côté front, les tests sont **à côté du composant** (`Flashcard.tsx` / `Flashcard.test.tsx`) : on voit tout de suite quel composant n'est pas testé. Les tests e2e, qui traversent les trois tiers, ont leur propre dossier à la racine.

### 8.2 Tests du domain : les plus rentables

SM-2 et Elo sont des fonctions pures : on les teste sans rien simuler (`STYLE_RUST.md` § 16).

```rust
#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    fn now() -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(1_773_000_000).unwrap()
    }

    #[test]
    fn review_first_success_schedules_review_tomorrow() {
        let start = now();
        let mut state = SrsState::new(account_id(1), question_id(1));

        state.review(5, start);

        assert_eq!(state.due_at(), Some(start + time::Duration::days(1)));
    }

    #[test]
    fn review_failure_resets_repetition_count() {
        let start = now();
        let mut state = SrsState::new(account_id(1), question_id(1));
        state.review(5, start);
        state.review(5, start + time::Duration::days(1));

        state.review(2, start + time::Duration::days(7));

        assert_eq!(state.repetition_count(), 0);
    }

    #[test]
    fn ease_factor_never_drops_below_1_3() {
        let start = now();
        let mut state = SrsState::new(account_id(1), question_id(1));

        for day in 0..20 {
            state.review(0, start + time::Duration::days(day));
        }

        assert!(state.ease_factor() >= 1.3);
    }
}
```

```rust
#[cfg(test)]
mod elo_tests {
    #[test]
    fn equal_ratings_give_even_chance() {
        assert!((elo::expected_score(1500.0, 1500.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn beating_a_harder_question_earns_more_than_half_k() {
        let (new_account, _) = elo::update(1400.0, 1600.0, 1.0, 32.0);

        assert!(new_account - 1400.0 > 16.0);
    }
}
```

Nommage : `méthode_ou_situation_résultat_attendu` en `snake_case`, lisible comme une phrase dans le rapport de tests.

### 8.3 Tests de l'application : faux écrits à la main

On remplace le repository SQL par une implémentation en mémoire du trait (port), écrite à la main : lisible, aucune bibliothèque à apprendre. Une crate de simulation ne s'ajoute que si les faux deviennent pénibles à maintenir.

```rust
// crates/application/tests/fakes.rs
pub struct FakePracticeRepo {
    pub sessions: Vec<PracticeSession>,
    pub elo_ratings: Vec<EloRating>,
    // … implémentation en mémoire du trait PracticeRepo
}

#[tokio::test]
async fn first_answer_creates_account_elo_for_each_question_tag() {
    let mut repo = FakePracticeRepo::default();
    repo.add_question_with_tags(question_id(10), vec![tag_id(1), tag_id(2)]);
    let session = repo.add_session(account_id(42));
    let service = PracticeService::new(Arc::new(repo), FixedClock(at(instant)));

    let attempt = service
        .submit_attempt(account_id(42), session.id(), submit_command(question_id(10)))
        .await
        .unwrap();

    assert_eq!(repo.elo_ratings_of(account_id(42)).len(), 2);
}
```

`FixedClock` (port `Clock` figé sur un instant connu) remplace l'horloge réelle : on avance le temps à volonté pour tester une révision due, une échéance de devoir, une série quotidienne.

### 8.4 Tests d'intégration : vraie base PostgreSQL

Pas de base « InMemory » ni de SQLite de substitution : le modèle repose sur des `CHECK`, des index partiels et `UNIQUE NULLS NOT DISTINCT` qu'un substitut ignore. **testcontainers-rs** démarre un vrai PostgreSQL jeté dans Docker, on y applique `db/migrations/`, puis on lance l'API (ou seulement les routes) contre cette base.

```rust
// crates/api/tests/common.rs
pub async fn test_app() -> (AxumRouter, PostgresContainer) {
    let container = PostgreSql::default()
        .with_tag("16")
        .start()
        .await
        .expect("postgres container");
    let url = container.url().await;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("pool");
    sqlx::migrate!("../../db/migrations")
        .run(&pool)
        .await
        .expect("migrations");
    let state = build_state(pool, test_config());
    (app_router(state), container)
}

#[tokio::test]
async fn create_question_without_authentication_returns_401() {
    let (app, _container) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/questions")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content": "2 + 2 ?"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

Ces tests attrapent ce que les tests unitaires ne voient pas : une requête SQLx fausse, un `CHECK` violé, une route mal protégée. Ils sont plus lents : on en écrit moins, sur les chemins importants (authentification, pipeline d'une réponse, droits de groupe).

### 8.5 Tests du frontend

```tsx
// src/features/practice/Flashcard.test.tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Flashcard } from './Flashcard';

test('affiche la réponse après un clic sur « Retourner »', async () => {
  render(<Flashcard question="2 + 2 ?" answer="4" />);

  await userEvent.click(screen.getByRole('button', { name: /retourner/i }));

  expect(screen.getByText('4')).toBeInTheDocument();
});
```

On teste ce que voit l'utilisateur (rôles, textes), pas l'implémentation. Les appels à l'API sont simulés au niveau réseau (MSW) plutôt qu'en remplaçant des fonctions.

### 8.6 Tests bout en bout

Peu nombreux, sur les parcours qui ne doivent jamais casser (`NAVIGATION.md` § 6) : inscription → connexion → séance de révision → réponse → prochaine révision affichée. Ils tournent contre la pile `docker compose` complète.

---

## 9. Configuration et secrets

| Où | Contenu | Versionné |
|---|---|---|
| `.env.example` | **liste** des variables attendues, valeurs factices | oui |
| `.env` | valeurs réelles pour Docker Compose | **non** (`.gitignore`) |
| variables d'environnement | tout ce que l'API lit au démarrage | selon l'environnement |

Secrets : mot de passe de la base, clé de signature JWT, poivre des mots de passe. Ils arrivent dans l'API par variables d'environnement uniquement (`DATABASE_URL`, `JWT_SECRET`, `PASSWORD_PEPPER`), désérialisées dans `AppConfig` **validée au démarrage** (`STYLE_RUST.md` § 13) : une variable manquante fait échouer le démarrage avec un message clair, pas la première connexion d'un utilisateur. Aucun secret dans le code, dans le `Cargo.toml` ni dans le compose commité.

**Changer le poivre rend tous les mots de passe existants invalides** : il se sauvegarde comme la base (§ 10, sauvegardes).

---

## 10. Docker et environnements

```yaml
# docker-compose.yml (extrait)
services:
  db:
    image: postgres:16
    environment:
      POSTGRES_DB: sigre
      POSTGRES_USER: ${DB_USER}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - db-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER} -d sigre"]
      interval: 5s
      retries: 10

  migrate:
    image: ghcr.io/launchbadge/sqlx-cli:latest
    command: >
      migrate run
      --source /migrations
      --database-url postgres://${DB_USER}:${DB_PASSWORD}@db:5432/sigre
    volumes:
      - ./db/migrations:/migrations:ro
    depends_on:
      db:
        condition: service_healthy

  api:
    build: ./backend
    environment:
      DATABASE_URL: postgres://${DB_USER}:${DB_PASSWORD}@db:5432/sigre
      JWT_SECRET: ${JWT_SECRET}
      PASSWORD_PEPPER: ${PASSWORD_PEPPER}
    depends_on:
      migrate:
        condition: service_completed_successfully

  web:
    build: ./frontend
    ports:
      - "8080:80"
    depends_on:
      - api

volumes:
  db-data:
```

- Seul **web** expose un port : la base et l'API restent sur le réseau interne de Docker. En développement, on peut exposer `5432` (client SQL) et le port de l'API dans un `docker-compose.override.yml` non utilisé en production.
- `frontend/nginx.conf` sert les fichiers statiques, renvoie `index.html` pour les routes de la SPA et relaie `/api/` vers `http://api:8080`.
- Les deux `Dockerfile` sont *multi-stage* : une image avec la chaîne complète pour construire (rust / node), une image minimale pour exécuter (Debian slim côté api). Côté Rust, `cargo-chef` est l'option à étudier pour ne pas recompiler les dépendances à chaque build.
- **Sauvegardes** : `pg_dump` planifié sur le volume `db-data`, **avec test de restauration régulier** (une sauvegarde jamais restaurée n'est pas une sauvegarde). Le poivre (§ 9) et la clé JWT font partie des secrets à sauvegarder avec.

| Environnement | Lancement | Usage |
|---|---|---|
| Développement | `docker compose up db migrate` puis back (`cargo watch -x check`) et front (`npm run dev`) en local | rechargement à chaud, débogueur |
| Pile complète | `docker compose up --build` | vérifier l'ensemble, tests e2e |
| Production | images publiées, `docker compose pull && docker compose up -d` sur Debian (VPS), derrière un reverse proxy TLS | — |

---

## 11. CI/CD

Plateforme : GitHub Actions (auto-hébergement plus tard, `GIT_WORKFLOW.md` § 8). Un workflow par tier, déclenché **seulement si ses fichiers changent** : une modification du front ne relance pas les tests du back.

| Workflow | Déclencheur | Étapes |
|---|---|---|
| `back.yml` | `backend/**`, `db/**` | fmt → clippy `-D warnings` → `sqlx prepare --check` → tests (testcontainers pour l'intégration) → build → `cargo audit` + `cargo deny` |
| `front.yml` | `frontend/**` | `npm ci` → lint → types → Vitest → build |
| `e2e.yml` | pull request vers `main` | `docker compose up --build` → Playwright |
| `release.yml` (plus tard) | tag `v*` | construction et publication des images, déploiement |

```yaml
# .github/workflows/back.yml (extrait)
name: back
on:
  pull_request:
    paths: ["backend/**", "db/**"]
  push:
    branches: [main]
    paths: ["backend/**", "db/**"]

env:
  SQLX_OFFLINE: "true"      # la CI compile sans base : cache .sqlx versionné

jobs:
  test:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: backend
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo sqlx prepare --check
      - run: cargo test --all           # Docker est disponible sur les runners Linux (testcontainers)
      - run: cargo audit
      - run: cargo deny check licenses
```

La branche `main` est protégée : fusion uniquement par pull request avec les workflows concernés au vert (`GIT_WORKFLOW.md` § 7). Merge sur `main` = build des images + déploiement : `main` est ce qui tourne en production.

---

## 12. Décisions d'architecture

| Décision | Pourquoi |
|---|---|
| Un seul dépôt pour les trois tiers | Un changement d'API et son adaptation côté front passent dans la même pull request ; un seul endroit pour la documentation et la CI. |
| Monolithe en V1, pas de microservices | Le cahier des charges vise à terme du découpage, mais un développeur seul paie le coût des microservices (réseau, déploiement, cohérence) sans en avoir le bénéfice. Le découpage par modules (§ 5.2) prépare une séparation future. |
| Une crate par couche | Le compilateur interdit au domain de référencer SQLx ou Axum. Coût faible sur un projet qui démarre. |
| Rangement par module dans chaque couche | Le modèle métier compte sept domaines ; on trouve tout ce qui concerne la pratique au même endroit. |
| Schéma géré par migrations SQL (sqlx), pas par un ORM | Le SQL est la source de vérité de `DOMAIN_MODEL.md` ; la base reste indépendante de l'application ; les requêtes sont vérifiées à la compilation sans couche d'abstraction qui cache le SQL. |
| Requêtes SQLx en macros + cache hors ligne `.sqlx` | Erreur SQL = erreur de compilation, y compris en CI sans base. |
| SM-2 et Elo dans le domain, fonctions synchrones pures | Logique la plus critique, la plus facile à tester sans infrastructure ni runtime async. |
| `now` injecté (port `Clock`) | Rend testable tout ce qui dépend de l'heure (révisions, échéances, séries). |
| Faux écrits à la main d'abord | Lisibles, sans dépendance ; une bibliothèque de simulation viendra si le besoin est mesuré. |
| Tests d'intégration sur un vrai PostgreSQL (testcontainers) | Un substitut ne vérifie aucune contrainte, or le modèle en dépend. |
| Tests front à côté des composants, e2e à la racine | Conventions de chaque écosystème ; les e2e n'appartiennent à aucun tier. |
| Front et API sur la même origine (`/api` relayé par nginx) | Pas de CORS à gérer, cookies plus simples à sécuriser. |
| Types TypeScript générés depuis l'OpenAPI (utoipa) | Un seul contrat, vérifié à la compilation des deux côtés. |
| `archived_at` + convention unique d'archivage | Suppression logique partout (RGPD : anonymisation du compte, contenu conservé). |
| Erreurs au format ProblemDetails (RFC 9457) | Format standard ; le front n'a qu'une seule forme d'erreur à gérer. |
| Migrations par un service `migrate` éphémère | L'api ne démarre qu'avec un schéma à jour ; le `V0001` s'applique aussi sans backend (outillage, futur worker). |
| Cargo workspace plutôt qu'un crate unique | Les couches sont une frontière de compilation, pas une convention de dossiers ; les temps de build restent raisonnables avec le cache. |

---

## 13. Questions ouvertes

| Question | Piste |
|---|---|
| Stockage du JWT côté navigateur | Résolu côté conventions (`STYLE_REACT.md` § 11 : accès en mémoire, refresh en cookie `HttpOnly` `SameSite=Lax`) ; reste à trancher le **CSRF** du cookie de rafraîchissement (token dédié ou SameSite=Strict). Avant l'étape 4 de la feuille de route. |
| Worker (défis quotidiens, rappels) | 4ᵉ service du compose quand l'étape 7 l'exige ; job interne Axum/Tokio d'abord si trivial. |
| Reverse proxy et TLS en production | Caddy (certificats automatiques) ou Nginx + Certbot devant le conteneur `web`. |
| `cargo-chef` dans le Dockerfile | À mesurer dès que le build CI ralentit (cache des dépendances). |
| Couverture de tests | Mesurer (`cargo-llvm-cov`, Vitest) sans fixer d'objectif chiffré au départ ; viser le domain en priorité. |
| WebAssembly plus tard | Partager la logique SM-2/Elo (crate `domain`) entre backend et front offline — prévu par le CDC comme ouverture, sans engagement V1. |

---

## 14. Annexe — initialiser le dépôt

### Backend (PowerShell, sous Windows avec Rust installé via rustup)

```powershell
mkdir backend; cd backend

# Workspace et crates : une crate par couche (§ 5.1)
cargo init --name sigre-domain          crates/domain
cargo init --name sigre-application     crates/application --lib
cargo init --name sigre-infrastructure crates/infrastructure --lib
cargo init --name sigre-api             crates/api

# Racine du workspace : backend/Cargo.toml
@"
[workspace]
resolver = "2"
members = ["crates/domain", "crates/application", "crates/infrastructure", "crates/api"]

[workspace.package]
edition = "2021"
"@ | Set-Content Cargo.toml
```

`backend/Cargo.toml` (workspace) et dépendances de path :

```toml
[workspace.dependencies]
time = { version = "0.3", features = ["macros", "serde"] }
thiserror = "2"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
axum = "0.8"
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "chrono", "migrate"] }
```

```powershell
# Règle de dépendance (§ 5.1)
cargo add -p sigre-application --path crates/domain
cargo add -p sigre-infrastructure --path crates/application --path crates/domain
cargo add -p sigre-api --path crates/application --path crates/infrastructure --path crates/domain

# Dépendances principales
cargo add -p sigre-application serde thiserror time
cargo add -p sigre-infrastructure sqlx argon2 jsonwebtoken tracing time
cargo add -p sigre-api tokio axum serde serde_json tracing tracing-subscriber utoipa

# Outillage SQLx (une fois) puis cache hors ligne
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate add initial_schema --source ../db/migrations
# après l'écriture des premières requêtes :
cargo sqlx prepare --workspace
```

Fichiers d'outillage (`STYLE_RUST.md` § 17) : `rust-toolchain.toml`, `rustfmt.toml` (`max_width = 100`), `.cargo/config.toml` (alias `lint`).

### Frontend

```powershell
npm create vite@latest frontend -- --template react-ts
cd frontend
npm install
npm install -D vitest jsdom @testing-library/react @testing-library/user-event @testing-library/jest-dom msw openapi-typescript
```

### Bout en bout

```powershell
mkdir e2e; cd e2e
npm init playwright@latest
```

### Fichiers racine

```gitattributes
# .gitattributes : fins de ligne LF dans le dépôt, même sous Windows
* text=auto eol=lf
*.ps1 text eol=crlf
```
