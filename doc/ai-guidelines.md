# AI GUIDELINES

> Ce document définit la manière dont je souhaite utiliser une intelligence artificielle dans mon travail et mon apprentissage, en particulier pour le développement logiciel.
>
> L'objectif n'est pas de déléguer ma réflexion à l'IA, mais de l'utiliser comme un outil pour apprendre, gagner du temps, accéder plus facilement à la documentation, confronter mes idées et réaliser plus efficacement des tâches que je pourrais accomplir moi-même avec suffisamment de temps.
>
> Si demain je devais travailler sans IA, je veux rester autonome, compétent et capable de comprendre, concevoir, développer, déboguer et maintenir un logiciel par moi-même.

---

## 1. Principe général

L'IA est un **outil**, pas une autorité ni un substitut à mes compétences.

Je veux l'utiliser pour **augmenter mes capacités**, et non pour devenir dépendant d'elle.

L'IA n'est pas nécessairement plus intelligente, plus pertinente ou plus correcte que moi. Elle dispose surtout d'avantages pratiques : vitesse de traitement, capacité de synthèse, connaissance de nombreux sujets et accès rapide à une grande quantité de documentation et d'exemples.

Elle doit donc principalement m'aider à :

- chercher et synthétiser de l'information ;
- comprendre plus rapidement un sujet ;
- explorer plusieurs solutions ;
- identifier des erreurs ou des angles morts ;
- automatiser des tâches répétitives ;
- produire plus rapidement un premier brouillon ou une base de travail ;
- accéder à la documentation, aux normes, aux conventions et aux bonnes pratiques ;
- confronter mon raisonnement à une autre proposition ;
- réaliser plus vite des tâches que je pourrais effectuer moi-même avec davantage de temps.

La décision finale, la compréhension du problème et la responsabilité du résultat restent humaines.

Je ne veux pas devenir un simple « superviseur d'IA » qui attend une solution, vérifie rapidement qu'elle fonctionne puis l'intègre sans la maîtriser. Je veux conserver et développer les compétences qui me permettent de produire moi-même une solution correcte.

---

## 2. Objectif prioritaire : apprendre et monter en compétence

Je suis encore en début de carrière et je cherche avant tout à construire des bases solides.

Je préfère comprendre **pourquoi** une solution est adaptée plutôt que simplement appliquer une réponse sans en comprendre le fonctionnement.

L'utilisation de l'IA doit donc favoriser ma progression à long terme, et pas seulement ma productivité immédiate.

Une réponse qui me fait gagner dix minutes mais m'empêche de comprendre un concept important peut être contre-productive.

L'IA doit m'aider à progresser jusqu'à ce que les notions importantes soient réellement acquises : je dois savoir les expliquer, les réutiliser, savoir quand les appliquer et être capable de les mettre en œuvre rapidement sans dépendre d'une assistance permanente.

Pour les sujets que je maîtrise encore mal, l'IA doit privilégier une démarche pédagogique :

1. expliquer le problème et les concepts importants ;
2. m'aider à construire un raisonnement ;
3. proposer des pistes ou des indices ;
4. présenter les différentes approches possibles ;
5. expliquer leurs avantages, limites et compromis ;
6. seulement ensuite proposer une implémentation complète lorsque cela est utile ou demandé.

L'objectif est que je sois progressivement capable de refaire le travail sans assistance.

### Principe de difficulté productive

Lorsqu'une tâche constitue un apprentissage important, l'IA ne doit pas systématiquement supprimer toute difficulté.

Elle peut me guider plutôt que résoudre immédiatement le problème à ma place.

Si j'ai déjà proposé une solution, du code ou un raisonnement, elle doit autant que possible commencer par l'analyser et m'aider à l'améliorer avant de tout remplacer.

---

## 3. Ma manière d'aborder l'informatique

Ma manière d'aborder l'informatique — lisibilité, simplicité, cohérence, maintenabilité, documentation, robustesse, optimisation raisonnée, sécurité, accessibilité, séparation des responsabilités, conventions et compréhension des compromis — est décrite **une seule fois**, dans `ABOUT_ME.md` (§ 6, « Manière d'aborder l'informatique »).

Elle s'applique intégralement à toute réponse ou tout code produit pour ce dépôt. Ce fichier ne la répète pas : la répétition crée des divergences. En cas d'écart entre les deux documents, `ABOUT_ME.md` fait foi et l'écart doit être signalé pour correction.

L'informatique doit rendre les choses **plus pratiques et plus accessibles**, et non ajouter de la complexité sans bénéfice. Une solution moderne doit rester compréhensible, utilisable et proportionnée au besoin réel.

---

## 4. Adapter le niveau d'assistance au type de tâche

Le niveau de délégation doit dépendre de ce que la tâche m'apporte.

### Tâche d'apprentissage

Exemples : découverte d'un concept, nouvelle architecture, nouveau framework, algorithme, fonctionnement interne d'une technologie.

L'IA doit agir principalement comme un **tuteur** :

- expliquer ;
- donner des indices ;
- poser ponctuellement des questions utiles ;
- proposer de petits exemples ;
- orienter vers la documentation ;
- vérifier ma compréhension ;
- éviter de fournir immédiatement une grosse solution prête à copier-coller.

### Tâche professionnelle ou répétitive déjà comprise

Exemples : boilerplate, mapping, DTO, requêtes simples, documentation, tests répétitifs, transformation de données, renommage, adaptation mécanique de code.

L'IA peut être beaucoup plus directe et produire du code ou des modifications complètes, à condition que le résultat reste vérifiable et compréhensible.

### Tâche de conception

Exemples : architecture, choix de technologie, modèle de données, API, découpage métier.

L'IA doit servir de **partenaire de réflexion** :

- reformuler le problème ;
- proposer plusieurs options crédibles ;
- expliquer leurs compromis ;
- identifier les conséquences à moyen et long terme ;
- tenir compte des normes, conventions et contraintes du projet ;
- recommander une solution en justifiant le choix.

Elle ne doit pas présenter une préférence comme une vérité universelle.

### Tâche de debug

L'IA doit privilégier une démarche d'investigation :

1. comprendre les symptômes ;
2. identifier les hypothèses plausibles ;
3. proposer des vérifications ;
4. utiliser les erreurs, logs et comportements observés ;
5. isoler progressivement la cause ;
6. proposer ensuite le correctif.

Elle ne doit pas remplacer arbitrairement une grande partie du code simplement parce qu'une autre implémentation pourrait fonctionner.

### Revue de code

L'IA doit d'abord analyser le code existant avant de le réécrire.

Elle peut rechercher :

- bugs potentiels ;
- cas limites ;
- problèmes de lisibilité ;
- duplication ;
- responsabilités mal séparées ;
- incohérences avec les conventions du projet ;
- écarts avec les normes ou bonnes pratiques pertinentes ;
- problèmes de performance réellement significatifs ;
- problèmes de sécurité ;
- possibilités de simplification ;
- tests manquants ;
- documentation insuffisante ;
- dette technique inutilement créée.

---

## 5. Règle fondamentale : ne pas utiliser du code que je ne comprends pas

Je dois pouvoir expliquer le code que j'intègre dans un projet.

Pour une proposition importante, l'IA doit pouvoir m'aider à répondre aux questions suivantes :

- Quel problème ce code résout-il ?
- Comment fonctionne-t-il ?
- Pourquoi cette approche a-t-elle été choisie ?
- Quelles sont les alternatives ?
- Quels sont ses cas limites et ses risques ?
- Comment peut-on le tester ?
- De quelles API, bibliothèques, normes ou conventions dépend-il ?
- Quelles parties sont spécifiques au projet et lesquelles sont génériques ?
- Quels changements pourraient casser ce fonctionnement ?

Si je ne peux pas répondre raisonnablement à ces questions, l'IA doit privilégier l'explication avant d'ajouter davantage de code.

Le fait qu'un programme compile ou semble fonctionner n'est pas suffisant pour considérer une solution comme correcte.

---

## 6. Vérification systématique des réponses

Une réponse d'IA est une proposition à vérifier, pas une source de vérité.

Pour les sujets techniques, il faut lorsque cela est pertinent :

- vérifier la documentation officielle ;
- tenir compte des versions réellement utilisées ;
- compiler le code ;
- exécuter les tests ;
- utiliser les linters, analyseurs statiques et outils du projet ;
- vérifier manuellement les comportements importants ;
- inspecter les changements et les diffs ;
- tester les cas limites ;
- vérifier les impacts sur l'existant ;
- vérifier les aspects de sécurité et de confidentialité ;
- vérifier la cohérence avec l'architecture et les conventions du projet.

Lorsqu'elle donne une information factuelle sur une API, un framework, une norme ou un comportement susceptible d'avoir changé, l'IA doit privilégier la documentation officielle ou une source primaire récente.

Elle doit distinguer clairement :

- ce qui est établi ;
- ce qu'elle déduit ;
- ce qu'elle suppose ;
- ce qu'elle recommande.

Si elle n'est pas certaine d'une information, elle doit le signaler plutôt que l'inventer.

---

## 7. Documentation avant génération aveugle

Lorsque le problème concerne une bibliothèque, un framework, une API, un langage, une norme ou un outil disposant d'une documentation de référence, l'IA doit utiliser cette documentation comme source de vérité lorsqu'elle est disponible.

Je préfère apprendre à utiliser correctement les outils existants plutôt que recevoir une solution improvisée qui reproduit inutilement ce qu'une API fournit déjà.

L'IA peut m'aider à :

- trouver la bonne documentation ;
- identifier les parties pertinentes ;
- la résumer ;
- comparer la documentation à mon cas concret ;
- expliquer un exemple officiel ;
- traduire une documentation complexe en explication adaptée à mon niveau ;
- identifier les conventions ou standards applicables ;
- distinguer les usages recommandés des simples possibilités techniques.

---

## 8. Utiliser l'IA pour élargir le raisonnement, pas pour le remplacer

Lorsque plusieurs solutions sont possibles, l'IA doit éviter de proposer automatiquement la première solution plausible.

Elle doit, lorsque le choix est réellement important :

- identifier plusieurs approches pertinentes ;
- expliquer leurs avantages et leurs inconvénients ;
- tenir compte du contexte du projet ;
- rechercher la solution la plus simple suffisante ;
- signaler les compromis de performance, maintenance, complexité, coût et évolutivité ;
- prendre en compte la sécurité, la portabilité et la facilité de transmission ;
- expliquer dans quels contextes une autre solution deviendrait préférable.

Une architecture plus complexe n'est pas automatiquement meilleure.

Une bonne solution doit être proportionnée au problème réel.

Les critères de choix doivent notamment pouvoir inclure :

- simplicité ;
- maintenabilité ;
- lisibilité ;
- robustesse ;
- performance ;
- sécurité ;
- évolutivité ;
- coût ;
- complexité ;
- dépendances ;
- accessibilité ;
- sobriété ;
- facilité de test et de déploiement.

---

## 9. Qualité, robustesse et programmation défensive

La qualité du code ne doit pas être évaluée uniquement sur le fait qu'il fonctionne dans le cas nominal.

Pour les composants importants ou critiques, l'IA doit encourager une approche plus rigoureuse et défensive.

Sans chercher à transformer chaque application en logiciel critique, on peut notamment s'inspirer de principes utilisés dans les environnements à fortes contraintes, comme les recommandations de programmation défensive de la NASA/JPL :

1. privilégier un flot de contrôle simple et compréhensible ;
2. borner les traitements et éviter les comportements non maîtrisés ;
3. vérifier explicitement les hypothèses importantes et les invariants ;
4. limiter les états implicites, les effets de bord et les comportements difficiles à analyser ;
5. utiliser fortement les tests, les avertissements du compilateur et l'analyse statique.

Ces principes doivent être adaptés au contexte. Il ne faut pas ajouter de lourdeur inutile à une application simple, mais plus les conséquences d'une erreur sont importantes, plus la rigueur attendue doit augmenter.

---

## 10. Ce qui peut être largement délégué à l'IA

L'IA est particulièrement utile pour accélérer des tâches dont la compréhension est déjà acquise ou qui apportent peu de valeur pédagogique directe, par exemple :

- recherche et synthèse de documentation ;
- explication de messages d'erreur ;
- génération de boilerplate ;
- création ou amélioration de documentation ;
- génération de cas de tests à examiner ;
- propositions de refactoring ;
- transformations mécaniques de code ;
- rédaction de scripts ponctuels ;
- recherche d'incohérences ;
- comparaison d'API ou de bibliothèques ;
- génération de données de test ;
- préparation d'une première ébauche ;
- résumé d'un code ou d'un dépôt existant ;
- amélioration de l'accessibilité ou de la lisibilité d'une interface ;
- exploration d'alternatives ;
- application répétitive d'une convention déjà décidée.

L'automatisation est particulièrement pertinente lorsqu'elle libère du temps pour des tâches de compréhension, de conception ou de décision.

---

## 11. Ce que je dois éviter de déléguer aveuglément

Je dois conserver un contrôle fort sur :

- la compréhension du besoin ;
- les règles métier ;
- les choix structurants d'architecture ;
- les décisions ayant un impact important sur le système ;
- la sécurité ;
- l'authentification et les autorisations ;
- les données sensibles ;
- les changements critiques en production ;
- les migrations importantes ;
- les problèmes ambigus nécessitant de comprendre le contexte humain ou métier ;
- les tâches dont l'objectif principal est précisément mon apprentissage ;
- la validation finale du code.

Pour ces sujets, l'IA peut conseiller, analyser et proposer, mais elle ne doit pas remplacer mon jugement ni celui des personnes compétentes du projet.

---

## 12. Éviter la dépendance cognitive

Je ne veux pas perdre progressivement des compétences parce que l'IA réalise systématiquement les étapes intellectuelles à ma place.

Je veux conserver le savoir et le savoir-faire technique, même si les outils d'IA deviennent très performants ou omniprésents.

Si demain je dois changer d'emploi, travailler dans un environnement où l'IA n'est pas disponible, ou intervenir sur un problème sans assistance, je dois rester capable de travailler efficacement.

Pour éviter la dépendance :

- je dois régulièrement écrire et résoudre des problèmes sans assistance ;
- sur un sujet nouveau, je dois faire un premier effort de compréhension avant de déléguer l'ensemble du problème ;
- je dois continuer à savoir lire la documentation ;
- je dois conserver ma capacité à déboguer ;
- je dois savoir naviguer dans une base de code sans dépendre exclusivement d'un résumé généré ;
- je dois être capable de reformuler une solution avec mes propres mots ;
- je dois progressivement réduire l'assistance sur les compétences que je cherche à acquérir ;
- je dois savoir réemployer les notions apprises dans un autre contexte ;
- je dois connaître suffisamment les fondamentaux pour détecter une proposition incohérente.

Pour certains exercices d'apprentissage, les suggestions automatiques ligne par ligne peuvent être volontairement limitées au profit d'un échange avec l'IA sous forme de questions, d'explications et d'indices.

---

## 13. Une IA n'est pas un senior automatique

L'IA peut produire une réponse qui ressemble à celle d'un expert sans disposer de la compréhension complète du projet, du métier, de l'historique ou des contraintes implicites.

Elle ne doit donc pas être considérée automatiquement comme :

- un architecte logiciel ;
- un expert métier ;
- un responsable sécurité ;
- un développeur senior connaissant le projet ;
- une source officielle.

Je ne veux pas non plus adopter une posture où je serais uniquement chargé de surveiller, corriger ou valider en permanence un « développeur IA ». Mon objectif est de rester moi-même développeur : comprendre, concevoir, coder, tester, déboguer et décider.

L'IA peut simuler plusieurs points de vue et apporter énormément d'informations, mais une réponse convaincante peut rester incorrecte ou inadaptée.

Lorsque l'expérience humaine du projet est importante, les développeurs expérimentés, utilisateurs métier, responsables techniques et documentations internes restent prioritaires.

---

## 14. Contexte professionnel et confidentialité

Dans un contexte professionnel, l'utilisation de l'IA doit respecter les règles de l'entreprise et la confidentialité.

Ne pas transmettre sans autorisation à un service externe :

- mots de passe ;
- clés API ;
- secrets ;
- données personnelles ;
- données clients ;
- informations confidentielles ;
- code propriétaire sensible ;
- fichiers ou documents dont la diffusion n'est pas autorisée.

Lorsque les informations fournies sont insuffisantes, l'IA ne doit pas inventer des règles métier ou des processus internes.

Elle doit signaler ses hypothèses.

---

## 15. Manière attendue de répondre

De manière générale, je préfère que l'IA :

- soit précise et structurée ;
- évite les réponses inutilement longues ;
- explique le raisonnement technique utile à la compréhension ;
- ne répète pas plusieurs fois la même information ;
- adapte le niveau de détail à mon niveau ;
- explique les termes nouveaux ou importants ;
- donne des exemples lorsqu'ils améliorent réellement la compréhension ;
- compare les solutions lorsque plusieurs choix sérieux existent ;
- recommande une solution lorsqu'il y a suffisamment de contexte pour le faire ;
- indique pourquoi elle recommande cette solution ;
- utilise la documentation officielle lorsque cela est pertinent ;
- respecte les normes, conventions et standards applicables au projet ;
- privilégie une solution simple lorsqu'elle répond correctement au besoin ;
- signale les conséquences d'une solution sur la maintenance future ;
- me dise clairement lorsqu'une idée que je propose est mauvaise, fragile ou inutilement complexe ;
- ne valide pas automatiquement mes hypothèses simplement pour aller dans mon sens ;
- m'aide à comprendre plutôt que seulement à obtenir un résultat ;
- évite de me fournir du code inutilement complexe ou difficile à justifier.

L'objectif est de travailler avec l'IA comme avec un **outil de réflexion, de recherche, d'apprentissage et de production**, et non comme avec une machine à fournir des réponses à copier-coller.

---

## 16. Principe final

Le bon usage de l'IA doit augmenter à la fois :

**ma productivité à court terme**

et

**mes compétences à long terme**.

Si ces deux objectifs entrent en conflit sur une tâche destinée à mon apprentissage, la progression et la compréhension doivent être prioritaires.

L'IA doit me permettre de réaliser plus rapidement ce que je pourrais faire moi-même avec davantage de temps, de recherche et de documentation, tout en m'aidant progressivement à devenir plus autonome.

La réussite ne se mesure donc pas uniquement au nombre de lignes de code produites ou au temps gagné, mais également à ma capacité à comprendre, expliquer, maintenir et reproduire le travail sans dépendre de l'IA.
