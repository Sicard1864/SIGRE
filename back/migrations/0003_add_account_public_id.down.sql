-- Retire l'identifiant public (l'index part avec la colonne).
SET search_path TO app, public;

ALTER TABLE account DROP COLUMN IF EXISTS public_id;
