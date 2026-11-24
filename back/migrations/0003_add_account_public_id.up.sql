-- =====================================================================
-- 0003 — Identifiant public UUID pour account
-- ---------------------------------------------------------------------
-- Même logique que 0002 (question) : l'API n'expose JAMAIS le BIGINT
-- interne (séquentiel = énumérable). Pour le compte, aller plus loin :
-- aucune route ne rend un compte consultable par identifiant externe
-- (la lecture passe par /accounts/me, déduit du jeton) ; public_id sert
-- de référence externe (JWT, liens depuis d'autres ressources).
-- =====================================================================

SET search_path TO app, public;

ALTER TABLE account
    ADD COLUMN public_id UUID NOT NULL DEFAULT gen_random_uuid();

-- Les jointures et recherches par identifiant externe passent par cet index.
CREATE UNIQUE INDEX ux_account_public_id ON account (public_id);
