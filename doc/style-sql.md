# Guide de style SQL — conventions PostgreSQL

> **Portée :** tout le SQL du projet — migrations, requêtes SQLx dans le code Rust, scripts d'exploitation.
> **Documents liés :** `V0001__initial_schema.sql` (exemple canonique, toutes les règles de ce guide y sont appliquées), `DOMAIN_MODEL.md` (vocabulaire métier), `STYLE_RUST.md` § 12 (SQLx), `ARCHITECTURE.md`.
> **Réécrit pour PostgreSQL 16** : l'ancien guide visait T-SQL / SQL Server. Les principes restent (un concept = un nom, contraintes nommées, mesurer avant d'optimiser) ; la syntaxe change. Une table de correspondance T-SQL → PostgreSQL clot ce document.

## Sommaire

1. [Principes](#1-principes)
2. [Mise en forme](#2-mise-en-forme)
3. [Nommage des objets](#3-nommage-des-objets)
4. [Contraintes et index](#4-contraintes-et-index)
5. [Types de données](#5-types-de-données)
6. [NULL et sémantique](#6-null-et-sémantique)
7. [Requêtes : jointures, CTE, agrégats](#7-requêtes--jointures-cte-agrégats)
8. [Migrations sqlx](#8-migrations-sqlx)
9. [Performance](#9-performance)
10. [Correspondance T-SQL → PostgreSQL](#10-correspondance-t-sql--postgresql)

---

## 1. Principes

- **Un concept = un nom, partout** : `due_at`, `recall_grade`, `study_group` portent le même nom de la colonne SQL au champ Rust au champ JSON (`STYLE_API.md` § 5). C'est la règle qui passe avant toutes les autres.
- **La base est la seule source de vérité** : toute règle qui peut être une contrainte (`CHECK`, `UNIQUE`, FK, arc exclusif) **est** une contrainte. Le code Rust valide pour l'ergonomie, jamais à la place de la base.
- **Toute requête est testée seule, avec des paramètres connus, avant d'être encapsulée** dans le code Rust ou une fonction SQL — puis comparée à un résultat de référence. (Principe conservé de l'ancien guide.)
- **Le schéma vit dans le dépôt**, jamais dans un outil graphique : `V0001__initial_schema.sql` est le canon, les migrations sont l'histoire (§ 8).

---

## 2. Mise en forme

- **Mots-clés en MAJUSCULES** : `SELECT`, `FROM`, `WHERE`, `INNER JOIN`, `CASE WHEN … END`. Identifiants en minuscules (PostgreSQL replie tout en minuscules de toute façon).
- **Indentation : 4 espaces**, une clause par ligne dès que la requête dépasse 2-3 lignes, virgule **en fin de ligne**.
- **Point-virgule en fin d'instruction, toujours.** En SQL interactif (`psql`), il protège du piège du `WITH` suivant une instruction ; dans les migrations, il délimite les instructions.
- **Une jointure par ligne**, et dans le `ON`, **la colonne de la table qu'on vient d'introduire en premier** — dans une longue chaîne de jointures, on voit immédiatement qui each ligne ajoute :

```sql
SELECT
    q.question_id,
    a.answer_id
FROM question q
    INNER JOIN answer a ON a.question_id = q.question_id
    INNER JOIN srs_state s ON s.question_id = q.question_id;
```

- **Alias courts mais non ambigus** : `q` pour `question`, `ss` pour `srs_state`, jamais la même lettre pour deux tables d'une même requête.
- Commentaires : `--` pour une ligne, et un commentaire explique **le pourquoi** (règle métier, référence au `DOMAIN_MODEL.md`), jamais le quoi. L'exemple canonique est l'en-tête de `V0001__initial_schema.sql` : objectif, hypothèses, références, marquage `[PG]` des syntaxes propres à PostgreSQL.

---

## 3. Nommage des objets

- **Un schéma applicatif : `app`** (pas de préfixe de table) ; `public` reste aux extensions PostgreSQL.

| Objet | Convention | Exemple |
|---|---|---|
| Table | singulier, `snake_case`, anglais | `study_group` |
| Table d'association | les deux entités, ordre de lecture naturel | `group_member`, `question_tag` |
| Vue | `vw_<concept>` | `vw_account_overview` |
| Fonction | verbe + objet | `set_updated_at()` |
| Colonne identifiant | `<entité>_id` | `account_id`, `question_id` |
| Colonne FK | même nom que la clé visée | `account_id` partout |
| Booléen | `is_` / `has_`, jamais nullable | `is_archived` |
| Date/heure | suffixe `_at` | `due_at`, `archived_at` |
| Date pure | suffixe `_date` | `birth_date` |
| Durée | unité dans le nom | `duration_ms` |
| Ratio/quantité bornée | suffixe `_ratio` | `pass_threshold_ratio` |

- **Singulier** (une ligne = une occurrence), **anglais** (cohérent avec Rust, JSON, React : `study_group` et pas `groupe`).
- **Pas d'abréviation hors glossaire** (`DOMAIN_MODEL.md` tient lieu de glossaire) ; pas d'accents ; pas de mots réservés (`user`, `order` : c'est `account` et `attempt`/`assignment` ici).
- Les codes à valeurs fermées (`account_type`, `session_type`, `publication_status`) sont des `VARCHAR` verrouillés par `CHECK` (et reflétés en enums Rust, `STYLE_RUST.md` § 6), documentés dans le commentaire de la colonne comme dans `V0001` :
  - `account_type` : `standard` / `anonymous` / `organization` — et le commentaire explique pourquoi « vérifié » n'est pas un type.

---

## 4. Contraintes et index

- **Toutes les contraintes sont nommées** (un `DROP CONSTRAINT` anonyme généré sur une base échoue sur une autre — la leçon de l'ancien guide reste) :

| Objet | Modèle | Exemple canonique |
|---|---|---|
| Clé primaire | `pk_<table>` | `pk_account` |
| Clé étrangère | `fk_<table>_<table_visée>` | `fk_attempt_question` |
| Contrôle | `ck_<table>_<colonne>` | `ck_account_account_type` |
| Unicité | `ux_<table>_<colonnes>` | `ux_tag_slug` |
| Index | `ix_<table>_<colonnes>` | `ix_srs_state_account_due_at` |

- **Unicité insensible à la casse** : index unique sur expression, pas de contrainte `UNIQUE` simple :

```sql
CREATE UNIQUE INDEX ux_account_username ON app.account (LOWER(username));
```

- **Unicité avec NULL** : `UNIQUE NULLS NOT DISTINCT` quand « une seule ligne porte ce couple, même si une des valeurs est NULL » (exigence PostgreSQL 15+ ; c'est le cas des colonnes de `tag` parentes facultatives dans `V0001`).
- **Index partiels** quand la contrainte ne concerne qu'un sous-ensemble (ex. index sur `archived_at IS NULL`) ; un index qui filtre la quasi-totalité des lignes ne sert qu'à faire scintiler `EXPLAIN`.
- **Arc exclusif = `CHECK` en `CASE`**, pas de vue ou de convention d'application (`elo_rating` vise `account_tag` **ou** `question_tag` **ou** `exercise_tag`, jamais deux) :

```sql
CONSTRAINT ck_elo_rating_arc CHECK
(
    (account_tag_id IS NOT NULL)::int
  + (question_tag_id IS NOT NULL)::int
  + (exercise_tag_id IS NOT NULL)::int = 1
)
```

- **`updated_at` maintenu par trigger** (`set_updated_at()`), quel que soit le client qui modifie la ligne — l'API Rust n'est pas seule à écrire.

---

## 5. Types de données

| Usage | Type | Exemple |
|---|---|---|
| Identifiant technique | `BIGINT GENERATED ALWAYS AS IDENTITY` | `account_id` |
| Instant | `TIMESTAMPTZ` (toujours avec fuseau) | `due_at`, `created_at` |
| Date pure | `DATE` | `birth_date` |
| Texte court borné | `VARCHAR(n)` | `username VARCHAR(50)` |
| Texte long | `TEXT` | `bio`, énoncé de question |
| Booléen | `BOOLEAN NOT NULL` | — |
| Ratio, score | `NUMERIC(4,3)` | `score_ratio` dans [0, 1] |
| Note SM-2 | `SMALLINT` + `CHECK (recall_grade BETWEEN 0 AND 5)` | `recall_grade` |
| Montant | `NUMERIC(12,2)` | (usage futur) |
| Code pays | `CHAR(2)` + clé naturelle | `country_code` |

- **`GENERATED ALWAYS`, pas `BY DEFAULT`** : l'application ne fournit jamais la clé, la base seule l'attribue (SQLx laisse la colonne hors du `INSERT`).
- **Jamais de durée en `TIME`** (leçon conservée de l'ancien guide : `TIME` est une heure de journée, pas une durée) : ici les durées sont en millisecondes entières (`duration_ms BIGINT`).
- **`NUMERIC` pour tout ce qui est compté, mesuré ou affiché** (score Elo, ratios) : pas de `FLOAT`/`DOUBLE PRECISION` pour des valeurs métier (arrondis silencieux interdits là où on calcule un niveau Elo ou un seuil de 17/20).
- Horodatages standards : `created_at`, `updated_at` en `NOT NULL DEFAULT CURRENT_TIMESTAMP` ; `archived_at TIMESTAMPTZ NULL` = suppression logique (jamais de `DELETE` physique sur les données pédagogiques, RGPD = anonymisation, voir `account`).

---

## 6. NULL et sémantique

- **`NULL` = « applicable mais inconnu/vide »** ; un champ non applicable n'existe pas (colonne absente, ou table séparée). `password_hash NULL` pour un compte anonyme, pas pour un compte standard (c'est le `CHECK ck_account_credentials`).
- **Deux `NULL` ne sont jamais égaux dans un `ON`/`WHERE`** — mais `UNIQUE NULLS NOT DISTINCT` (§ 4) et `ORDER BY` les traitent comme égaux. Choisir l'outil en connaissance de cause, et le commenter.
- **Jamais de chaîne vide pour dire NULL** (l'ancien `ISNULL(x, '') = ''` de T-SQL n'a pas lieu d'être ici : on modélise proprement, la base est neuve).
- Un `CHECK` laisse passer `NULL` : l'interdiction passe par `NOT NULL` — les deux vont ensemble (`recall_grade SMALLINT NOT NULL CHECK (…)`).

---

## 7. Requêtes : jointures, CTE, agrégats

- **Toute requête applicative vit dans le code Rust** via les macros SQLx (vérifiées à la compilation, `STYLE_RUST.md` § 12) ; le SQL ad hoc (exploration, correction de données) vit dans des scripts commentés, jamais dans l'historique de session.
- **Attention au fan-out des jointures 1:N** (leçon conservée) : joindre `answer` (N par question) multiplie les lignes de `question` ; un `SUM`/`COUNT` après cette jointure compte plusieurs fois la même chose. Avant d'agréger après une jointure, se demander explicitement si elle peut multiplier les lignes.
- **CTE nommés (`WITH … AS`)** pour les étapes intermédiaires réutilisées ou documentées ; sous-requête `EXISTS`/`NOT EXISTS` corrélée quand la condition ne sert qu'une fois au plus près de son application. Pas de nom générique (`t1`, `tmp`) : `due_questions`, `session_stats`.
- **`EXISTS` plutôt que `IN (SELECT …)`** sur les sous-requêtes à NULL possible ou volumétrie inconnue ; le planner de PostgreSQL gère les deux, `EXISTS` se lit mieux sur les gardes d'arc exclusif.
- **Fenêtrage quand la question est « le premier/le rang par groupe »** : `ROW_NUMBER() OVER (PARTITION BY account_id, question_id ORDER BY answered_at DESC)` pour « la dernière tentative par carte », plutôt qu'un auto-join tordu.
- **`SELECT` explicites, jamais `SELECT *`** applicatif : la macro SQLx ne mappe que ce qu'on demande, et un `*` casse silencieusement le mapping à la première colonne ajoutée.

---

## 8. Migrations sqlx

- **Dossier `migrations/`**, fichiers `V000X__<slug>.sql` (c'est `V0001__initial_schema.sql` qui fixe le style : en-tête complet, sections numérotées alignées sur `DOMAIN_MODEL.md`).
- **Une migration appliquée n'est jamais modifiée.** Une erreur se corrige par une nouvelle migration `V000X+1`. Modifier un fichier déjà appliqué désynchronise toutes les bases (dev, CI, prod) de manière irrécouvrable sans reset.
- **Chaque migration est une unité atomique** : le DDL et ses index/contraintes/triggers dans le même fichier ; sqlx l'applique dans une transaction.
- **Les migrations sont relues comme du code** : en-tête objectif + hypothèses, `-- [PG]` sur les syntaxes spécifiques, commentaires sur les `CHECK` métier.
- **`cargo sqlx prepare`** régénère le cache hors ligne après toute modification de requête ; `sqlx prepare --check` en CI garantit que le cache suit le code (`STYLE_RUST.md` § 16).
- En dev, `sqlx migrate revert` existe pour itérer ; en prod, on avance, on ne recule jamais (§ 8, première règle).

---

## 9. Performance

- **`EXPLAIN (ANALYZE, BUFFERS)` avant tout jugement** : mesurer avant d'optimiser (leçon conservée de l'ancien guide — la mémoïsation n'y a lieu qu'après comptage des combinaisons distinctes ; ici, l'équivalent est l'index qui sert vraiment).
- Les index du canon servent les requêtes chaudes connues : file due (`ix_srs_state_account_due_at` partiel sur `due_at <= now()` si besoin), historique par compte (`ix_attempt_account_answered_at`). On n'indexe pas par anticipation : un index inutile coûte à chaque écriture — et `attempt` est la table la plus écrite du système.
- **Le paging est en base** (`LIMIT …/ OFFSET …` ou keyset) : jamais « je charge tout et je filtre en Rust » (et le contrat impose la pagination dès le premier jour, `STYLE_API.md` § 9).
- Pas de vues matérialisées en V1 : les volumes ne les justifient pas ; le jour où un tableau de bord agrège lourdement, on mesurera d'abord.

---

## 10. Correspondance T-SQL → PostgreSQL

Pour la transition depuis SQL Server (l'ancien guide visait T-SQL) :

| T-SQL (SQL Server) | PostgreSQL | Remarque |
|---|---|---|
| `IDENTITY(1,1)` | `GENERATED ALWAYS AS IDENTITY` | la base refuse qu'on fournisse la valeur (voulu) |
| `DATETIME2` | `TIMESTAMPTZ` | toujours avec fuseau, `CURRENT_TIMESTAMP` par défaut |
| `GETDATE()` | `CURRENT_TIMESTAMP` ou `now()` | en SQL ; `now()` côté Rust/SQLx |
| `ISNULL(x, y)` | `COALESCE(x, y)` | |
| `TOP (n)` | `LIMIT n` | ou `FETCH FIRST n ROWS ONLY` (standard) |
| `sp_rename` | `ALTER TABLE … RENAME` | migrations uniquement, jamais d'objet renommé en silence |
| `CREATE OR ALTER` | `CREATE OR REPLACE` (fonctions/vues) ; les tables passent par migration | |
| `sys.dm_sql_referencing_entities` | `pg_depend`, `information_schema` | recenser les consommateurs avant un renommage, même règle |
| `EXEC proc @p = 1` | `SELECT fn(1)` / `CALL proc(1)` | paramètres nommés = appel de fonction ordinaire |
| tables `#temp` | CTE, `CREATE TEMP TABLE …` | coûteuses pareil : mesurer avant |
| `NVARCHAR(n)` | `VARCHAR(n)` | PostgreSQL est UTF-8 par défaut, `VARCHAR` = texte unicode |
| `BIT` | `BOOLEAN` | vrai booléen, pas un entier |
| `NOCOUNT`/gestion erreurs `TRY/CATCH` | transactions sqlx / `ON CONFLICT` | la gestion d'erreur vit côté Rust et en SQL déclaratif |
| `MERGE` | `INSERT … ON CONFLICT DO UPDATE` | l'upsert PostgreSQL, un seul chemin |
| `STRING_SPLIT` | `string_to_array` / `unnest` | |
| `NEWID()` | `gen_random_uuid()` | si un jour un `public_id` UUID arrive |
| collations `CI` (insensible casse) | `LOWER()` + index sur expression | cf. `ux_account_username` — pas de collation CI par défaut en PG |

**Règle de transition :** en cas de doute réflexe T-SQL, chercher d'abord « postgresql équivalent de … » plutôt que traduire mot à mot ; puis noter la réponse ici si elle manque du tableau.

---

## À retenir en priorité

Un concept = un nom de la colonne au JSON ; toutes les contraintes nommées (`pk_`, `fk_`, `ck_`, `ux_`, `ix_`) ; la règle métier qui peut être une contrainte **est** une contrainte (`CHECK`, arc exclusif, `UNIQUE NULLS NOT DISTINCT`) ; `TIMESTAMPTZ` partout, `NUMERIC` pour les valeurs métier, durées en entiers `duration_ms` ; jamais modifier une migration appliquée ; `EXPLAIN (ANALYZE)` avant d'optimiser ; tester chaque requête seule avec des paramètres connus avant de l'encapsuler ; attention au fan-out des jointures 1:N avant tout agrégat.
