-- =====================================================================
-- V0001 — Schéma initial de la plateforme d'apprentissage
-- ---------------------------------------------------------------------
-- SGBD cible  : PostgreSQL 16 (15 minimum, pour UNIQUE NULLS NOT DISTINCT)
-- Conventions : SQL_STYLE_GUIDE.md
-- Description : MODELE_METIER.md (mêmes sections, même ordre)
-- [PG]        : syntaxe propre à PostgreSQL, à adapter si changement de SGBD
-- =====================================================================

CREATE SCHEMA IF NOT EXISTS app;
SET search_path TO app;

-- Maintient updated_at quel que soit le client qui modifie la ligne
-- (API C#, script d'administration, outil BI). [PG]
CREATE FUNCTION set_updated_at()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at := CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;

-- =====================================================================
-- 0. Référentiels
-- =====================================================================

-- Pays ISO 3166-1 alpha-2. Clé naturelle : code international stable.
CREATE TABLE country
(
    country_code CHAR(2) NOT NULL,
    name VARCHAR(100) NOT NULL,
    CONSTRAINT pk_country PRIMARY KEY (country_code),
    CONSTRAINT ck_country_country_code CHECK (country_code = UPPER(country_code))
);

-- =====================================================================
-- 1. Comptes & social
-- =====================================================================

-- account_type :
--   standard      personne, connexion par email + mot de passe
--   anonymous     compte sans identifiants (essai, appareil)
--   organization  personne morale (établissement, association)
-- « Vérifié » n'est pas un type : c'est l'existence d'une ligne
-- dans verified_identity (une seule source de vérité).
-- archived_at : NULL = compte actif. Suppression RGPD = anonymisation
-- (username remplacé, email et password_hash à NULL) + archived_at.
CREATE TABLE account
(
    account_id BIGINT GENERATED ALWAYS AS IDENTITY,
    account_type VARCHAR(20) NOT NULL DEFAULT 'standard',
    username VARCHAR(50) NOT NULL,
    email VARCHAR(254) NULL,
    email_verified_at TIMESTAMPTZ NULL,
    password_hash VARCHAR(255) NULL, -- Argon2id au format PHC (paramètres + sel inclus) ; poivre côté application
    bio TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_at TIMESTAMPTZ NULL,
    archived_at TIMESTAMPTZ NULL,
    CONSTRAINT pk_account PRIMARY KEY (account_id),
    CONSTRAINT ck_account_account_type CHECK (account_type IN ('standard', 'anonymous', 'organization')),
    -- Un compte actif non anonyme doit pouvoir se connecter.
    CONSTRAINT ck_account_credentials CHECK
    (
        account_type = 'anonymous'
        OR archived_at IS NOT NULL
        OR (email IS NOT NULL AND password_hash IS NOT NULL)
    ),
    CONSTRAINT ck_account_email_verified_at CHECK (email_verified_at IS NULL OR email IS NOT NULL)
);

-- Unicité insensible à la casse : « Alice » et « alice » sont le même pseudo. [PG] index sur expression
CREATE UNIQUE INDEX ux_account_username ON account (LOWER(username));
CREATE UNIQUE INDEX ux_account_email ON account (LOWER(email));

CREATE TRIGGER trg_account_set_updated_at
    BEFORE UPDATE ON account
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Identité vérifiée d'une personne (extension 1:1 de account).
-- Minimisation RGPD : uniquement ce qu'exige une attestation de compétences.
CREATE TABLE verified_identity
(
    account_id BIGINT NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    birth_date DATE NOT NULL,
    birth_place VARCHAR(100) NULL,
    verified_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_verified_identity PRIMARY KEY (account_id),
    CONSTRAINT fk_verified_identity_account_id FOREIGN KEY (account_id) REFERENCES account (account_id) ON DELETE CASCADE
);

CREATE TRIGGER trg_verified_identity_set_updated_at
    BEFORE UPDATE ON verified_identity
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Amitié : relation symétrique stockée une seule fois.
-- lower_account_id < higher_account_id empêche le doublon (A,B) / (B,A).
-- friendship_status : pending (demande envoyée), accepted.
-- Le blocage n'est pas un état de l'amitié : il est orienté (qui bloque qui),
-- voir account_block.
CREATE TABLE friendship
(
    lower_account_id BIGINT NOT NULL,
    higher_account_id BIGINT NOT NULL,
    requester_account_id BIGINT NOT NULL,
    friendship_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    requested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    accepted_at TIMESTAMPTZ NULL,
    CONSTRAINT pk_friendship PRIMARY KEY (lower_account_id, higher_account_id),
    CONSTRAINT fk_friendship_lower_account_id FOREIGN KEY (lower_account_id) REFERENCES account (account_id) ON DELETE CASCADE,
    CONSTRAINT fk_friendship_higher_account_id FOREIGN KEY (higher_account_id) REFERENCES account (account_id) ON DELETE CASCADE,
    CONSTRAINT ck_friendship_account_order CHECK (lower_account_id < higher_account_id),
    CONSTRAINT ck_friendship_requester CHECK (requester_account_id IN (lower_account_id, higher_account_id)),
    CONSTRAINT ck_friendship_friendship_status CHECK (friendship_status IN ('pending', 'accepted')),
    CONSTRAINT ck_friendship_accepted_at CHECK
    (
        (friendship_status = 'pending' AND accepted_at IS NULL)
        OR (friendship_status = 'accepted' AND accepted_at IS NOT NULL)
    )
);

CREATE INDEX ix_friendship_higher_account_id ON friendship (higher_account_id);

-- Blocage orienté : blocker_account_id ne veut plus d'interaction avec blocked_account_id.
CREATE TABLE account_block
(
    blocker_account_id BIGINT NOT NULL,
    blocked_account_id BIGINT NOT NULL,
    blocked_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_account_block PRIMARY KEY (blocker_account_id, blocked_account_id),
    CONSTRAINT fk_account_block_blocker_account_id FOREIGN KEY (blocker_account_id) REFERENCES account (account_id) ON DELETE CASCADE,
    CONSTRAINT fk_account_block_blocked_account_id FOREIGN KEY (blocked_account_id) REFERENCES account (account_id) ON DELETE CASCADE,
    CONSTRAINT ck_account_block_not_self CHECK (blocker_account_id <> blocked_account_id)
);

CREATE INDEX ix_account_block_blocked_account_id ON account_block (blocked_account_id);

-- Groupe d'étude : classe d'un professeur, groupe d'amis, club…
-- created_by_account_id garde l'historique ; le propriétaire actuel
-- est le membre de rôle owner.
CREATE TABLE study_group
(
    study_group_id BIGINT GENERATED ALWAYS AS IDENTITY,
    created_by_account_id BIGINT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMPTZ NULL,
    CONSTRAINT pk_study_group PRIMARY KEY (study_group_id),
    CONSTRAINT fk_study_group_created_by_account_id FOREIGN KEY (created_by_account_id) REFERENCES account (account_id) ON DELETE SET NULL
);

CREATE INDEX ix_study_group_created_by_account_id ON study_group (created_by_account_id);

CREATE TRIGGER trg_study_group_set_updated_at
    BEFORE UPDATE ON study_group
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- member_role   : owner, admin, member (ce que le membre a le droit de faire)
-- member_status : invited, active, banned (où en est l'adhésion)
-- Rôle et statut sont séparés : un admin peut être banni, un invité a déjà un rôle.
CREATE TABLE study_group_member
(
    study_group_id BIGINT NOT NULL,
    account_id BIGINT NOT NULL,
    member_role VARCHAR(20) NOT NULL DEFAULT 'member',
    member_status VARCHAR(20) NOT NULL DEFAULT 'invited',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    joined_at TIMESTAMPTZ NULL,
    CONSTRAINT pk_study_group_member PRIMARY KEY (study_group_id, account_id),
    CONSTRAINT fk_study_group_member_study_group_id FOREIGN KEY (study_group_id) REFERENCES study_group (study_group_id) ON DELETE CASCADE,
    CONSTRAINT fk_study_group_member_account_id FOREIGN KEY (account_id) REFERENCES account (account_id) ON DELETE CASCADE,
    CONSTRAINT ck_study_group_member_member_role CHECK (member_role IN ('owner', 'admin', 'member')),
    CONSTRAINT ck_study_group_member_member_status CHECK (member_status IN ('invited', 'active', 'banned')),
    CONSTRAINT ck_study_group_member_joined_at CHECK (member_status <> 'active' OR joined_at IS NOT NULL)
);

-- Un seul propriétaire par groupe.
CREATE UNIQUE INDEX ux_study_group_member_owner ON study_group_member (study_group_id) WHERE member_role = 'owner';
CREATE INDEX ix_study_group_member_account_id ON study_group_member (account_id);

-- =====================================================================
-- 2. Contenu pédagogique
-- =====================================================================

-- publication_status (commun à exercise et question) :
--   private         visible par l'auteur, et par un groupe via un devoir
--   pending_review  proposé à la communauté, en attente de validation
--   published       validé, visible par tous
--   rejected        refusé (doublon, droit d'auteur, thème interdit…)
--
-- exercise_type : quiz, deck, mixed
-- question_count : nombre de questions tirées à chaque passage ; NULL = toutes.
CREATE TABLE exercise
(
    exercise_id BIGINT GENERATED ALWAYS AS IDENTITY,
    author_account_id BIGINT NOT NULL,
    exercise_type VARCHAR(20) NOT NULL,
    title VARCHAR(200) NOT NULL,
    description TEXT NULL,
    question_count INT NULL,
    publication_status VARCHAR(20) NOT NULL DEFAULT 'private',
    source TEXT NULL,
    resource_uri VARCHAR(500) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMPTZ NULL,
    CONSTRAINT pk_exercise PRIMARY KEY (exercise_id),
    CONSTRAINT fk_exercise_author_account_id FOREIGN KEY (author_account_id) REFERENCES account (account_id),
    CONSTRAINT ck_exercise_exercise_type CHECK (exercise_type IN ('quiz', 'deck', 'mixed')),
    CONSTRAINT ck_exercise_question_count CHECK (question_count > 0),
    CONSTRAINT ck_exercise_publication_status CHECK (publication_status IN ('private', 'pending_review', 'published', 'rejected'))
);

CREATE INDEX ix_exercise_author_account_id ON exercise (author_account_id);

CREATE TRIGGER trg_exercise_set_updated_at
    BEFORE UPDATE ON exercise
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- question_type :
--   flashcard        carte recto / verso, auto-évaluée
--   true_false       vrai / faux
--   single_choice    QCU : une seule bonne réponse
--   multiple_choice  QCM : une ou plusieurs bonnes réponses
--   short_answer     question simple, réponse courte saisie
--   multi_answer     question appelant plusieurs réponses saisies
--   fill_in_blank    texte à trous
--   ordering         classement / remise dans l'ordre
--   error_spotting   repérage d'erreur (+ correction éventuelle)
--   span_selection   repérage simple d'un passage
--   open_ended       question ouverte, corrigée par soi-même ou un tiers
-- instruction : consigne (« calculer de tête », « traduire »…).
CREATE TABLE question
(
    question_id BIGINT GENERATED ALWAYS AS IDENTITY,
    author_account_id BIGINT NOT NULL,
    question_type VARCHAR(20) NOT NULL,
    instruction TEXT NULL,
    content TEXT NOT NULL,
    publication_status VARCHAR(20) NOT NULL DEFAULT 'private',
    source TEXT NULL,
    resource_uri VARCHAR(500) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMPTZ NULL,
    CONSTRAINT pk_question PRIMARY KEY (question_id),
    CONSTRAINT fk_question_author_account_id FOREIGN KEY (author_account_id) REFERENCES account (account_id),
    CONSTRAINT ck_question_question_type CHECK
    (
        question_type IN
        (
            'flashcard',
            'true_false',
            'single_choice',
            'multiple_choice',
            'short_answer',
            'multi_answer',
            'fill_in_blank',
            'ordering',
            'error_spotting',
            'span_selection',
            'open_ended'
        )
    ),
    CONSTRAINT ck_question_publication_status CHECK (publication_status IN ('private', 'pending_review', 'published', 'rejected'))
);

CREATE INDEX ix_question_author_account_id ON question (author_account_id);

CREATE TRIGGER trg_question_set_updated_at
    BEFORE UPDATE ON question
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Une question peut servir dans plusieurs exercices.
-- sort_order : ordre d'affichage dans l'exercice (si pas de tirage aléatoire).
CREATE TABLE exercise_question
(
    exercise_id BIGINT NOT NULL,
    question_id BIGINT NOT NULL,
    sort_order INT NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_exercise_question PRIMARY KEY (exercise_id, question_id),
    CONSTRAINT fk_exercise_question_exercise_id FOREIGN KEY (exercise_id) REFERENCES exercise (exercise_id) ON DELETE CASCADE,
    CONSTRAINT fk_exercise_question_question_id FOREIGN KEY (question_id) REFERENCES question (question_id) ON DELETE CASCADE,
    CONSTRAINT ck_exercise_question_sort_order CHECK (sort_order >= 0)
);

CREATE INDEX ix_exercise_question_question_id ON exercise_question (question_id);

-- Élément de réponse attendu ou proposé pour une question.
-- answer_type :
--   choice         proposition (QCM, QCU, vrai/faux) ; is_correct indique la bonne
--   solution       réponse attendue ou correction (flashcard, réponse courte, correction d'erreur)
--   gap            contenu d'un trou ; sort_order = numéro du trou
--   ordering_item  élément à classer ; sort_order = position attendue
--   span           passage à repérer dans question.content : [span_start, span_end[ en caractères
-- Pour les types sans distracteur (solution, gap…), is_correct = TRUE.
CREATE TABLE answer
(
    answer_id BIGINT GENERATED ALWAYS AS IDENTITY,
    question_id BIGINT NOT NULL,
    answer_type VARCHAR(20) NOT NULL,
    content TEXT NULL,
    explanation TEXT NULL,
    is_correct BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INT NULL,
    span_start INT NULL,
    span_end INT NULL,
    resource_uri VARCHAR(500) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_answer PRIMARY KEY (answer_id),
    CONSTRAINT fk_answer_question_id FOREIGN KEY (question_id) REFERENCES question (question_id) ON DELETE CASCADE,
    CONSTRAINT ck_answer_answer_type CHECK (answer_type IN ('choice', 'solution', 'gap', 'ordering_item', 'span')),
    CONSTRAINT ck_answer_sort_order CHECK (answer_type NOT IN ('gap', 'ordering_item') OR sort_order IS NOT NULL),
    CONSTRAINT ck_answer_span CHECK
    (
        (answer_type = 'span' AND span_start >= 0 AND span_end > span_start)
        OR (answer_type <> 'span' AND span_start IS NULL AND span_end IS NULL)
    )
);

CREATE INDEX ix_answer_question_id ON answer (question_id);

CREATE TRIGGER trg_answer_set_updated_at
    BEFORE UPDATE ON answer
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- =====================================================================
-- 3. Tags
-- =====================================================================

-- Arborescence de compétences. Le même nom peut exister sous des parents
-- différents : « Histoire > Sport » ≠ « Sport » (racine).
-- country_code : contexte national du tag (histoire de France ≠ histoire
-- des États-Unis), pour ne pas mélanger les Elo ; NULL = tag universel.
-- On archive un tag, on ne le supprime pas (Elo et historiques en dépendent).
CREATE TABLE tag
(
    tag_id BIGINT GENERATED ALWAYS AS IDENTITY,
    parent_tag_id BIGINT NULL,
    country_code CHAR(2) NULL,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMPTZ NULL,
    CONSTRAINT pk_tag PRIMARY KEY (tag_id),
    CONSTRAINT fk_tag_parent_tag_id FOREIGN KEY (parent_tag_id) REFERENCES tag (tag_id),
    CONSTRAINT fk_tag_country_code FOREIGN KEY (country_code) REFERENCES country (country_code),
    -- NULLS NOT DISTINCT : deux tags racine « Sport » seraient sinon acceptés,
    -- car par défaut deux NULL ne sont jamais égaux pour UNIQUE. [PG] 15+
    CONSTRAINT uq_tag_parent_name_country UNIQUE NULLS NOT DISTINCT (parent_tag_id, name, country_code),
    CONSTRAINT ck_tag_not_own_parent CHECK (parent_tag_id <> tag_id)
);

-- Associations : une table par type d'entité (pas d'association polymorphe),
-- pour que chaque lien soit protégé par une vraie clé étrangère.

-- account_tag : l'utilisateur pratique ce tag (créée automatiquement à sa
-- première réponse à une question portant ce tag). Porte son Elo.
CREATE TABLE account_tag
(
    account_id BIGINT NOT NULL,
    tag_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_account_tag PRIMARY KEY (account_id, tag_id),
    CONSTRAINT fk_account_tag_account_id FOREIGN KEY (account_id) REFERENCES account (account_id) ON DELETE CASCADE,
    CONSTRAINT fk_account_tag_tag_id FOREIGN KEY (tag_id) REFERENCES tag (tag_id)
);

CREATE INDEX ix_account_tag_tag_id ON account_tag (tag_id);

CREATE TABLE question_tag
(
    question_id BIGINT NOT NULL,
    tag_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_question_tag PRIMARY KEY (question_id, tag_id),
    CONSTRAINT fk_question_tag_question_id FOREIGN KEY (question_id) REFERENCES question (question_id) ON DELETE CASCADE,
    CONSTRAINT fk_question_tag_tag_id FOREIGN KEY (tag_id) REFERENCES tag (tag_id)
);

CREATE INDEX ix_question_tag_tag_id ON question_tag (tag_id);

CREATE TABLE exercise_tag
(
    exercise_id BIGINT NOT NULL,
    tag_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_exercise_tag PRIMARY KEY (exercise_id, tag_id),
    CONSTRAINT fk_exercise_tag_exercise_id FOREIGN KEY (exercise_id) REFERENCES exercise (exercise_id) ON DELETE CASCADE,
    CONSTRAINT fk_exercise_tag_tag_id FOREIGN KEY (tag_id) REFERENCES tag (tag_id)
);

CREATE INDEX ix_exercise_tag_tag_id ON exercise_tag (tag_id);

CREATE TABLE study_group_tag
(
    study_group_id BIGINT NOT NULL,
    tag_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_study_group_tag PRIMARY KEY (study_group_id, tag_id),
    CONSTRAINT fk_study_group_tag_study_group_id FOREIGN KEY (study_group_id) REFERENCES study_group (study_group_id) ON DELETE CASCADE,
    CONSTRAINT fk_study_group_tag_tag_id FOREIGN KEY (tag_id) REFERENCES tag (tag_id)
);

CREATE INDEX ix_study_group_tag_tag_id ON study_group_tag (tag_id);

-- =====================================================================
-- 4. Devoirs & pratique
-- =====================================================================

-- Devoir : un exercice donné à un groupe sur une période.
-- pass_threshold_ratio : objectif = barre minimum à atteindre
-- (0.850 pour 17/20), pas la moyenne du groupe ; NULL = pas d'objectif.
CREATE TABLE assignment
(
    assignment_id BIGINT GENERATED ALWAYS AS IDENTITY,
    study_group_id BIGINT NOT NULL,
    exercise_id BIGINT NOT NULL,
    assigned_by_account_id BIGINT NOT NULL,
    instructions TEXT NULL,
    opens_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    due_at TIMESTAMPTZ NULL,
    pass_threshold_ratio NUMERIC(4, 3) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMPTZ NULL,
    CONSTRAINT pk_assignment PRIMARY KEY (assignment_id),
    CONSTRAINT fk_assignment_study_group_id FOREIGN KEY (study_group_id) REFERENCES study_group (study_group_id) ON DELETE CASCADE,
    CONSTRAINT fk_assignment_exercise_id FOREIGN KEY (exercise_id) REFERENCES exercise (exercise_id),
    CONSTRAINT fk_assignment_assigned_by_account_id FOREIGN KEY (assigned_by_account_id) REFERENCES account (account_id),
    CONSTRAINT ck_assignment_due_at CHECK (due_at > opens_at),
    CONSTRAINT ck_assignment_pass_threshold_ratio CHECK (pass_threshold_ratio BETWEEN 0 AND 1)
);

CREATE INDEX ix_assignment_study_group_id ON assignment (study_group_id);
CREATE INDEX ix_assignment_exercise_id ON assignment (exercise_id);
CREATE INDEX ix_assignment_assigned_by_account_id ON assignment (assigned_by_account_id);

CREATE TRIGGER trg_assignment_set_updated_at
    BEFORE UPDATE ON assignment
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Séance de pratique : regroupe les tentatives faites d'une traite.
-- session_type :
--   free             questions choisies librement
--   exercise         passage d'un exercice (exercise_id obligatoire)
--   assignment       passage d'un devoir (assignment_id obligatoire, l'exercice vient du devoir)
--   daily_challenge  défi quotidien
--   placement_test   test de niveau initial (tag_id conseillé)
--   srs_review       révision des questions dues (répétition espacée)
CREATE TABLE practice_session
(
    practice_session_id BIGINT GENERATED ALWAYS AS IDENTITY,
    account_id BIGINT NOT NULL,
    session_type VARCHAR(20) NOT NULL,
    exercise_id BIGINT NULL,
    assignment_id BIGINT NULL,
    tag_id BIGINT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at TIMESTAMPTZ NULL,
    CONSTRAINT pk_practice_session PRIMARY KEY (practice_session_id),
    CONSTRAINT fk_practice_session_account_id FOREIGN KEY (account_id) REFERENCES account (account_id) ON DELETE CASCADE,
    CONSTRAINT fk_practice_session_exercise_id FOREIGN KEY (exercise_id) REFERENCES exercise (exercise_id),
    CONSTRAINT fk_practice_session_assignment_id FOREIGN KEY (assignment_id) REFERENCES assignment (assignment_id),
    CONSTRAINT fk_practice_session_tag_id FOREIGN KEY (tag_id) REFERENCES tag (tag_id),
    CONSTRAINT ck_practice_session_session_type CHECK
    (
        session_type IN ('free', 'exercise', 'assignment', 'daily_challenge', 'placement_test', 'srs_review')
    ),
    CONSTRAINT ck_practice_session_context CHECK
    (
        (session_type = 'assignment' AND assignment_id IS NOT NULL AND exercise_id IS NULL)
        OR (session_type = 'exercise' AND exercise_id IS NOT NULL AND assignment_id IS NULL)
        OR (session_type IN ('free', 'daily_challenge', 'placement_test', 'srs_review') AND assignment_id IS NULL)
    ),
    CONSTRAINT ck_practice_session_finished_at CHECK (finished_at >= started_at)
);

CREATE INDEX ix_practice_session_account_id_started_at ON practice_session (account_id, started_at);
CREATE INDEX ix_practice_session_exercise_id ON practice_session (exercise_id);
CREATE INDEX ix_practice_session_assignment_id ON practice_session (assignment_id);
CREATE INDEX ix_practice_session_tag_id ON practice_session (tag_id);

-- Tentative : la réponse d'un utilisateur à une question.
-- L'utilisateur se retrouve par la séance (pas de account_id en double).
-- score_ratio  : justesse objective, 0 = faux, 1 = juste, entre les deux = partiel ;
--                c'est le score réel R du calcul Elo.
-- recall_grade : qualité de rappel SM-2 (0 à 5), auto-évaluée pour une flashcard ;
--                NULL si la question n'est pas suivie en répétition espacée.
CREATE TABLE attempt
(
    attempt_id BIGINT GENERATED ALWAYS AS IDENTITY,
    practice_session_id BIGINT NOT NULL,
    question_id BIGINT NOT NULL,
    given_answer TEXT NULL,
    score_ratio NUMERIC(4, 3) NOT NULL,
    recall_grade SMALLINT NULL,
    duration_ms INT NULL,
    answered_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_attempt PRIMARY KEY (attempt_id),
    CONSTRAINT fk_attempt_practice_session_id FOREIGN KEY (practice_session_id) REFERENCES practice_session (practice_session_id) ON DELETE CASCADE,
    CONSTRAINT fk_attempt_question_id FOREIGN KEY (question_id) REFERENCES question (question_id),
    CONSTRAINT ck_attempt_score_ratio CHECK (score_ratio BETWEEN 0 AND 1),
    CONSTRAINT ck_attempt_recall_grade CHECK (recall_grade BETWEEN 0 AND 5),
    CONSTRAINT ck_attempt_duration_ms CHECK (duration_ms >= 0)
);

CREATE INDEX ix_attempt_practice_session_id ON attempt (practice_session_id);
CREATE INDEX ix_attempt_question_id ON attempt (question_id);

-- =====================================================================
-- 5. Apprentissage adaptatif (SRS & Elo)
-- =====================================================================

-- État de répétition espacée SM-2, par utilisateur et par question (niveau micro).
-- srs_status : new, learning, review, relearning, suspended (vocabulaire Anki).
-- Les nombres de tentatives et de réussites ne sont pas stockés : ils se
-- calculent à partir de attempt (une seule source de vérité).
-- due_at : prochaine révision ; NULL = jamais vue.
CREATE TABLE srs_state
(
    account_id BIGINT NOT NULL,
    question_id BIGINT NOT NULL,
    srs_status VARCHAR(20) NOT NULL DEFAULT 'new',
    repetition_count INT NOT NULL DEFAULT 0,
    ease_factor REAL NOT NULL DEFAULT 2.5,
    interval_days INT NOT NULL DEFAULT 0,
    lapse_count INT NOT NULL DEFAULT 0,
    due_at TIMESTAMPTZ NULL,
    last_reviewed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_srs_state PRIMARY KEY (account_id, question_id),
    CONSTRAINT fk_srs_state_account_id FOREIGN KEY (account_id) REFERENCES account (account_id) ON DELETE CASCADE,
    CONSTRAINT fk_srs_state_question_id FOREIGN KEY (question_id) REFERENCES question (question_id) ON DELETE CASCADE,
    CONSTRAINT ck_srs_state_srs_status CHECK (srs_status IN ('new', 'learning', 'review', 'relearning', 'suspended')),
    CONSTRAINT ck_srs_state_repetition_count CHECK (repetition_count >= 0),
    CONSTRAINT ck_srs_state_ease_factor CHECK (ease_factor >= 1.3), -- plancher de l'algorithme SM-2
    CONSTRAINT ck_srs_state_interval_days CHECK (interval_days >= 0),
    CONSTRAINT ck_srs_state_lapse_count CHECK (lapse_count >= 0)
);

-- File de révision : « quelles questions sont dues pour cet utilisateur ? »
CREATE INDEX ix_srs_state_account_id_due_at ON srs_state (account_id, due_at);
CREATE INDEX ix_srs_state_question_id ON srs_state (question_id);

-- Elo d'un sujet dans un tag (niveau macro). Le sujet est soit un compte
-- (niveau), soit une question ou un exercice (difficulté) : arc exclusif,
-- exactement une des trois colonnes est renseignée.
-- Chaque Elo s'appuie sur l'association correspondante (account_tag,
-- question_tag ou exercise_tag) par une clé étrangère composite. Une clé
-- étrangère dont une colonne est NULL n'est pas vérifiée (MATCH SIMPLE,
-- comportement par défaut) : seule celle du sujet renseigné s'applique.
CREATE TABLE elo_rating
(
    elo_rating_id BIGINT GENERATED ALWAYS AS IDENTITY,
    tag_id BIGINT NOT NULL,
    account_id BIGINT NULL,
    question_id BIGINT NULL,
    exercise_id BIGINT NULL,
    rating DOUBLE PRECISION NOT NULL DEFAULT 1500,
    game_count INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_elo_rating PRIMARY KEY (elo_rating_id),
    CONSTRAINT fk_elo_rating_account_tag FOREIGN KEY (account_id, tag_id) REFERENCES account_tag (account_id, tag_id) ON DELETE CASCADE,
    CONSTRAINT fk_elo_rating_question_tag FOREIGN KEY (question_id, tag_id) REFERENCES question_tag (question_id, tag_id) ON DELETE CASCADE,
    CONSTRAINT fk_elo_rating_exercise_tag FOREIGN KEY (exercise_id, tag_id) REFERENCES exercise_tag (exercise_id, tag_id) ON DELETE CASCADE,
    CONSTRAINT ck_elo_rating_single_subject CHECK
    (
        (CASE WHEN account_id IS NOT NULL THEN 1 ELSE 0 END)
        + (CASE WHEN question_id IS NOT NULL THEN 1 ELSE 0 END)
        + (CASE WHEN exercise_id IS NOT NULL THEN 1 ELSE 0 END) = 1
    ),
    CONSTRAINT ck_elo_rating_game_count CHECK (game_count >= 0)
);

-- Un seul Elo par sujet et par tag.
CREATE UNIQUE INDEX ux_elo_rating_account_tag ON elo_rating (account_id, tag_id) WHERE account_id IS NOT NULL;
CREATE UNIQUE INDEX ux_elo_rating_question_tag ON elo_rating (question_id, tag_id) WHERE question_id IS NOT NULL;
CREATE UNIQUE INDEX ux_elo_rating_exercise_tag ON elo_rating (exercise_id, tag_id) WHERE exercise_id IS NOT NULL;

CREATE TRIGGER trg_elo_rating_set_updated_at
    BEFORE UPDATE ON elo_rating
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Historique des variations d'Elo (traçabilité).
-- attempt_id : tentative qui a provoqué la variation ; NULL si recalcul global.
CREATE TABLE elo_rating_history
(
    elo_rating_history_id BIGINT GENERATED ALWAYS AS IDENTITY,
    elo_rating_id BIGINT NOT NULL,
    attempt_id BIGINT NULL,
    rating_before DOUBLE PRECISION NOT NULL,
    rating_after DOUBLE PRECISION NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_elo_rating_history PRIMARY KEY (elo_rating_history_id),
    CONSTRAINT fk_elo_rating_history_elo_rating_id FOREIGN KEY (elo_rating_id) REFERENCES elo_rating (elo_rating_id) ON DELETE CASCADE,
    CONSTRAINT fk_elo_rating_history_attempt_id FOREIGN KEY (attempt_id) REFERENCES attempt (attempt_id) ON DELETE SET NULL
);

CREATE INDEX ix_elo_rating_history_elo_rating_id_changed_at ON elo_rating_history (elo_rating_id, changed_at);
CREATE INDEX ix_elo_rating_history_attempt_id ON elo_rating_history (attempt_id);

-- =====================================================================
-- 6. Avis & signalements
-- =====================================================================

-- Note (1 à 5 étoiles) d'un contenu : une seule par auteur et par contenu.
-- Cible : arc exclusif question / exercice. Les personnes ne sont pas notées.
CREATE TABLE rating
(
    rating_id BIGINT GENERATED ALWAYS AS IDENTITY,
    author_account_id BIGINT NOT NULL,
    question_id BIGINT NULL,
    exercise_id BIGINT NULL,
    star_count SMALLINT NOT NULL,
    comment TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_rating PRIMARY KEY (rating_id),
    CONSTRAINT fk_rating_author_account_id FOREIGN KEY (author_account_id) REFERENCES account (account_id),
    CONSTRAINT fk_rating_question_id FOREIGN KEY (question_id) REFERENCES question (question_id) ON DELETE CASCADE,
    CONSTRAINT fk_rating_exercise_id FOREIGN KEY (exercise_id) REFERENCES exercise (exercise_id) ON DELETE CASCADE,
    CONSTRAINT ck_rating_single_target CHECK
    (
        (CASE WHEN question_id IS NOT NULL THEN 1 ELSE 0 END)
        + (CASE WHEN exercise_id IS NOT NULL THEN 1 ELSE 0 END) = 1
    ),
    CONSTRAINT ck_rating_star_count CHECK (star_count BETWEEN 1 AND 5)
);

CREATE UNIQUE INDEX ux_rating_author_question ON rating (author_account_id, question_id) WHERE question_id IS NOT NULL;
CREATE UNIQUE INDEX ux_rating_author_exercise ON rating (author_account_id, exercise_id) WHERE exercise_id IS NOT NULL;
CREATE INDEX ix_rating_question_id ON rating (question_id);
CREATE INDEX ix_rating_exercise_id ON rating (exercise_id);

CREATE TRIGGER trg_rating_set_updated_at
    BEFORE UPDATE ON rating
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Retour écrit, traité par la modération.
-- feedback_type   : comment, suggestion, error_report, abuse_report
-- feedback_status : open, resolved, dismissed
-- Cible : arc exclusif question / exercice / groupe / compte (signalement).
CREATE TABLE feedback
(
    feedback_id BIGINT GENERATED ALWAYS AS IDENTITY,
    author_account_id BIGINT NOT NULL,
    feedback_type VARCHAR(20) NOT NULL,
    question_id BIGINT NULL,
    exercise_id BIGINT NULL,
    study_group_id BIGINT NULL,
    target_account_id BIGINT NULL,
    content TEXT NOT NULL,
    feedback_status VARCHAR(20) NOT NULL DEFAULT 'open',
    handled_by_account_id BIGINT NULL,
    handled_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_feedback PRIMARY KEY (feedback_id),
    CONSTRAINT fk_feedback_author_account_id FOREIGN KEY (author_account_id) REFERENCES account (account_id),
    CONSTRAINT fk_feedback_question_id FOREIGN KEY (question_id) REFERENCES question (question_id) ON DELETE CASCADE,
    CONSTRAINT fk_feedback_exercise_id FOREIGN KEY (exercise_id) REFERENCES exercise (exercise_id) ON DELETE CASCADE,
    CONSTRAINT fk_feedback_study_group_id FOREIGN KEY (study_group_id) REFERENCES study_group (study_group_id) ON DELETE CASCADE,
    CONSTRAINT fk_feedback_target_account_id FOREIGN KEY (target_account_id) REFERENCES account (account_id) ON DELETE CASCADE,
    CONSTRAINT fk_feedback_handled_by_account_id FOREIGN KEY (handled_by_account_id) REFERENCES account (account_id) ON DELETE SET NULL,
    CONSTRAINT ck_feedback_feedback_type CHECK (feedback_type IN ('comment', 'suggestion', 'error_report', 'abuse_report')),
    CONSTRAINT ck_feedback_feedback_status CHECK (feedback_status IN ('open', 'resolved', 'dismissed')),
    CONSTRAINT ck_feedback_single_target CHECK
    (
        (CASE WHEN question_id IS NOT NULL THEN 1 ELSE 0 END)
        + (CASE WHEN exercise_id IS NOT NULL THEN 1 ELSE 0 END)
        + (CASE WHEN study_group_id IS NOT NULL THEN 1 ELSE 0 END)
        + (CASE WHEN target_account_id IS NOT NULL THEN 1 ELSE 0 END) = 1
    ),
    CONSTRAINT ck_feedback_handled_at CHECK
    (
        (feedback_status = 'open' AND handled_at IS NULL)
        OR (feedback_status <> 'open' AND handled_at IS NOT NULL)
    )
);

CREATE INDEX ix_feedback_author_account_id ON feedback (author_account_id);
CREATE INDEX ix_feedback_question_id ON feedback (question_id);
CREATE INDEX ix_feedback_exercise_id ON feedback (exercise_id);
CREATE INDEX ix_feedback_study_group_id ON feedback (study_group_id);
CREATE INDEX ix_feedback_target_account_id ON feedback (target_account_id);
CREATE INDEX ix_feedback_handled_by_account_id ON feedback (handled_by_account_id);
-- File de modération : seulement les retours encore ouverts.
CREATE INDEX ix_feedback_open ON feedback (created_at) WHERE feedback_status = 'open';

CREATE TRIGGER trg_feedback_set_updated_at
    BEFORE UPDATE ON feedback
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- =====================================================================
-- Données de référence minimales
-- (la liste complète des pays relève d'un script de données à part)
-- =====================================================================
INSERT INTO country (country_code, name)
VALUES ('FR', 'France');
