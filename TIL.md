# TIL — Today I Learned

> Ce que j'apprends au jour le jour, **avec mes propres mots**.
> Le but n'est pas d'archiver, mais de **me tester** : je dois pouvoir répondre aux questions sans relire la réponse.
> **Règle absolue :** rien n'entre ici que je n'ai pas écrit moi-même. Ni copier-coller de doc, ni texte généré par une IA. Une entrée n'a de valeur que si elle reformule ce que j'ai compris — c'est le test de compréhension, pas une archive.

---

<br>

## Semaine du 16/11/2026 — projet SIGRE

*Trois amorces ci-dessous : les questions que le projet pose dès maintenant. À remplir avec mes mots quand j'aurai creusé — une entrée incomplète que j'ai écrite vaut mieux qu'une réponse parfaite copiée.*

### [Modèle] Pourquoi `score_ratio` ET `recall_grade` distincts ?

- **Contexte :** `attempt` porte deux notes : `score_ratio NUMERIC(4,3)` et `recall_grade SMALLINT` 0-5.
- **À creuser :** qui sert l'Elo, qui sert SM-2, et pourquoi une seule colonne ne peut pas faire les deux.
- **Piège :** mélanger les deux, c'est faire dépendre le calendrier de révision (SM-2) du niveau (Elo) — deux moteurs indépendants par conception (`ABOUT_PROJECT.md` § 3.1).

<details>
<summary>Question : pourquoi la note de rappel SM-2 (0-5) et le score de réussite Elo (0-1) sont-ils deux colonnes différentes ?</summary>

  *À rédiger par moi-même.*
</details>

<br>

### [SQL] Pourquoi `UNIQUE NULLS NOT DISTINCT` sur `tag` ?

- **Contexte :** `tag` peut avoir un `parent_tag_id` facultatif ; l'unicité du couple (slug, parent) doit tenir même quand le parent est NULL.
- **À creuser :** le comportement par défaut de `UNIQUE` face à NULL (deux NULL ne sont jamais égaux), et pourquoi PostgreSQL 15+ résout ça.
- **Piège :** avec un `UNIQUE` classique, deux tags racines de même slug coexisteraient silencieusement.

<details>
<summary>Question : que se passerait-il avec un `UNIQUE (slug, parent_tag_id)` ordinaire quand deux tags racines ont le même slug ?</summary>

  *À rédiger par moi-même.*
</details>

<br>

### [Modèle] Pourquoi pas d'association polymorphe ?

- **Contexte :** `elo_rating` vise `account_tag` OU `question_tag` OU `exercise_tag` — mais via trois colonnes + arc exclusif `CHECK`, pas une seule colonne « (type, id) ».
- **À creuser :** ce qu'une FK polymorphe `(entity_type, entity_id)` interdit (clé étrangère réelle, intégrité garantie par la base) et pourquoi on a payé trois colonnes pour ça.
- **Piège :** la colonne polymorphe est séduisante et casse la règle « la base est la seule source de vérité » (`STYLE_SQL.md` § 1).

<details>
<summary>Question : qu'est-ce qu'une clé étrangère polymorphe empêche que l'arc exclusif garantit ?</summary>

  *À rédiger par moi-même.*
</details>

<br>

---

<br>

## Semaine du 16/10/2026 — pro R200K (entrées existantes, gardées comme exemples)

<br>

### [SQL] Filtrer sur la pièce, pas sur la huisserie

- **Contexte :** la recherche d'accessoires remontait toutes les pièces d'une huisserie (montants, traverses) alors que la gâche n'était que sur un montant.
- **Appris :** `ABAT_HB_Usinage` est liée à la huisserie (`id_hb`), mais possède aussi une colonne `element`. Il faut filtrer sur `hbu.element = li.Element_case`.
- **Piège :** filtrer avec `EXISTS` plutôt qu'avec un `INNER JOIN`, qui duplique la pièce pour chaque accessoire correspondant.

```sql
WHERE EXISTS (
    SELECT 1 FROM GPSQL.ABAT_HB_Usinage hbu
    WHERE hbu.id_hb = ah.id_hb
      AND hbu.element = li.Element_case
)
```

<details>
<summary>Question : pourquoi EXISTS plutôt qu'INNER JOIN pour « la pièce porte au moins un des accessoires » ?</summary>

  `EXISTS` répond seulement oui ou non pour chaque pièce. Un `INNER JOIN` renvoie une ligne par accessoire trouvé, donc des doublons.
</details>

<br>

### [C#] Une fausse valeur par défaut fausse l'affichage

- **Contexte :** la qualité « à faire » s'affichait en rouge au lieu de noir dans la recherche d'accessoires.
- **Appris :** `AvancementFin` valait `DateTime.Now`, donc `HasValue` était toujours vrai et la pièce était vue comme usinée. Il fallait lire la vraie colonne `li.Date_Fin_Reelle`, `null` si la pièce n'est pas usinée.
- **Piège :** ne jamais remplir une donnée inconnue avec une valeur plausible.

<details>
<summary>Question : pourquoi mettre null plutôt que DateTime.Now quand une date est inconnue ?</summary>

  `null` veut dire « pas d'information ». Une fausse date est impossible à distinguer d'une vraie et casse tous les tests du type `HasValue`.
</details>

<br>

### [Données] Même source pour la liste et pour le filtre

- **Contexte :** les accessoires cochés (désignations du configurateur) ne correspondaient pas à `Libelle_Accessoire` du pointage (`MotDirecteur` de l'article) : la recherche ne trouvait rien.
- **Appris :** filtrer sur un **code** (`hbu.Code` joint à la table du configurateur), pas sur un texte. Dans une `CheckedListBox`, on peut ajouter des objets : `ToString()` gère l'affichage, une propriété garde la valeur de filtre.
- **Piège :** le texte affiché n'est pas une clé fiable : il peut changer (ajout des codes articles) sans que la donnée change.

<details>
<summary>Question : pourquoi ne pas filtrer directement sur le texte affiché dans la liste ?</summary>

  Le texte affiché peut être enrichi ou reformulé, et il ne correspond pas forcément à la colonne filtrée en base. On sépare la valeur affichée de la valeur métier.
</details>

<br>

### [C#] Champ ou propriété

- **Appris :** un **champ** stocke une donnée (en général `private`, préfixe `_`). Une **propriété** contrôle l'accès avec `get` / `set`. `{ get; set; }` est une propriété automatique : le compilateur crée le champ caché.
- **Piège :** `=>` et `=` ne font pas du tout la même chose.

```csharp
public DateTime Debut => dtpDebut.Value;  // propriété : relue à chaque accès
public DateTime Debut = dtpDebut.Value;   // champ : ne compile pas ici (CS0236)
```

<details>
<summary>Question : pourquoi la deuxième ligne ne compile pas ?</summary>

  Un initialiseur de champ ne peut pas utiliser un autre membre d'instance (`dtpDebut`), car il s'exécute avant le constructeur et `InitializeComponent()`.
</details>

<br>

### [WinForms] Task.Run, contrôles et connexion partagée

- **Appris :** lire les valeurs des contrôles **avant** `Task.Run`, car les contrôles WinForms ne s'utilisent que depuis le thread de l'interface. Une méthode qui utilise `Statique.CONNEC` depuis `Task.Run` doit prendre `lock (Statique.VerrouConnec)`.
- **Piège :** l'erreur n'apparaît pas toujours en test, seulement quand deux accès tombent en même temps.

<details>
<summary>Question : pourquoi écrire string famille = FiltreFamilleAccessoire; avant le Task.Run ?</summary>

  `FiltreFamilleAccessoire` lit la `ComboBox`. Lue dans `Task.Run`, elle serait accédée depuis un thread de fond, ce que WinForms n'autorise pas.
</details>

<br>

### [Code] Supprimer plutôt que commenter

- **Contexte :** `groupFab` n'était plus utile dans `GetListAccessoireRecherche`, et son filtre SQL avait été commenté (`--" + filtreGroupe + @"`).
- **Appris :** un paramètre gardé mais ignoré est trompeur : un appelant croit filtrer alors que rien ne se passe. On supprime, Git garde l'historique.

<details>
<summary>Question : quel est le risque de commenter l'usage d'un paramètre sans le supprimer ?</summary>

  La signature promet un comportement qui n'existe plus : le filtre est ignoré silencieusement, sans erreur ni avertissement.
</details>

<br>

### [Markdown] Lignes vides et commentaires

- **Appris :** toujours une ligne vide avant et après une liste, un tableau ou un titre. Une ligne de texte collée après une liste est absorbée par le dernier élément. Plusieurs lignes vides n'en font qu'une : `<br>` ou `&nbsp;` pour aérer.
- **Piège :** un commentaire `<!-- ... >` sans `-->` masque **tout** le reste du fichier.

<details>
<summary>Question : pourquoi les sections 7 et 8 de ABOUT_ME.md n'apparaissaient plus dans l'aperçu ?</summary>

  Un commentaire HTML de la section 6 n'était pas fermé : tout ce qui suivait était considéré comme du commentaire.
</details>

<br>

### [Outils] Copier-coller bloqué

- **Appris :** dans VS Code, c'est souvent une extension (Vim) ou un raccourci réassigné : `Ctrl+K Ctrl+S` pour vérifier, `Help: Start Extension Bisect` pour trouver l'extension fautive. En Bureau à distance, relancer `rdpclip.exe`.

<details>
<summary>Question : quelle commande VS Code trouve automatiquement l'extension qui pose problème ?</summary>

  `Help: Start Extension Bisect` : elle désactive les extensions par moitiés jusqu'à isoler la coupable.
</details>

<br>

### [Git] Taille d'un commit

- **Appris :** le nombre de commits ne mesure rien. Un bon commit = une modification logique et autonome, avec un message qui dit quoi et pourquoi. Sur une branche perso, on peut faire de petits commits puis les regrouper (squash) avant de fusionner.

<details>
<summary>Question : à quel moment faire un commit ?</summary>

  Dès qu'une étape logique est terminée et que le projet compile.
</details>

<br>

### Revue du vendredi 16/10/2026

- **Acquis :**
- **À revoir dans un mois :**

<br>

---

<br>

## Mode d'emploi

- **1 entrée = 1 notion**, en 3 à 5 lignes maximum.
- **Écrire avec mes mots**, jamais un copier-coller (ni de la doc, ni de l'IA).
- **Une question par entrée**, réponse cachée dans un bloc dépliable.
- **Chaque vendredi** : relire la semaine, répondre de tête aux questions, noter ce qui est à revoir.
- **Un mois plus tard** : refaire les questions des entrées marquées « à revoir ».

### Modèle

```markdown
### [Domaine] Titre court

- **Contexte :** où je l'ai rencontré.
- **Appris :** ce que j'ai compris, en une ou deux phrases.
- **Piège :** ce qui m'a fait perdre du temps ou ce qu'il ne faut pas faire.

<details>
<summary>Question : ... ?</summary>

  Réponse courte.
</details>
```
