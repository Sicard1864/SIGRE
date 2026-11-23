-- Retire l'identifiant public (l'index part avec la colonne).
SET search_path TO app, public;

ALTER TABLE question DROP COLUMN IF EXISTS public_id;
