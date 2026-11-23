# TODO Projet

Le fil conducteur du quotidien : ce qui est fait, ce qui est en cours, ce qui vient ensuite, et ce qu'on refuse de faire maintenant. Les grandes lignes vivent dans `doc/about-project.md` § 4 (feuille de route V1) ; ici, ce sont les tâches concrètes, une ligne = une tâche faisable en une session.

Chaque tâche de « À faire » correspond à une issue Git (n°) et à une branche (`doc/git-workflow.md` § 2).

<br>

## Fait

- **16/11/2026 — Docs** : cahier des charges refondu en about-project v3.0 (stack Rust + React + PostgreSQL actée, déploiement 3 serveurs, CI/CD dès le premier jour).
- **16/11/2026 — Docs** : guides de style réécrits — rust (à la place du guide C#), react, api, sql (T-SQL → PostgreSQL), git-workflow complété.
- **16/11/2026 — Docs** : modèle métier v2 (`doc/domain-model.md`) + schéma initial rédigés.
- **22/11/2026 — Environnement** : poste Windows complet et vérifié : Git 2.56.1, Rust 1.100.0, Node 26.13.1 LTS, PostgreSQL 16.11, VSCodium, auth SSH GitHub (`Sicard1864`) opérationnelle.
- **22/11/2026 — Dépôt** : repo GitHub structuré (`doc/`, `back/`, `front/`), licence GPL-3.0, .gitignore, README complet avec badge CI, doc normalisée en kebab-case.
- **22/11/2026 — Base** : migration initiale `0001_initial_schema` appliquée (24 tables, schéma `app`), migrations up/down vérifiées.
- **22/11/2026 — Squelettes** : crate `sigre-back` (Axum + SQLx, `/health` → `OK db=1` vérifié en local), front Vite `react-ts` (ESLint).
- **23/11/2026 — Tâche 0 terminée** : CI GitHub Actions **verte** (fmt, clippy, migrations, tests, build front) + badge ; branche `develop` créée, `main` protégée (PR obligatoire) ; description + topics du repo ; `back/.env.example` ; README enrichi (Docker, Versions testées, corrections).
- **23/11/2026 — Tâche 1 terminée** : virtualisation activée (BIOS), Docker Desktop opérationnel, `docker-compose.yml` + Dockerfiles (cache cargo-chef), migrations automatiques au démarrage (`sqlx::migrate!`), pile complète par une commande, `/health` joignable depuis le conteneur (fix `0.0.0.0`).
- **23/11/2026 — Tâche 2 terminée** : arbitrage `public_id` **acté : UUID externe, BIGINT interne jamais exposé** (anti-énumération) ; migration `0002` ; `GET /api/v1/questions` validé bout en bout (UUID dans la réponse, page React affichant « flashcard — private ») ; CORS à liste explicite (5173, 8080) ; module `problem.rs` (ProblemDetails RFC 9457) ; workflow branches/PR en place.
- **Leçons intégrées** : signature des handlers Axum (le bound `Handler` pointe la signature, pas le corps) ; checksum SQLx octet par octet → `.gitattributes` `*.sql eol=lf` (une migration appliquée ne se retouche jamais) ; `target/` sans slash initial dans .gitignore ; `search_path TO app, public` pour le carnet sqlx.

<br>

## En cours

- **Merger la PR** `feat/api-v1-questions` → `develop` sur GitHub (attendre la CI verte dans la PR, puis Merge + suppression de la branche), puis localement : `git checkout develop && git pull`.

<br>

## À faire

Ordonné par priorité : chaque étape dépend des précédentes (le détail et les « definition of done » sont dans `doc/about-project.md` § 4).

- **3. Compte utilisateur** : inscription, DTO et validations, `POST /api/v1/accounts` + `GET /api/v1/accounts/me` (penser `public_id` sur `account`, même logique que 0002).
- **4. Authentification** : Argon2id, JWT accès courte durée + refresh rotatif, middleware Axum, rate limiting login, `PATCH /accounts/me`, anonymisation RGPD.
- **5. Contenu** : `tag`, `question_type`, `question`, `answer`, publication (`private`/`pending_review`/`published`), endpoints de création et de lecture.
- **6. Moteurs SM-2 + Elo** : implémentation pure et testée (SM-2 par question, Elo par tag, pipeline `attempt` transactionnel : SM-2 → Elo → historique). *Le cœur différenciant du projet — conversation Vibe dédiée.*
- **7. Pratique** : `practice_session`, sélection des questions dues, soumission de tentative, réponse avec `srs` et `elo_deltas` (l'exemple canonique de `doc/style-api.md` § 13).
- **8. Groupes et devoirs** : `study_group`, membres et rôles, `assignment` avec seuil 17/20, suivi des résultats par le propriétaire.
- **9. Mise en ligne** : images Docker des 3 serveurs (ébauchées), déploiement (VPS français ou équivalent), HTTPS, sauvegardes PostgreSQL testées, premier tag `v0.1.0`.

<br>

## Points à voir

Questions ouvertes, à trancher quand on les rencontre :

- **Contenu** : comment versionner une question déjà utilisée dans des tentatives quand on la corrige (l'historique des notes doit rester interprétable) ?
- **Juridique** : consentement parental pour les moins de 15 ans (RGPD, article 8) — à arbitrer avant l'ouverture réelle aux élèves.
- **Elo** : biais de population (les premiers contributeurs fixent la difficulté initiale des questions) — prévoir une période de calibration et une note dans `doc/domain-model.md` § 9.
- **Ops** : choix de l'hébergeur (VPS français ?) et stratégie de sauvegarde/restauration testée.

<br>

## À ne pas faire pour l'instant

Pas important, pas maintenant, ou compliqué pour pas grand chose :

- **Kubernetes** : Docker Compose d'abord ; K3s seulement si le projet dépasse réellement une machine (le cœur du sujet, c'est PostgreSQL persistant, pas l'orchestration).
- **Gamification** : badges, succès, chaîne quotidienne (flamme) — le cœur pédagogique d'abord (`doc/about-project.md` § 6).
- **Notifications** : rappels par email/web push, défis quotidiens notifiés.
- **Communauté** : forum, chat, commentaires — après la boucle élève/professeur.
- **Exercices avancés** : dictée, reconnaissance vocale, contrôles scannés corrigés.
- **Techno** : pgvector / recommandation par plongements, WebAssembly, QUIC, offline-first.
- **Front** : mode sombre (palette prête dans le CDC, implémentation plus tard), i18n anglais.
