# Guide de style API — conventions REST

> **Portée :** toute l'API HTTP exposée par le backend aux clients (front React, futurs partenaires).
> **Documents liés :** `STYLE_RUST.md` (handlers, DTO, `ApiError`), `DOMAIN_MODEL.md` (ressources et vocabulaire), `ARCHITECTURE.md` (couches), `ABOUT_PROJECT.md` § 2.5 (sécurité).
> **Base :** REST sur HTTP JSON, RFC 9457 (Problem Details) pour les erreurs, RFC 9110 pour les codes de statut.

## Sommaire

1. [Principes](#1-principes)
2. [Base et versionnement](#2-base-et-versionnement)
3. [Ressources et nommage](#3-ressources-et-nommage)
4. [Méthodes et sémantique](#4-méthodes-et-sémantique)
5. [Corps et conventions de champs](#5-corps-et-conventions-de-champs)
6. [Codes de statut](#6-codes-de-statut)
7. [Format d'erreur](#7-format-derreur)
8. [Authentification et autorisation](#8-authentification-et-autorisation)
9. [Collections : pagination, tri, filtrage](#9-collections--pagination-tri-filtrage)
10. [Idempotence et concurrence](#10-idempotence-et-concurrence)
11. [En-têtes et CORS](#11-en-têtes-et-cors)
12. [Documentation OpenAPI](#12-documentation-openapi)
13. [Exemple complet](#13-exemple-complet)

---

## 1. Principes

- **L'URL nomme des ressources, les méthodes font les actions.** Jamais de verbe dans l'URL (`/api/v1/questions` et pas `/api/v1/getQuestions`).
- **Une seule convention de nommage pour tout le projet.** Ce guide est appliqué mécaniquement, jamais négocié endpoint par endpoint.
- **Le contrat est public même si le consommateur est notre propre front.** Les deux tiers vivent dans le même dépôt, mais le front ne lit jamais le code du back : il lit ce guide.
- **Ce qui change fait une nouvelle version, pas un cas particulier.** Un champ ajouté est rétrocompatible ; un champ renommé ou supprimé passe par une nouvelle version d'endpoint (§ 2).

---

## 2. Base et versionnement

- Toutes les routes sont préfixées : **`/api/v1/…`**.
- La version est **dans l'URL** (pas dans un en-tête) : simple à tester au navigateur, à logger, à mettre en cache.
- Tant que la V1 est la seule version, les routes restent sous `/api/v1/`. Le jour où une modification casse le contrat : `/api/v2/…` en parallèle, V1 dépréciée avec en-tête `Deprecation` et `Sunset` (RFC 8594).
- **Les champs ajoutés ne montent pas la version** : les clients ignorent les champs inconnus.

---

## 3. Ressources et nommage

- **Pluriel, `kebab-case`, anglais** — aligné sur les tables (`DOMAIN_MODEL.md`) :

| Table | Ressource |
|---|---|
| `account` | `/api/v1/accounts` |
| `study_group` | `/api/v1/study-groups` |
| `exercise` | `/api/v1/exercises` |
| `question` | `/api/v1/questions` |
| `assignment` | `/api/v1/assignments` |
| `practice_session` | `/api/v1/practice-sessions` |

- Imbrication uniquement quand la ressource fille **n'existe pas hors de son parent** :
  - `/api/v1/study-groups/{id}/members` — un membre n'a de sens que dans son groupe ;
  - `/api/v1/practice-sessions/{id}/attempts` — une tentative n'existe que dans sa séance ;
  - mais `/api/v1/attempts?account_id=…` si un jour on liste toutes les tentatives d'un compte (pas d'imbrication profonde : jamais plus d'**un** niveau).
- **La relation devient un sous-chemin**, pas un paramètre : `/api/v1/questions/{id}/tags` plutôt que `POST /api/v1/questions/{id}` avec `{ "add_tag_ids": [...] }`.
- Pas de suffixe de format (`.json`), pas d'extension, pas de verbe.

---

## 4. Méthodes et sémantique

| Méthode | Sémantique | Idempotente | Exemple |
|---|---|---|---|
| `GET` | lecture, jamais d'effet de bord | oui | `GET /api/v1/practice-sessions/{id}` |
| `POST` | création, action non idempotente | non | `POST /api/v1/practice-sessions` |
| `PATCH` | modification partielle (champs fournis) | non | `PATCH /api/v1/exercises/{id}` |
| `PUT` | remplacement complet | oui | réservé aux rares cas (statut de publication : `PUT …/publication`) |
| `DELETE` | suppression (archivage ici) | oui | `DELETE /api/v1/study-groups/{id}/members/{account_id}` |

- Pas de `POST` fourre-tout pour les actions : si l'action est un changement d'état nommé du métier, elle devient un sous-chemin explicite : `POST /api/v1/friendships/{id}/acceptance` (création de l'acceptation), pas `POST …/accept`. Les états métier (`publication_status`, `member_status`) suivent le vocabulaire du `DOMAIN_MODEL.md`.
- **`GET` ne modifie jamais rien** — même « juste compter » un événement : le comptage d'une vue passe par un `POST` explicite.

---

## 5. Corps et conventions de champs

- **JSON uniquement** (`Content-Type: application/json`), UTF-8.
- **Clés en `snake_case`** — identiques aux colonnes SQL et aux champs Rust : `due_at`, `pass_threshold_ratio`, `recall_grade`. Un seul nom pour un concept, de la base jusqu'au composant React.
- **Identifiants : des chaînes.** Les clés sont des `BIGINT` en base ; le JSON les expose en `string` (`"account_id": "42"`). JavaScript ne distingue pas `Number.MAX_SAFE_INTEGER` des grands `BIGINT` : le piège est silencieux, la règle est absolue.
- **Dates : ISO 8601 UTC** (`2026-11-16T19:03:12Z`). Le client formate pour l'affichage ; l'API n'envoie jamais de date locale sans fuseau.
- **Ratio et scores : nombres décimaux** (`"score_ratio": 0.85`). Jamais de pourcentage, jamais de « note sur 20 » dans l'API (conversion d'affichage côté front).
- **Champs `null` explicites vs absents :** un champ absent = « non applicable » ; un champ `null` = « applicable mais inconnu/vide ». En pratique : on émet `null` quand la colonne SQL est `NULL`, on omet le champ quand il ne fait pas partie de la réponse. Pas de tableau vide pour dire « pas d'objet ».
- **DTO d'entrée et de sortie distincts** : jamais le même struct (`SubmitAttemptRequest` ≠ `AttemptResponse`, § `STYLE_RUST.md` 14).
- Champs horodatés standards sur les ressources mutables : `created_at`, `updated_at` ; `archived_at` présent (null ou date) sur les ressources archivables.

---

## 6. Codes de statut

| Code | Quand | Exemple métier |
|---|---|---|
| `200 OK` | lecture ou modification réussie | `GET`/`PATCH` |
| `201 Created` | création réussie, **avec `Location`** | nouvelle `practice_session` |
| `204 No Content` | succès sans corps | acceptation d'amitié, `DELETE` |
| `400 Bad Request` | JSON illisible, champs inattendus | payload non désérialisable |
| `401 Unauthorized` | pas ou plus authentifié | jeton absent/expiré, jamais « mot de passe faux » (voir § 8) |
| `403 Forbidden` | authentifié mais pas le droit | consulter les réponses d'un groupe dont on n'est pas propriétaire |
| `404 Not Found` | ressource inexistante **ou** que le compte n'a pas le droit de voir | une question `private` d'un autre auteur |
| `409 Conflict` | conflit d'état | accepter une amitié déjà acceptée |
| `422 Unprocessable Entity` | JSON valide, règles de validation en échec | `score_ratio` hors [0, 1] |
| `429 Too Many Requests` | rate limiting atteint | login répété (§ 8) |
| `500 Internal Server Error` | bug côté serveur — **jamais volontaire** | |

- **Ne jamais renvoyer `200` avec un corps d'erreur.**
- `404` plutôt que `403` quand l'existence même de la ressource est confidentielle (questions privées, groupes fermés) : on ne confirme pas l'existence à qui n'a pas le droit de la voir.

---

## 7. Format d'erreur

**RFC 9457 (Problem Details)**, une seule forme pour toutes les erreurs :

```json
{
  "type": "https://sigre.example/problems/friendship-not-pending",
  "title": "La demande d'amitié n'est pas en attente.",
  "status": 409,
  "detail": "Cette demande a déjà été acceptée ou refusée.",
  "code": "friendship.not_pending",
  "errors": [
    { "field": "due_at", "code": "out_of_range" }
  ]
}
```

- **`code` : le contrat avec le front.** Stable, en `snake_case` pointé, listé dans le dépôt (`DomainErrors` côté Rust) : `"friendship.not_pending"`, `"auth.invalid_credentials"`, `"exercise.already_published"`. Le front affiche son texte français à partir de ce code — le message `detail` est un texte de repli, jamais la clé.
- **`errors` : uniquement pour `422`** — la liste des champs en échec, machine à mapper sur un formulaire.
- Le `title`/`detail` sont en français (texte utilisateur de repli) ; le `type` pointera vers la documentation publique des codes quand elle existera.
- **Les `500` ne révèlent rien** : titre générique (« Erreur interne »), la cause va dans les logs `tracing` avec l'identifiant de requête, jamais dans la réponse.

---

## 8. Authentification et autorisation

- **`Authorization: Bearer <jwt>`** sur toute route non publique. Pas de jeton dans l'URL, jamais en `localStorage` (cookie ou mémoire front, voir `STYLE_REACT.md`).
- Jeton d'accès **courte durée** (minutes) ; **refresh token rotatif** : `POST /api/v1/auth/refresh` (l'ancien refresh est révoqué à chaque usage — rotation exigée par le CDC § 2.5).
- `POST /api/v1/auth/login` : identifiants invalides → **`401` + code `auth.invalid_credentials`**, sans dire lequel est faux ; réponse identique si le compte n'existe pas (on ne révèle pas l'existence d'un email).
- **Rate limiting** sur `login`, `refresh` et inscription (`429` + en-tête `Retry-After`).
- Les routes de compte (`/accounts/me`) et d'authentification sont les seules sous `/api/v1/auth` et `/api/v1/accounts` à ne pas suivre le pluriel-ressource strict — assumé, car `auth` est un espace de traitement, pas une ressource.
- **Le compte courant vient du jeton, jamais du corps** : pas de `account_id` en paramètre sur `/api/v1/accounts/me/attempts`. Un utilisateur ne demande que **ses** données ; le propriétaire de groupe consulte via les routes `/api/v1/study-groups/{id}/…` (contrôle de rôle côté serveur).

---

## 9. Collections : pagination, tri, filtrage

- **Toute collection est paginée dès le premier jour** (même si elle a 3 éléments) : changer un `Vec` en page cassera le front ; commencer paginé ne coûte rien.

```json
{
  "items": [ … ],
  "page": 1,
  "per_page": 20,
  "total": 137,
  "total_pages": 7
}
```

- Paramètres : `page` (≥ 1) et `per_page` (1–100, défaut 20, plafond absolu 100).
- **Tri :** `sort=<champ>` et `order=asc|desc`, ex. `?sort=due_at&order=asc`. Seuls les champs triables d'une collection sont documentés ; toute autre valeur → `422`.
- **Filtrage :** `?tag_id=…`, `?session_type=…`, bornes `?due_before=…&due_after=…`. Les noms de filtres sont les noms de colonnes SQL (`DOMAIN_MODEL.md`), pas des inventions.
- Les erreurs de pagination/tri/filtrage sont des `422` avec `errors[].field` rempli.

---

## 10. Idempotence et concurrence

- Les actions créatrices sensibles acceptent un **`Idempotency-Key`** (UUID fourni par le client, RFC_Idempotency) : `POST /practice-sessions` rejoué avec la même clé renvoie la réponse initiale au lieu de créer deux séances. Utile sur réseau mobile — notre cas d'usage principal.
- **Aucun compteur incrémental côté client** : la base est la seule source de vérité (`game_count`, `repetition_count` se lisent, jamais ne se postent).
- Optimistic locking inutile en V1 : les entités métier à écriture concurrente (Elo) sont mises à jour par le serveur seul, dans une transaction.

---

## 11. En-têtes et CORS

- Réponses : `Content-Type: application/json; charset=utf-8` systématique, `Location` sur les `201`, `X-Request-Id` (généré par le middleware, repris dans les logs `tracing` : c'est la corde qui relie une plainte utilisateur à sa trace).
- **CORS restreint** : origines autorisées = le front (et rien d'autre en production). Jamais `*` avec des routes authentifiées.
- **Cookies** : si le refresh token passe en cookie, il est `HttpOnly`, `Secure`, `SameSite=Lax`, chemin dédié `/api/v1/auth`.
- **Cache :** `GET` des contenus publics peut être mis en cache (`Cache-Control` explicite) ; **jamais de cache** sur une réponse authentifiée (`Cache-Control: no-store`).

---

## 12. Documentation OpenAPI

- Le schéma est **généré depuis le code** (`utoipa` sur les handlers et DTO Rust, servi sur `/api/v1/openapi.json`, UI en environnement de dev uniquement).
- Doc = code : un endpoint sans sa déclaration OpenAPI ne passe pas la revue — c'est vérifiable en CI.
- Chaque endpoint documente : réponses possibles **et leurs codes d'erreur métier** (`responses(409 = friendship.not_pending)`), authentification requise, pagination si collection.

---

## 13. Exemple complet

`POST /api/v1/practice-sessions/{session_id}/attempts` — soumettre une réponse (le cœur du système, `ABOUT_PROJECT.md` § 3.1) :

```http
POST /api/v1/practice-sessions/1042/attempts
Authorization: Bearer eyJhbGciOi…
Content-Type: application/json
Idempotency-Key: 7c9e6679-7425-40de-944b-e07fc1f90ae7

{
  "question_id": "88",
  "given_answer": "3,14",
  "score_ratio": 1.0,
  "recall_grade": 5,
  "duration_ms": 4200
}
```

Réponse `201 Created` :

```http
Location: /api/v1/practice-sessions/1042/attempts/55123

{
  "attempt_id": "55123",
  "question_id": "88",
  "score_ratio": 1.0,
  "recall_grade": 5,
  "duration_ms": 4200,
  "answered_at": "2026-11-16T19:03:12Z",
  "srs": {
    "due_at": "2026-11-17T19:03:12Z",
    "interval_days": 1,
    "ease_factor": 2.5
  },
  "elo_deltas": [
    { "tag_id": "12", "account_delta": 8.4, "question_delta": -8.4 }
  ]
}
```

Erreurs possibles (documentées dans OpenAPI) : `401 auth.token_expired`, `403 attempt.session_of_another_account`, `404` (séance ou question inexistante), `409 attempt.session_finished`, `422` (`score_ratio` hors [0,1], `recall_grade` hors 0–5).

Ce que cet exemple illustre : pluriel-kebab, imbrication à un niveau, identifiants en chaînes, clés `snake_case` alignées SQL, dates ISO UTC, `Location` sur 201, codes d'erreur stables, et la réponse qui **raconte l'effet métier** (où en est la carte SRS, ce que l'Elo a bougé) — c'est ce qui rend le front simple.
