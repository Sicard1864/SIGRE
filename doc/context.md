# CONTEXT.md — Projet SIGRE

> **Rôle de ce fichier :** carte d'identité du projet et porte d'entrée du dépôt. Un lecteur humain ou une IA commence ici.
>
> **Projet :** Système Informatique de Gestion de Révisions Espacées (SIGRE)
>
> **Dernière mise à jour :** 16/11/2026

## Problème

Les savoirs fondamentaux s'effacent faute de répétition au bon moment (courbe de l'oubli). Le système scolaire n'a ni le temps ni les moyens d'offrir une répétition personnalisée à chacun, et les outils existants (Anki, SuperMemo) sont individuels, peu accessibles et sans suivi pour les enseignants.

Constat de départ : recul en mathématiques et disciplines scientifiques, classements internationaux décevants (PISA), fortes inégalités selon le milieu social, sur-stimulation constante qui dégrade concentration et mémorisation.

## Idée proposée

Une plateforme web **gratuite, open-source et communautaire** combinant :

- la **répétition espacée** (SM-2 : macro pour les savoir-faire, micro pour les savoirs par cœur) → *quand* revoir ;
- le **classement Elo par tag** → *quel niveau* pour l'utilisateur, *quelle difficulté* pour le contenu ;
- les **tags** en arborescence → *quoi* réviser ;
- une **gamification sobre** (défi quotidien, succès) pour créer l'habitude, **sans comparaison à la moyenne du groupe**.

Le détail complet (vision, cas d'usage, fonctionnalités, thèmes, stack, sécurité, feuille de route, financement) est dans `ABOUT_PROJECT.md`.

## Utilisateurs cibles

| Profil | Fréquence attendue | Contexte |
|---|---|---|
| Élèves (primaire → études sup.) | quotidienne, sessions courtes | révisions, renforcement des bases |
| Enseignants | hebdomadaire (préparation + suivi) | classe, devoirs, contrôles imprimables |
| Adultes en (re)formation | régulière | auto-formation structurée |
| Particuliers | ponctuelle à régulière | code de la route, code du travail, vivre seul, santé |

## Contraintes

- **Temps :** projet personnel à temps partiel (10–15 h/semaine), MVP visé en 3 à 5 semaines avec assistance IA (`AI_GUIDELINES.md`).
- **Budget :** gratuit, sans publicité ; coûts serveurs fixes et maîtrisés ; hébergement souverain préféré (VPS Debian).
- **Compétences disponibles :** développeur junior C#/.NET en poste (React/TS et SQL déjà utilisés) ; Rust découvert via ce projet. Détail dans `ABOUT_ME.md`.
- **Stack (décision actée, V1) :** backend Rust (Axum + SQLx), frontend React + TypeScript, PostgreSQL 16 (15 minimum pour `UNIQUE NULLS NOT DISTINCT`). Justification dans `ABOUT_PROJECT.md` § 0 et § 2.2. Environnement de développement sous Windows ; monolithe déployé en 3 serveurs Docker (back, base, front), CI/CD et tests dès le premier jour (`ABOUT_PROJECT.md` § 2.10).
  - **Décision tranchée :** l'alternative C#/.NET est écartée pour ce projet. Le guide C# est remplacé par `STYLE_RUST.md` (une table de correspondance C# → Rust y facilite la transition). L'ADR-001 reste à rédiger pour formaliser la décision.
- **Souveraineté et open-source :** composants 100 % libres, données sous juridiction française, auto-hébergeable.
- **Cadre légal :** RGPD (public jeune : consentement parental < 15 ans à valider juridiquement), pas de politique ni de religion dans les contenus, licences CC BY-SA respectées pour les sources libres.
- **Compatibilité :** Debian côté serveur, Firefox (Gecko) côté client en priorité, WCAG 2.1 AA.

## Hypothèses

À valider, par ordre d'impact :

1. SM-2 (micro) + Elo (macro) produisent un suivi personnalisé réellement utile.
2. Le défi quotidien (chaîne de jours) crée l'habitude de retour.
3. Les enseignants adoptent l'outil comme **complément** qui ne contraint pas leur pédagogie.
4. Une communauté se constituera pour proposer et valider des exercices (doublons, droits d'auteur, dérives maîtrisés par la relecture).
5. Le calibrage Elo des exercices par les résultats réels remplace efficacement un niveau déclaré à la main.
6. L'empreinte faible de Rust permet un hébergement à coût négligeable.

## Critères de réussite

Mesurables dès le MVP :

- un utilisateur crée un compte, pratique, et le moteur planifie correctement ses révisions (file `due_at` cohérente) ;
- un enseignant crée un groupe, donne un devoir avec barre minimum (`pass_threshold_ratio`), consulte les résultats avec temps passé ;
- les moteurs SM-2 et Elo sont couverts par des tests automatisés (bornes, monotonie, historique) ;
- l'application est déployée et utilisable sur Firefox/Debian, en mode clair et sombre ;
- la documentation (`ABOUT_PROJECT.md`, `DOMAIN_MODEL.md`, schéma SQL) reflète exactement le code livré.

## Questions ouvertes

Reprises de `DOMAIN_MODEL.md` § 9 et `ABOUT_PROJECT.md` § 7 — à trancher avant d'ouvrir les contributions :

- versionnage des questions (une question corrigée change le sens des anciennes réponses) ;
- consentement parental (< 15 ans, RGPD art. 8) et connaissance de l'âge à l'inscription ;
- biais de population des difficultés Elo ;
- cycles dans l'arborescence des tags ;
- identifiants énumérables dans les URL (`public_id` UUID ?) ;
- statut juridique et modèle de financement (`ABOUT_PROJECT.md` § 5).

## Cartographie des documents

Un besoin = un fichier. En cas de doute, la vérité appartient au fichier listé ; les autres renvoient vers lui.

| Question | Fichier |
|---|---|
| Par quoi commencer ? (ce fichier) | `CONTEXT.md` |
| Le quoi et le pourquoi du produit | `ABOUT_PROJECT.md` |
| Les données : tables, décisions, flux d'une réponse | `DOMAIN_MODEL.md` (+ `V0001__initial_schema.sql`) |
| L'architecture de l'application | `ARCHITECTURE.md` |
| Les écrans et le parcours utilisateur | `NAVIGATION.md` |
| Comment écrire le code | `STYLE_RUST.md`, `STYLE_REACT.md`, `STYLE_SQL.md` |
| Les conventions de l'API HTTP | `STYLE_API.md` |
| Comment utiliser Git | `GIT_WORKFLOW.md` |
| Ce qu'il reste à faire | `TODO.md` |
| Ce que j'ai appris (à relire, ne pas réexpliquer) | `TIL.md` |
| Synthèse de fin de projet | `0-CONCLUSION.md` |
| Qui je suis, comment j'apprends | `ABOUT_ME.md` |
| Comment je travaille avec une IA | `AI_GUIDELINES.md` |
| Comment builder, lancer, tester | `README.md` (à créer) |

**Langue du dépôt :** documents en français ; identifiants techniques et code en anglais (`snake_case` en SQL, voir `STYLE_SQL.md`) ; le vocabulaire métier reste en français quand il n'a pas d'équivalent technique (modèle métier, cahier des charges).

## Historique des décisions prises ici

- **16/11/2026 (V1)** — Décisions actées : stack Rust + React + PostgreSQL définitive (C# écarté, `STYLE_RUST.md` remplace le guide C#) ; développement sous Windows, monolithe en 3 serveurs Docker (back, base, front) ; tests et CI/CD dès le premier jour ; ordre des tâches V1 (Git → 3 tiers → API → compte → auth). Voir `ABOUT_PROJECT.md` § 0 et § 4.
- **16/11/2026** — Stack cible Rust (Axum + SQLx) + React/TypeScript, PostgreSQL 16. Justification dans `ABOUT_PROJECT.md` § 2.2. *Statut : confirmé depuis (entrée V1 ci-dessus).*
