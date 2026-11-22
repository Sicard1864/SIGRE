# TODO Projet

Le fil conducteur du quotidien : ce qui est fait, ce qui est en cours, ce qui vient ensuite, et ce qu'on refuse de faire maintenant. Les grandes lignes vivent dans `ABOUT_PROJECT.md` § 4 (feuille de route V1) ; ici, ce sont les tâches concrètes, une ligne = une tâche faisable en une session.

Chaque tâche de « À faire » correspond à une issue Git (n°) et à une branche (`GIT_WORKFLOW.md` § 2).

<br>

## Fait — 16/11/2026

- **Docs** : cahier des charges refondu en `ABOUT_PROJECT.md` v3.0 (stack Rust + React + PostgreSQL actée, déploiement 3 serveurs, CI/CD dès le premier jour).
- **Docs** : guides de style réécrits — `STYLE_RUST.md` (à la place du guide C#), `STYLE_REACT.md`, `STYLE_API.md`, `STYLE_SQL.md` (T-SQL → PostgreSQL), `GIT_WORKFLOW.md` complété.
- **Docs** : modèle métier v2 (`DOMAIN_MODEL.md`) + schéma initial (`V0001__initial_schema.sql`) rédigés.

<br>

## En cours

- **Docs** : relire l'ensemble des guides après cette passe d'écriture, corriger les incohérences de nommage restantes (ex. références à `SPECIFICATIONS.md` à remplacer par `ABOUT_PROJECT.md` partout).

<br>

## À faire

Ordonné par priorité : chaque étape dépend des précédentes (le détail et les « definition of done » sont dans `ABOUT_PROJECT.md` § 4).

- **0. Tâche zéro — environnement et dépôt** : init du dépôt Git, push sur GitHub, branches `main`/`develop` + protection, `README.md` (installation sous Windows), CI minimale (fmt + clippy + test + lint front, même sur des projets quasi vides). *Rien ne commence avant ça.*
- **1. Les 3 tiers qui démarrent** : squelette Rust/Axum (route `/health`), squelette React/Vite (page d'accueil), PostgreSQL dans Docker Compose. *DoD : les trois conteneurs démarrent d'une commande, `/health` répond.*
- **2. Relier par l'API** : un premier endpoint bout en bout (ex. `GET /api/v1/questions` sur une table vide), CORS configuré, erreurs ProblemDetails. *DoD : le front affiche la réponse du back.*
- **3. Compte utilisateur** : migration `account`, inscription, DTO et validations, `POST /api/v1/accounts` + `GET /api/v1/accounts/me`.
- **4. Authentification** : Argon2id, JWT accès courte durée + refresh rotatif, middleware Axum, rate limiting login, `PATCH /accounts/me`, anonymisation RGPD.
- **5. Contenu** : `tag`, `question_type`, `question`, `answer`, publication (`private`/`pending_review`/`published`), endpoints de création et de lecture.
- **6. Moteurs SM-2 + Elo** : implémentation pure et testée (SM-2 par question, Elo par tag, pipeline `attempt` transactionnel : SM-2 → Elo → historique).
- **7. Pratique** : `practice_session`, sélection des questions dues, soumission de tentative, réponse avec `srs` et `elo_deltas` (l'exemple canonique de `STYLE_API.md` § 13).
- **8. Groupes et devoirs** : `study_group`, membres et rôles, `assignment` avec seuil 17/20, suivi des résultats par le propriétaire.
- **9. Mise en ligne** : images Docker des 3 serveurs, déploiement (VPS ou équivalent), HTTPS, sauvegardes PostgreSQL, premier tag `v0.1.0`.

<br>

## Points à voir

Questions ouvertes, à trancher quand on les rencontre :

- **Contenu** : comment versionner une question déjà utilisée dans des tentatives quand on la corrige (l'histoire des notes doit rester interprétable) ?
- **Juridique** : consentement parental pour les moins de 15 ans (RGPD, article 8) — à arbitrer avant l'ouverture réelle aux élèves.
- **Elo** : biais de population (les premiers contributeurs fixent la difficulté initiale des questions) — prévoir une période de calibration et une note dans `DOMAIN_MODEL.md` § 9.
- **Schéma** : ajouter un `public_id` UUID (interne BIGINT, externe UUID) ou non ? À décider avant la première API publique.
- **Ops** : choix de l'hébergeur (VPS français ?) et stratégie de sauvegarde/restauration testée.

<br>

## À ne pas faire pour l'instant

Pas important, pas maintenant, ou compliqué pour pas grand chose :

- **Gamification** : badges, succès, chaîne quotidienne (flamme) — le cœur pédagogique d'abord (`ABOUT_PROJECT.md` § 6).
- **Notifications** : rappels par email/web push, défis quotidiens notifiés.
- **Communauté** : forum, chat, commentaires — après la boucle élève/professeur.
- **Exercices avancés** : dictée, reconnaissance vocale, contrôles scannés corrigés.
- **Techno** : pgvector / recommandation par plongements, WebAssembly, QUIC, offline-first.
- **Front** : mode sombre (palette prête dans le CDC, implémentation plus tard), i18n anglais.
