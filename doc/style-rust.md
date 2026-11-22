# Guide de style Rust — Plateforme d'apprentissage

> **Portée :** tout le code Rust de `backend/` (sources et tests).
> **Documents liés :** `ARCHITECTURE.md` (couches, dossiers), `DOMAIN_MODEL.md` (vocabulaire métier), `STYLE_SQL.md`, `STYLE_API.md`, `GIT_WORKFLOW.md`.
> **Base :** les idiomes officiels de Rust (The Book, API Guidelines de rust-lang), les conventions d'Axum, de SQLx et de Tokio, adaptées à ce projet.
> **Application :** ce qui peut être vérifié par un outil l'est (`rustfmt.toml`, `clippy` en erreur en CI, § 17). Ce document explique le reste et le *pourquoi*.
> **Origine :** ce guide remplace `STYLE_CSHARPE_GUIDE.md` (C#/.NET écarté en V1). La correspondance C# → Rust est en § 18.

## Sommaire

1. [Principes](#1-principes)
2. [Langue](#2-langue)
3. [Mise en forme](#3-mise-en-forme)
4. [Nommage](#4-nommage)
5. [Organisation d'un fichier et d'un module](#5-organisation-dun-fichier-et-dun-module)
6. [Propriété, clonage et types](#6-propriété-clonage-et-types)
7. [Option : la nullabilité](#7-option--la-nullabilité)
8. [Entités du Domain](#8-entités-du-domain)
9. [Commentaires](#9-commentaires)
10. [Erreurs : Result et paniques](#10-erreurs--result-et-paniques)
11. [Async : Tokio](#11-async--tokio)
12. [SQLx et PostgreSQL](#12-sqlx-et-postgresql)
13. [État partagé et configuration (Axum)](#13-état-partagé-et-configuration-axum)
14. [Handlers et DTO](#14-handlers-et-dto)
15. [Journalisation et données personnelles](#15-journalisation-et-données-personnelles)
16. [Tests](#16-tests)
17. [Outillage](#17-outillage)
18. [Correspondance avec le guide C#](#18-correspondance-avec-le-guide-c)

---

## 1. Principes

- **Le code se lit plus qu'il ne s'écrit.** Entre une version courte et une version claire, on prend la claire.
- **Le code se documente lui-même.** Un bon nom vaut mieux qu'un commentaire ; un commentaire vaut mieux qu'un mauvais nom.
- **Une seule façon de faire une chose.** Deux conventions qui coexistent obligent à se demander laquelle appliquer.
- **Ce qui peut être vérifié automatiquement doit l'être.** Une règle que seul un humain contrôle finit par être oubliée.
- **En cas de doute,** suivre ce que font les crates officiels (Axum, SQLx, Tokio) : le borrow checker et `clippy` sont des relecteurs de code, pas des adversaires.
- **Le compilateur est le premier test.** Si ça compile sans `unwrap` ni `clone` déraisonnable, c'est probablement juste.

---

## 2. Langue

| Élément | Langue | Exemple |
|---|---|---|
| Identifiants (types, fonctions, variables, modules) | anglais | `PracticeSession`, `submit_attempt`, `is_owner` |
| Noms de tests | anglais | `review_first_success_schedules_review_tomorrow` |
| Messages d'erreur techniques (logs, `Display` des erreurs) | anglais | `"session {session_id} is already finished"` |
| Commentaires | français | `// SM-2 impose un plancher à 1,3 : en dessous, une carte ne remonte plus.` |
| Documentation (`docs/`, ce dépôt) | français | — |

**L'avantage Rust du projet :** la base est en `snake_case` (`DOMAIN_MODEL.md`) et **les champs Rust sont aussi en `snake_case`**. Un même concept garde donc le même nom en SQL et en Rust, sans transformation :

| SQL | Rust |
|---|---|
| `practice_session` | `struct PracticeSession` |
| `srs_state.ease_factor` | `SrsState.ease_factor` |
| `elo_rating.game_count` | `EloRating.game_count` |
| `account.archived_at` | `Account.archived_at` |

On n'invente pas de synonyme (`User` pour `account`, `Review` pour `rating`) : le glossaire est celui de `DOMAIN_MODEL.md`. Les suffixes de relation sont conservés : `assigned_by_account_id` reste `assigned_by_account_id`.

---

## 3. Mise en forme

- **`rustfmt` partout** : le style n'est jamais discuté, il est appliqué. On ne reformate jamais à la main.
- **Longueur de ligne :** 100 caractères (`rustfmt.toml`).
- **Indentation :** 4 espaces, jamais de tabulation.
- **Fins de ligne :** LF (`.gitattributes` le garantit, voir `ARCHITECTURE.md` ; développement sous Windows, conteneurs Linux).
- **Accolades ouvrantes sur la même ligne** (style K&R de Rust) — c'est le style du langage, on ne le configure pas.
- **`cargo fmt` avant chaque commit** ; la CI vérifie (`cargo fmt --check`).
- **Un type par fichier**, nom du fichier = nom du type en `snake_case` (`practice_session.rs`). Exception tolérée : un petit module de contrats (requête + réponse d'un même endpoint) dans `STYLE_API.md`.
- **Une ligne vide** entre deux items ; jamais deux lignes vides de suite.
- **Appels longs :** un argument par ligne, la parenthèse fermante sur la dernière ligne d'argument.

  ```rust
  let result = practice_service
      .submit_attempt(current_account_id, session_id, request.into_command())
      .await?;
  ```

---

## 4. Nommage

| Élément | Convention | Exemple |
|---|---|---|
| Types (structs, enums, traits) | `PascalCase` | `EloCalculator`, `SrsStatus` |
| Fonctions, méthodes, variables, champs | `snake_case` | `expected_score()`, `due_at` |
| Modules et fichiers | `snake_case` | `elo`, `srs_state.rs` |
| Constantes et variables statiques | `SCREAMING_SNAKE_CASE` | `const MINIMUM_EASE_FACTOR: f32 = 1.3;` |
| Paramètres génériques | nom court et parlant | `T`, `Conn` |
| Paramètres de durée de vie | `'a`, ou nommé court s'il y en a plusieurs | `'conn` |

Règles complémentaires :

- **Pas de préfixe ni de notation hongroise** : ni `strName`, ni `mCount`.
- **Booléens** formulés comme une affirmation : `is_archived`, `has_verified_email`, `can_manage_group`, pas `archived` ni `flag`.
- **Fonctions** nommées par un verbe ; dans le Domain, le verbe du métier : `accept()`, `archive()`, `submit_for_review()`, `review()`, plutôt que `set_status()`.
- **Enums au singulier** (`QuestionType`, `SrsStatus`), membres en `PascalCase` (`SingleChoice`, `Relearning`) — correspondance avec les valeurs SQL en § 8.
- **Unités dans le nom** quand le type ne les porte pas : `duration_ms`, `interval_days` (comme en SQL, même nom).
- **Abréviations :** aucune sauf les très courantes (`id`, `url`, `dto`) ; au-delà de 2 lettres, on écrit le mot (`elo`, `srs` sont des termes métier assumés du projet).
- **Les conversions suivent `From`/`Into`** quand elles sont infaillibles (`impl From<CreateExerciseRequest> for CreateExerciseCommand`), une méthode `into_command()` ou `to_response()` quand elles peuvent échouer ou perdre de l'information.

### Suffixes et préfixes par rôle

| Rôle | Suffixe | Exemple |
|---|---|---|
| Moteur / service applicatif | `Service` | `PracticeService` |
| Contrat d'accès aux données (trait) | `Repository` | `PracticeRepository` |
| Implémentation SQLx | `Sql` en préfixe | `SqlPracticeRepository` |
| Handler Axum | ressource au pluriel + `Handler` | `PracticeSessionsHandler` |
| DTO entrant / sortant | `Request` / `Response` | `SubmitAttemptRequest`, `AttemptResponse` |
| Paramètre d'un cas d'usage | `Command` (écriture) / `Query` (lecture) | `SubmitAttemptCommand` |
| Erreur métier | `Error` | `DomainError`, `AuthError` |
| Configuration | `Config` | `JwtConfig` |
| Type identifiant (newtype) | le nom du concept | `AccountId(i64)`, `TagId(i64)` |

---

## 5. Organisation d'un fichier et d'un module

### Modules

- **Pas de `mod.rs`** : un dossier `practice/` contient `practice/mod`... non — la convention 2018 : `practice.rs` à côté du dossier `practice/` qui déclare le contenu. On choisit cette forme ; elle distingue fichiers de modules dans l'arborescence.
- Le chemin du module suit le chemin du dossier (`api::handlers::practice_sessions`).
- Visibilité : `pub` seulement à la frontière du module ; `pub(crate)` quand c'est interne au binaire ; sinon, privé par défaut. Un module ne révèle que ce dont les autres ont besoin.

### Intérieur d'un fichier

Ordre : `use`, puis types et fonctions dans cet ordre, avec un bloc de commentaire par section **présente** (on n'écrit pas de section vide) :

```rust
// ============================================================
// Champs (constantes, statics)
// ============================================================

// ============================================================
// Constructeurs et fabriques
// ============================================================

// ============================================================
// Accesseurs (méthodes publiques de lecture)
// ============================================================

// ============================================================
// Méthodes publiques
// ============================================================

// ============================================================
// Méthodes privées
// ============================================================

// ============================================================
// Tests
// ============================================================
```

- Les **fabriques** (`Friendship::request(...)`, `SrsState::new(...)`) vont avec les constructeurs : elles en jouent le rôle.
- Une ligne vide avant et après chaque bloc.
- Pas de repliement magique : **si un fichier a besoin d'être replié pour être lu, il fait probablement trop de choses** ; le découper en sous-modules.

### Extraire ou garder en ligne ?

Le critère n'est pas « combien de fois c'est appelé », mais : **est-ce un concept qu'on peut nommer clairement ?** Une fonction privée à usage unique se justifie si son nom rend l'appelant plus lisible. Si le nom ne dit rien de plus que le code, on garde le code en ligne.

---

## 6. Propriété, clonage et types

**Règle officielle du projet (V1) : avancer d'abord, optimiser ensuite.**

- **Cloner sans culpabilité quand le compilateur bloque.** Pour une application web classique, l'impact de `Arc`, `clone()` ou `to_owned()` est invisible face au temps réseau. Se battre avec les durées de vie dès le début est le piège n° 2 du `ABOUT_PROJECT.md` § 4.
- **Le borrow checker a raison.** S'il refuse, c'est presque toujours que l'état est partagé d'une manière qui mérite d'être explicite : `Arc`, passer par la base, ou réorganiser le code.
- **Interdit malgré tout :** `unsafe` (aucune justification dans ce projet), `Rc<RefCell<…>>` en async (non `Send`), `static mut`.

### Choisir le bon type

| Besoin | Choix |
|---|---|
| Identifiant (identité en base) | **newtype** : `struct AccountId(i64);` — empêche de confondre un `AccountId` et un `QuestionId` |
| Entité du Domain (état qui évolue) | `struct` à champs privés + méthodes métier (§ 8) |
| DTO, commande, requête | `struct` (avec `serde::Deserialize`/`Serialize`) |
| Valeur bornée (`ScoreRatio` 0–1) | newtype dont le constructeur **valide** (`ScoreRatio::new(0.85)?`) |
| Erreur métier | `enum` avec `thiserror` (§ 10) |
| Collection en sortie | `Vec<T>` ; jamais exposer un `&mut Vec` interne |
| Instant | `time::OffsetDateTime` (UTC) ; `chrono` accepté si préféré, mais **un seul des deux crates dans le projet** |

- **Newtypes pour tout ce qui se confond.** Leur coût (zéro à l'exécution) est nul face au bénéfice : `fn review(account_id: AccountId, question_id: QuestionId)` ne peut pas recevoir les arguments dans le mauvais ordre — le piège classique des `i64` nus.
- **Enums exhaustives.** `QuestionType`, `SrsStatus`, `SessionType`, `PublicationStatus` sont des enums Rust, et le `match` **doit** couvrir tous les cas sans `_ =>` fourre-tout : ajouter un cas en base casse la compilation, pas la production.
- **`match` et `if let` plutôt qu'une cascade de `if`** sur une même valeur.

---

## 7. Option : la nullabilité

L'`Option<T>` reflète la colonne SQL : `Null` vide dans `DOMAIN_MODEL.md` ⇒ non-`Option` ; `oui` ⇒ `Option<T>`.

```rust
pub username: String,              // username VARCHAR(50) NOT NULL
pub email: Option<String>,         // email VARCHAR(254) NULL
pub archived_at: Option<OffsetDateTime>,
```

- **Pas de `unwrap()` ni `expect()` dans le code de production**, sauf au démarrage (configuration) avec un message qui dit quoi corriger. `clippy` le vérifie (§ 17).
- **Pas de `unwrap_or_default()` silencieux** pour faire taire le compilateur : si `None` est impossible, restructurer ; s'il est possible, le gérer.
- Une fonction qui peut ne rien trouver le dit dans sa signature : `async fn find_by_id(id: QuestionId) -> sqlx::Result<Option<Question>>`.
- **La garde est aux frontières** : la désérialisation serde (DTO), l'extracteur Axum, et les newtypes validants (`ScoreRatio::new`). Après la frontière, le code métier manipule des types sûrs.
- Un `Option` de la base n'arrive jamais jusqu'au JSON sans avoir été transformé en champ JSON explicite (`null` voulu ou valeur par défaut documentée dans `STYLE_API.md`).

---

## 8. Entités du Domain

Une entité protège ses propres règles : aucun code extérieur ne doit pouvoir la mettre dans un état incohérent.

- **Champs privés.** L'état change par des méthodes métier, jamais par affectation depuis l'extérieur (pas de `pub` sur les champs).
- **Fabriques qui vérifient les règles** : `Friendship::request(...)` renvoie `Result<Friendship, DomainError>`. Le « constructeur » qui saute les règles n'existe pas (SQLx passe par un type de chargement à part, voir § 12).
- **L'heure est un paramètre** (`now: OffsetDateTime`) : le Domain n'appelle jamais l'horloge réelle. C'est l'`Application` qui la fournit (`TimeProvider` — un injectable pour les tests).
- **Toujours UTC** pour un instant (`TIMESTAMPTZ`), `Date` pour une date sans heure (`birth_date`).
- **Les règles d'une ligne** (statut, dates cohérentes) vivent dans l'entité **et** dans un `CHECK` SQL ; celles qui concernent plusieurs entités vont dans le service applicatif (`ARCHITECTURE.md`).

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FriendshipError {
    #[error("an account cannot befriend itself")]
    SelfRequest,
    #[error("friendship is not pending")]
    NotPending,
    #[error("only the recipient can accept a friendship")]
    RequesterCannotAccept,
}

pub struct Friendship {
    lower_account_id: AccountId,
    higher_account_id: AccountId,
    requester_account_id: AccountId,
    status: FriendshipStatus,
    requested_at: OffsetDateTime,
    accepted_at: Option<OffsetDateTime>,
}

impl Friendship {
    pub fn request(
        requester: AccountId,
        recipient: AccountId,
        now: OffsetDateTime,
    ) -> Result<Self, FriendshipError> {
        if requester == recipient {
            return Err(FriendshipError::SelfRequest);
        }

        let (lower, higher) = if requester < recipient {
            (requester, recipient)
        } else {
            (recipient, requester)
        };

        Ok(Self {
            lower_account_id: lower,
            higher_account_id: higher,
            requester_account_id: requester,
            status: FriendshipStatus::Pending,
            requested_at: now,
            accepted_at: None,
        })
    }

    pub fn accept(&mut self, accepter: AccountId, now: OffsetDateTime) -> Result<(), FriendshipError> {
        if self.status != FriendshipStatus::Pending {
            return Err(FriendshipError::NotPending);
        }
        if accepter == self.requester_account_id {
            return Err(FriendshipError::RequesterCannotAccept);
        }

        self.status = FriendshipStatus::Accepted;
        self.accepted_at = Some(now);
        Ok(())
    }
}
```

### Énumérations ↔ SQL

Les valeurs SQL sont en `snake_case` (`'single_choice'`, `'relearning'`), les membres Rust en `PascalCase` (`SingleChoice`, `Relearning`). La correspondance se fait **au même endroit** (implémentation `FromStr`/`Display` ou convertisseur SQLx dans l'`Infrastructure`), jamais avec des `to_string()` dispersés. On ne s'appuie **jamais** sur la valeur ordinale : elle n'est stockée nulle part.

```rust
impl TryFrom<&str> for QuestionType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "flashcard" => Ok(Self::Flashcard),
            "single_choice" => Ok(Self::SingleChoice),
            // ... tous les cas, aucun _ => fourre-tout
            other => Err(format!("unknown question_type: {other}")),
        }
    }
}
```

---

## 9. Commentaires

- **Pas de rustdoc (`///`) qui répète le nom.** `/// Returns the due date` sur `pub fn due_at()` n'apporte rien. Un `///` documente un contrat non évident (invariants, paniques, garanties) sur l'API publique du crate ; le reste du code n'en a pas besoin.
- Un commentaire explique le **pourquoi**, jamais le **quoi** :

  ```rust
  // À éviter : redit le code
  // Met l'intervalle à 1
  state.interval_days = 1;

  // Préféré : explique une décision
  // Un oubli relance l'apprentissage : SM-2 repart d'un jour, quel que soit l'intervalle atteint.
  state.interval_days = 1;
  ```

- **Référencer la source** d'un algorithme ou d'une règle externe (formule SM-2, article RGPD) plutôt que de la recopier.
- **TODO avec un ticket :** `// TODO(#42) : versionner la question avant d'ouvrir les contributions.` Un TODO sans ticket est une note perdue.
- **Pas de code commenté :** Git garde l'historique.

---

## 10. Erreurs : Result et paniques

| Situation | Traitement |
|---|---|
| Argument invalide (bug de l'appelant) | type invalide à la compilation (newtype) ou `Result` du Domain |
| Règle métier violée (accepter une amitié déjà acceptée) | `DomainError` (enum `thiserror`) avec un **code d'erreur stable** |
| Ressource absente, cas attendu | `Ok(None)` (signature `Option`), **pas d'erreur** |
| Droit insuffisant | erreur typée de l'`Application` (`Forbidden`) |
| Erreur technique (base injoignable) | on laisse remonter (`?` sur `sqlx::Error`) |
| Configuration absente au démarrage | `expect()` avec message de correction — le seul `expect` autorisé |

- **Panique interdite en production** : pas de `unwrap`, pas d'indexation non vérifiée, pas d'arithmétique `debug_assert` dépendante. `clippy` et les tests couvrent ça.
- **`?` partout où l'erreur se propage** ; on n'imbrique pas de `match` sur `Result` pour reformer une `Option`.
- **Un crate d'erreurs métier distinct** (`thiserror`) ; **pas d'`anyhow`** dans le code métier — l'erreur non typée masque les cas à traiter. `anyhow` est toléré seulement dans un petit `main`/bootstrap.
- **Le middleware global** est le seul endroit qui attrape tout : il journalise et convertit en réponse d'erreur normalisée (`STYLE_API.md`, RFC 9457 + code stable). Ailleurs, on n'attrape pas ce qu'on ne sait pas traiter.
- On n'avale jamais une erreur : ignorer volontairement un échec (ex. notification non bloquante) s'écrit explicitement (`if let Err(e) = send_notification(...).await { tracing::warn!(...) }`), jamais par un `_ =`.
- **Codes d'erreur stables** (`"friendship.not_pending"`) regroupés par domaine et renvoyés dans la réponse API. Le front affiche son propre texte en français à partir du code : le `Display` de l'erreur reste technique et en anglais.

---

## 11. Async : Tokio

- **`#[tokio::main]`** une fois dans `main.rs` ; Axum et SQLx tournent sur ce runtime. **Pas de `async` dans le Domain** : les moteurs SM-2 et Elo sont des fonctions synchrones et pures, testées sans runtime.
- **Jamais d'opération bloquante dans l'async** : `std::fs`, `std::thread::sleep`, hachage Argon2 long → soit `tokio::fs`, soit `tokio::task::spawn_blocking`. Un blocage gèle tout le runtime : c'est l'équivalent Rust du deadlock `lock` en WinForms.
- **Pas de `async fn` dans les traits** maison pour l'instant : les repositories se modélisent en **fonctions synchrones autour d'un pool/transaction passé en paramètre** (SQLx 0.7) ou avec `#[async_trait]` si l'indirection est vraiment nécessaire (une seule approche pour tout le projet).
- **Timeouts explicites** sur tout appel externe (base : configuration du pool ; HTTP sortant : `tokio::time::timeout`).
- **Pas de `spawn` sans raison** : une tâche détachée doit être justifiée (elle vit après la requête) et journalisée ; sinon, `await` simple.
- **L'annulation** : Axum annule le futur quand le client ferme la connexion ; les opérations critiques (écriture d'un `attempt`) sont idempotentes ou vérifient leur effet — on ne suppose jamais qu'un `await` arrive au bout.

---

## 12. SQLx et PostgreSQL

- **Requêtes vérifiées à la compilation** avec les macros `query!`/`query_as!` quand c'est possible. Le mode hors ligne (`cargo sqlx prepare`) est obligatoire dès le premier commit : **la CI n'a pas de base**.
- **`query_as` avec `FromRow`** pour les lectures ; un struct de chargement (`SrsStateRow`) distinct de l'entité du Domain si les champs diffèrent (privés) — la conversion se fait par une fabrique `try_into_domain()`.
- **Transactions : une par cas d'usage.** Le service ouvre `pool.begin()` et le passe (ou l'objet `Transaction`) aux repositories ; commit en fin de cas d'usage. Écrire SM-2 **et** les Elo **et** l'historique dans la même transaction (§ 3.1 du CDC).
- **`fetch_one` vs `fetch_optional`** : l'absence possible se lit dans le choix — jamais `fetch_one` suivi d'un `match` sur l'erreur `RowNotFound`.
- **Projeter en SQL plutôt que charger pour filtrer en mémoire** ; attention au N+1 (une requête dans une boucle = signal d'alarme ; charger par lot `WHERE id = ANY($1)`).
- **Construire les chaînes dynamiques uniquement via SQLx** : `QueryBuilder` pour les filtres, jamais de concaténation de chaîne pour du SQL.
- **Le pool ne sort jamais des handlers** : les repositories le reçoivent en paramètre ou vivent dans l'état applicatif (§ 13).
- **Migrations** : dossiers `migrations/V000X__description.sql`, générés par `sqlx migrate add`, alignés avec `STYLE_SQL.md`. **Une migration appliquée ne se modifie plus jamais.**

---

## 13. État partagé et configuration (Axum)

- **Un `AppState` unique** passé à `Router::with_state` : le pool SQLx, la configuration, les services. Pas de globale mutable, pas de `lazy_static` d'état applicatif.

  ```rust
  #[derive(Clone)]
  pub struct AppState {
      pub pool: PgPool,
      pub config: AppConfig,
      pub practice_service: PracticeService,
  }
  ```

- `Arc` interne aux services qui en ont besoin ; cloner l'`AppState` par requête est le modèle attendu d'Axum (peu coûteux).
- **Chaque couche s'enregistre elle-même** : `PracticeService::new(pool, config)` construit dans une fonction `build_state()` ; `main.rs` reste une liste lisible d'appels.
- **Configuration typée** désérialisée depuis l'environnement (`serde` + envy, ou figment) :

  ```rust
  #[derive(Deserialize, Clone)]
  pub struct JwtConfig {
      pub jwt_secret: String,   // variable d'environnement uniquement
      pub jwt_ttl_minutes: u64,
  }
  ```

  **Validée au démarrage** (`expect` avec message) : une clé manquante fait échouer le démarrage, pas la première connexion d'un utilisateur.
- **Aucun secret dans le code, dans `Cargo.toml` ni dans le compose commité** : `.env` ignoré par Git, `.env.example` versionné (`ARCHITECTURE.md` § 9).

---

## 14. Handlers et DTO

- Handlers **fins** : extraire la requête, appeler un service, traduire le résultat en HTTP. Aucune règle métier, aucun SQL.
- **Extracteurs Axum dans l'ordre** (état, chemins, requête) ; le compte courant vient d'un extracteur `CurrentUser` construit par le middleware d'authentification.
- **Jamais d'entité du Domain dans une réponse JSON** : toujours un DTO.
- **DTO = struct serde**, conversions explicites (`impl From<Attempt> for AttemptResponse`, `impl TryFrom<&SubmitAttemptRequest> for SubmitAttemptCommand`) — pas de bibliothèque de mapping automatique : une conversion explicite se lit, se débogue et casse à la compilation.
- **Clés JSON en `snake_case`** (par défaut serde) : les clés API portent les mêmes noms que les colonnes SQL (`due_at`, `pass_threshold_ratio`), détail dans `STYLE_API.md`.
- **Validation des DTO entrants** (`validator` ou newtypes) avant d'entrer dans le métier ; l'erreur de validation devient une réponse 422 normalisée.

```rust
pub async fn submit_attempt(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(session_id): Path<i64>,
    Json(request): Json<SubmitAttemptRequest>,
) -> Result<(StatusCode, Json<AttemptResponse>), ApiError> {
    let attempt = state
        .practice_service
        .submit_attempt(current_user.account_id, session_id, request.try_into()?)
        .await?;

    Ok((StatusCode::CREATED, Json(attempt.into())))
}
```

- Le type `ApiError` centralise la traduction métier → HTTP (404, 403, 409, 422, 500) selon `STYLE_API.md` ; le handler n'en sait rien.

---

## 15. Journalisation et données personnelles

- **`tracing`** avec spans et champs nommés, jamais de formatage de chaîne :

  ```rust
  // À éviter : texte figé, impossible à filtrer
  tracing::info!("Attempt recorded for session {session_id}");

  // Préféré : session_id devient un champ interrogeable
  tracing::info!(session_id = session_id, "Attempt recorded");
  ```

- **Niveaux :** `debug` diagnostic, `info` événement métier notable, `warn` anomalie gérée, `error` échec de requête, le `error` du runtime pour la panne.
- **Jamais dans les logs :** mot de passe, hash, jeton JWT, poivre, chaîne de connexion. **Pas de données personnelles** non plus (email, nom, date de naissance) : on journalise l'`account_id`. Le public visé est jeune et les logs sont rarement protégés comme la base (RGPD).
- Comparer des secrets en temps constant : `subtle::ConstantTimeEq`, jamais `==` sur des octets.

---

## 16. Tests

Organisation et outils : `ARCHITECTURE.md` § 8. Répartition : tests unitaires `#[cfg(test)]` dans le même fichier ; tests d'intégration dans `tests/` (API + PostgreSQL éphémère via testcontainers).

- **Nommage :** `méthode_scénario_résultat_attendu` en `snake_case`, anglais.

  ```rust
  review_first_success_schedules_review_tomorrow
  accept_by_requester_returns_err
  submit_attempt_session_of_another_account_is_forbidden
  ```

- **Arrange / Act / Assert séparés par une ligne vide**, sans commentaire `// Arrange` : la structure se voit.
- **Un comportement par test.** Plusieurs assertions sont permises si elles vérifient ce même comportement.
- **Pas d'horloge réelle, pas d'aléatoire non maîtrisé** : l'heure est un paramètre (`now`), les échantillons sont constants ou générés avec graine fixe.
- **Faux écrits à la main** (repository en mémoire implémentant le trait) ; testcontainers pour la base réelle. Pas de bibliothèque de simulation au début.
- **Les moteurs SM-2 et Elo ont la couverture la plus stricte** : tableaux de cas (`#[test]` multiples plutôt qu'un test paramétrique caché), propriétés (bornes du `ease_factor` ≥ 1,3, `score_ratio` ∈ [0, 1], historique complet).

```rust
#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    #[test]
    fn review_first_success_schedules_review_tomorrow() {
        let now = OffsetDateTime::from_unix_timestamp(1_773_000_000).unwrap();
        let mut state = SrsState::new(account_id(1), question_id(1));

        state.review(5, now);

        assert_eq!(state.due_at(), Some(now + time::Duration::days(1)));
    }

    #[test]
    fn review_failure_resets_repetition_count() {
        let now = OffsetDateTime::from_unix_timestamp(1_773_000_000).unwrap();
        let mut state = SrsState::new(account_id(1), question_id(1));
        state.review(5, now);

        state.review(2, now + time::Duration::days(1));

        assert_eq!(state.repetition_count(), 0);
    }
}
```

- **`unwrap()` autorisé dans les tests** (et seulement là) : les fixtures l'utilisent librement.

---

## 17. Outillage

Trois fichiers rendent ces règles automatiques, quel que soit l'éditeur.

### `rust-toolchain.toml`

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

### `rustfmt.toml`

```toml
max_width = 100
```

Le reste : valeurs par défaut de `rustfmt` — c'est le point entier de l'outil (une seule façon de faire).

### `.cargo/config.toml` (à la racine du dépôt)

```toml
[alias]
lint = "clippy --all-targets -- -D warnings"
check-all = "clippy --all-targets -- -D warnings && fmt --check"
```

### CI (`back.yml`)

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo sqlx prepare --check      # requêtes hors ligne à jour
cargo test                      # + service PostgreSQL pour l'intégration
cargo build --release
cargo audit                     # dépendances vulnérables
cargo deny check licenses       # licences : critère « Libéré » du CDC § 2.1
```

- `cargo audit` et `cargo deny` (double licence) protègent les critères *Sécurisé* et *Libéré* du `ABOUT_PROJECT.md` § 2.1 : une dépendance non libre ou vulnérable casse la pipeline, pas la production.
- En local : `cargo fmt` puis `cargo lint` avant chaque commit ; `cargo watch -x test` pendant le développement.

---

## 18. Correspondance avec le guide C#

Pour qui vient du guide C# (`STYLE_CSHARPE_GUIDE.md`) : les principes sont identiques, seule la mécanique change.

| Concept C# | Équivalent Rust retenu |
|---|---|
| `string?` / nullabilité NRT | `Option<T>` — l'absence est dans le type, pas une annotation |
| `sealed class` par défaut | structs à champs privés ; pas d'héritage — composition et traits |
| `record` pour DTO | struct + `derive(Serialize, Deserialize)` |
| Exceptions métier (`DomainException`) | enums `thiserror` + `?` — l'erreur fait partie de la signature |
| `async`/`await`, `CancellationToken` | Tokio ; annulation par drop du futur |
| LINQ (`Where`, `Select`) | itérateurs (`iter().filter().map()`) |
| EF Core (`DbContext`, `Include`) | SQLx : requêtes SQL explicites vérifiées à la compilation |
| Injection de dépendances (DI) | `AppState` Axum construit explicitement — pas de conteneur magique |
| `TimeProvider` pour l'heure | paramètre `now: OffsetDateTime` dans le Domain |
| `ILogger` structuré | `tracing` à champs nommés |
| xUnit, `[Fact]`, `[Theory]` | `cargo test`, `#[test]`, tableaux de cas |
| `.editorconfig` + `dotnet format` | `rustfmt.toml` + `cargo clippy -D warnings` |
| `ConfigureAwait(false)` (interdit ici aussi) | sans objet : pas de contexte de synchronisation |
| `nameof`, réflexion | sans objet : macros, `#[derive]`, types statiques |

**Ce qui change vraiment :** l'ordre d'exécution. En C#, on découvre l'incohérence à l'exécution ; en Rust, le compilateur la refuse à la compilation. Écrire « pour contenter le compilateur » n'est pas une perte de temps : c'est le test qui se joue avant même `cargo test`.
