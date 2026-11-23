-- =====================================================================
-- 0002 — Identifiant public UUID pour question
-- ---------------------------------------------------------------------
-- Décision (tâche 2, arbitrage « public_id » du TODO) : l'API n'expose
-- JAMAIS les BIGINT internes (séquentiels = énumérables, ils renseignent
-- le volume d'activité et facilitent le scraping).
-- 0001 est déjà appliquée : on ne la retouche pas, on ajoute 0002.
-- =====================================================================

SET search_path TO app, public;

ALTER TABLE question
    ADD COLUMN public_id UUID NOT NULL DEFAULT gen_random_uuid();

-- Les requêtes API arrivent par UUID : l'index unique sert ces recherches.
CREATE UNIQUE INDEX ux_question_public_id ON question (public_id);
