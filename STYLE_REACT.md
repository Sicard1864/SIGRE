# Guide de style React / TypeScript — front SIGRE

> **Portée :** tout le code du front (SPA React + TypeScript, constructeur Vite).
> **Documents liés :** `STYLE_API.md` (contrat d'API, ce guide suppose ses conventions acquises), `ABOUT_PROJECT.md` § 2.3 (stack), `NAVIGATION.md` (écrans et parcours), `DOMAIN_MODEL.md` (vocabulaire métier).
> **Écrit pour être appliqué mécaniquement.** En cas de doute sur un point non couvert, on choisit la solution la plus simple à tester, puis on la note ici.

## Sommaire

1. [Principes](#1-principes)
2. [TypeScript : strict, pas de mage](#2-typescript--strict-pas-de-mage)
3. [Langue et nommage](#3-langue-et-nommage)
4. [Mise en forme et outil](#4-mise-en-forme-et-outil)
5. [Arborescence : fonctionnalités d'abord](#5-arborescence--fonctionnalités-dabord)
6. [Composants : fonctions, pas de classes](#6-composants--fonctions-pas-de-classes)
7. [Hooks et état](#7-hooks-et-état)
8. [Données serveur : TanStack Query](#8-données-serveur--tanstack-query)
9. [Formulaires et validation](#9-formulaires-et-validation)
10. [Contrat API : couche dédiée et typée](#10-contrat-api--couche-dédiée-et-typée)
11. [Sécurité côté front](#11-sécurité-côté-front)
12. [Accessibilité (obligatoire, pas optionnelle)](#12-accessibilité-obligatoire-pas-optionnelle)
13. [Tests](#13-tests)
14. [Interdits](#14-interdits)

---

## 1. Principes

- **Le front est bête, le serveur est intelligent.** Toute règle métier (SM-2, Elo, seuil 17/20) vit dans le back : le front affiche ce que l'API répond. Il ne recalcule jamais un `due_at`, ne devine jamais un niveau Elo, n'applique jamais une barre de réussite locale.
- **Un seul fichier par responsabilité :** un composant, un fichier ; un hook, un fichier ; une route API, un fichier.
- **Pas de dépendance sans motif.** Avant d'ajouter un paquet : qu'apporte-t-il qu'on ne peut écrire en dix lignes testées ? (Doctrine « Libéré » du CDC : privilégier l'open-source sobre, pas la mode.)
- **Les conventions de ce guide sont les mêmes que celles des autres guides** : un concept = un nom (`due_at`, `recall_grade`, `study_group` du modèle métier jusqu'aux props), identifiants anglais, textes français.

---

## 2. TypeScript : strict, pas de mage

- `tsconfig.json` en mode strict complet : `"strict": true`, `"noUncheckedIndexedAccess": true`, `"noFallthroughCasesInSwitch": true`.
- **Aucun `any`.** Un `unknown` + un rétrécissement (`typeof`, prédicat, `zod`) à chaque fois que le monde extérieur entre (réponse réseau, `localStorage`, événement DOM).
- **Les types vivent près de leur usage**, dans le fichier de la couche API pour les contrats (§ 10), près du composant pour les props.
- **Pas de types « cérémoniaux »** : si une interface n'a qu'un consommateur, un type local anonyme suffit.
- Un `enum` pour les listes fermées venant du métier (`session_type`, `publication_status`) : `as const` + union de littéraux, jamais d'`enum` TypeScript (不值得 la friction à la compilation).
- Les types de la couche API sont **générés depuis les DTO Rust si possible** (paquets `specta`/`ts-rust` un jour), sinon écrits à la main **une seule fois** dans `src/api/schema.ts` — jamais recopiés dans les composants.

---

## 3. Langue et nommage

- **Identifiants en anglais, textes visibles en français.** `const [isSubmitting, setIsSubmitting] = useState(false)` ; l'écran affiche « Révision en cours… ».
- **Composants : `PascalCase`**, nom explicite sur ce qu'il rend : `QuestionCard`, `GroupMemberList`, `DailyChallengeBanner` ; jamais `Index`, `Page2`, `Comp`.
- **Hooks : `use` + camelCase**, nom sur ce qu'ils apportent : `usePracticeSession`, `useDueQuestions`, `useAuth`.
- **Fonctions et variables : `camelCase`** ; **types et interfaces : `PascalCase`** ; **constantes module : `SCREAMING_SNAKE_CASE`**.
- Le vocabulaire suit le `DOMAIN_MODEL.md` : un composant qui liste des questions s'appelle `QuestionList`, pas `ItemList` — un concept métier = un nom (même règle que SQL, Rust et API).
- Fichiers : `PascalCase.tsx` pour les composants, `camelCase.ts` pour le reste, suffixe `.test.tsx` pour les tests.

---

## 4. Mise en forme et outil

- **Prettier** (pas de débat de style en revue) : `"printWidth": 100`, guillemets simples, virgule finale. Formatage **au save**, `--check` en CI.
- **ESLint** avec `typescript-eslint` recommandé + règles React (`react-hooks` en `error` : ses avertissements attrapent 80 % des bugs d'état).
- `npm run lint`, `npm run typecheck`, `npm run test`, `npm run build` — tous verts avant tout `git commit`, tous exécutés par la CI (`ABOUT_PROJECT.md` § 2.10).
- **Ordre des imports** (simple-import-sort ou manuel) : bibliothèques externes, puis internes par profondeur de chemin, séparés par lignes vides.
- **Pas de code mort** : pas de `console.log` oublié (ESLint le refuse en dehors des fichiers de dev), pas de composant commenté, pas de variable de secours « au cas où ».

---

## 5. Arborescence : fonctionnalités d'abord

- Organisation **par fonctionnalité**, pas par type technique — les fichiers qui changent ensemble vivent ensemble :

```text
src/
  api/            # la seule couche qui parle au réseau (§ 10)
  features/       # un dossier par brique métier
    practice/     #   la séance de révision (écran principal)
    groups/       #   groupes, membres, devoirs
    questions/    #   banque de questions, flashcards
    auth/         #   connexion, inscription, session
    dashboard/    #   accueil, défis quotidiens, progression
  shared/         # composants sans métier (Button, Modal, EmptyState)
  app/            # routes, providers, App.tsx
```

- Un composant qui n'existerait que dans une fonctionnalité y reste ; il ne migre vers `shared/` qu'au **deuxième** usage réel (règle des trois appliquée à deux : on duplique une fois, on factorise à la deuxième).
- Pas de `src/components/` généraliste qui devient un dépotoir : `shared/` contient des composants **sans** vocabulaire métier.

---

## 6. Composants : fonctions, pas de classes

- **Uniquement des composants-fonctions** avec hooks. Les classes React sont interdites (aucune raison valable en 2026).
- Ordre de lecture d'un composant, toujours le même :
  1. les hooks (état, requêtes),
  2. les dérivations et gardes (`if (!data) return <Loading />`),
  3. le JSX.
- **Un composant = une responsabilité.** Si son JSX dépasse un écran de code, on extrait un sous-composant qui reçoit des props explicites.
- **Les props sont typées une seule fois** :

```tsx
type QuestionCardProps = {
  question: Question;
  onSubmitAnswer: (answer: string) => void;
};
```

- **Pas de prop drilling au-delà de deux niveaux** : le contexte React est réservé aux préoccupations transverses (session auth, thème) ; pour le reste, la donnée descend par props, et ce qui remonte remonte par callbacks explicites.
- Les enfants **ne rendent jamais la donnée qu'ils ne connaissent pas** : un `QuestionCard` reçoit une question, il ne va pas la chercher dans un hook global.

---

## 7. Hooks et état

- **État local** (`useState`) pour l'UI éphémère : champ en cours de saisie, panneau ouvert.
- **État serveur = TanStack Query, jamais `useState` + `useEffect` + `fetch`** (§ 8) : la donnée qui vient de l'API n'est jamais « possédée » par un composant.
- **Pas d'`useEffect` pour dériver une valeur** : si `x` se calcule depuis les props ou l'état, c'est une simple variable pendant le rendu (ou `useMemo` si le calcul est vraiment lourd — mesuré, pas deviné).
- `useEffect` a le droit d'exister pour : s'abonner au monde extérieur (titre de page, dimensions, timer d'exercice), et c'est presque tout. Chaque `useEffect` porte son nettoyage (`return () => …`) quand il s'abonne.
- **Ordre et stabilité** : jamais de hook conditionnel ; les callbacks passés aux enfants sont stables (`useCallback`) uniquement si la mesure le justifie — pas de mémorisation défensive systématique.

---

## 8. Données serveur : TanStack Query

- **Toute lecture d'API passe par `useQuery` ; toute mutation par `useMutation`.** Aucun `fetch` dans un composant.
- **`queryKey` = la ressource**, hiérarchique et cohérente avec les URL de l'API (§ `STYLE_API.md`) : `['practice-sessions', sessionId]`, `['questions', { tagId, page }]`, `['accounts', 'me']`.
- **Invalidation plutôt que mise à jour manuelle du cache** : après `POST /practice-sessions/{id}/attempts`, on invalide `['practice-sessions', sessionId]` — on n'essaie pas de patcher le cache à la main, source classique d'incohérences.
- **États standards partout** : chaque `useQuery` gère `isPending`, `isError`, `data` ; l'erreur serveur affiche le message du `ProblemDetails` (`STYLE_API.md` § 7) via un `Toast` standard, jamais un `console.log` silencieux.
- Pas d'état global (Redux & co) : TanStack Query **est** l'état serveur ; le reste est local ou en contexte auth.

---

## 9. Formulaires et validation

- **`react-hook-form` + `zod`** pour tout formulaire à plus d'un champ (inscription, création de groupe, proposition de question).
- Le schéma zod **est** la règle de validation du formulaire ; les règles métier qui exigent le serveur (unicité d'email) restent au serveur : le client valide le format, le serveur valide le métier, et la réponse `422` se mappe sur les champs via `setError` (codes d'erreur `STYLE_API.md` § 7).
- **Valeurs par défaut explicites** et types dérivés du schéma (`z.infer<typeof schema>`), jamais de `any` dans un `register`.

---

## 10. Contrat API : couche dédiée et typée

- **Un seul dossier parle au réseau : `src/api/`** ; un module par ressource (`practiceSessions.ts`, `groups.ts`, `questions.ts`), un fichier `schema.ts` pour les types partagés.
- Chaque fonction de la couche : URL (celles de `STYLE_API.md`), typage entrée/sortie explicite, gestion de l'`ApiError` normalisée.
- **Traduction stricte du contrat :**
  - identifiants reçus en chaînes (`question_id: string`), stockés et renvoyés tels quels — **jamais** convertis en `number` (le piège `BIGINT` JSON, § `STYLE_API.md` 5) ;
  - dates ISO UTC reçues, converties **au moment de l'affichage** (`Intl.DateTimeFormat('fr-FR')`), jamais modifiées en amont ;
  - `scoreRatio`, `recallGrade`, `dueAt`… : mêmes noms que l'API en snake_case dans la couche, adaptés en camelCase uniquement aux frontières (convention d'uniformisation documentée dans `schema.ts`).
- Pas de logique métier dans `src/api/` : c'est un traducteur, pas un cerveau.

---

## 11. Sécurité côté front

- **Jamais de JWT dans `localStorage`** (injection XSS = vol de session). Jeton d'accès en mémoire (hook de session) ; refresh token en cookie `HttpOnly` posé par le serveur (`STYLE_API.md` § 11).
- **Pas de secret dans le front, jamais.** Tout ce qui est dans le bundle est public : clé d'API, secret de signature, mot de passe de test — aucun.
- **Anti-XSS par défaut :** React échappe déjà ; interdits donc `dangerouslySetInnerHTML` (aucun contenu riche ne l'exige en V1), et les URLs construites depuis des saisies sont encodées (`encodeURIComponent`).
- Le front **n'affiche pas** ce qu'il ne devrait pas savoir faire, mais ne protège rien : toute autorisation est rejugée côté serveur (règle « le front est bête », § 1).

---

## 12. Accessibilité (obligatoire, pas optionnelle)

Objectif WCAG 2.1 AA (exigence du CDC, école = public jeune et varié) :

- **Sémantique d'abord** : un bouton est un `<button>`, une navigation une `<nav>`, une liste un `<ul>`. Pas de `<div onClick>` cliquable.
- **Toute action atteignable au clavier**, focus visible (outline jamais en `outline: none` sans remplacement), piège de focus dans les modales.
- **Étiquettes explicites** : `<label htmlFor>` sur chaque champ, `aria-label` sur les boutons icônes (la flashcard de révision doit rester utilisable au clavier : c'est le parcours principal d'une séance).
- **États communicants** : `aria-live` pour le retour de correction (le résultat d'une tentative doit être annoncé aux lecteurs d'écran), `aria-busy` pendant les chargements.
- **Contraste** : la palette du CDC, corrigée pour WCAG AA (`NAVIGATION.md` § 2.3) — en mode clair, CTA et liens en `#0E7490` avec texte blanc, le teal `#06B6D4` réservé au mode sombre et à la décoration ; toute nouvelle couleur passe par le même contrôle (ratio 4,5:1 texte, 3:1 composants).
- **7 à 14 éléments par écran** (règle UX du CDC) : un écran qui dépasse se découpe, en accord avec `NAVIGATION.md`.

---

## 13. Tests

Trois niveaux, tous en **Vitest** + **React Testing Library** (et **Playwright** pour le bout en bout) :

- **Utilitaires purs** (formatage, dérivations) : testés directement, pas de rendu nécessaire.
- **Composants** : on teste ce que l'utilisateur voit et fait (query by role, pas by class) — « la carte question affiche l'énoncé et le bouton Valider devient actif quand on répond ».
- **Parcours critiques** (Playwright, CI) : le parcours **séance de révision** (ouvrir une séance → répondre → voir la correction) et le parcours **connexion → dashboard**. Ces deux parcours ne cassent jamais en silence.

Règles : les tests ne mockent que les frontières (`msw` simule l'API) ; jamais de snapshot aveugle ; un test a un nom en français qui décrit le comportement attendu (« affiche la correction après validation »).

---

## 14. Interdits

- `any`, `!` (assertion non nulle) sans garde, `as` de contournement.
- `fetch` hors de `src/api/`.
- `useEffect` sans liste de dépendences, ou pour dériver de l'état.
- `localStorage` pour un jeton, un secret ou de la donnée métier.
- Bibliothèque d'état global (Redux, Zustand…) sans avoir prouvé que Query + contexte ne suffisent pas.
- Un composant qui dépasse ~150 lignes, un hook qui parle à l'API en direct, une classe React.
- Une couleur, une chaîne visible, une date formatée hors du système centralisé (palette CDC, `i18n` des textes, `Intl`) — le jour du mode sombre ou de l'anglais, tout doit tenir en un seul dossier.
