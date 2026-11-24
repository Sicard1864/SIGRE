-- =====================================================================
-- 0004 — Refresh tokens rotatifs (tâche 4 : authentification)
-- ---------------------------------------------------------------------
-- Le refresh token est un credential « bearer » : on ne stocke JAMAIS le
-- token brut en base, seulement son SHA-256 (token_hash). Une fuite de la
-- base ne donne donc aucun refresh token utilisable.
-- Rotation (CDC § 2.5) : à chaque usage, l'ancien token est révoqué et
-- pointe vers son remplaçant (replaced_by_refresh_token_id) ; un token
-- révoqué réutilisé = vol probable, on révoque toute la famille côté API.
-- =====================================================================

SET search_path TO app, public;

CREATE TABLE refresh_token
(
    refresh_token_id BIGINT GENERATED ALWAYS AS IDENTITY,
    token_hash CHAR(64) NOT NULL,
    account_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NULL,
    replaced_by_refresh_token_id BIGINT NULL,
    CONSTRAINT pk_refresh_token PRIMARY KEY (refresh_token_id),
    CONSTRAINT fk_refresh_token_account_id
        FOREIGN KEY (account_id) REFERENCES app.account (account_id) ON DELETE CASCADE,
    CONSTRAINT fk_refresh_token_replaced_by
        FOREIGN KEY (replaced_by_refresh_token_id) REFERENCES app.refresh_token (refresh_token_id),
    -- Un token remplacé est révoqué : la chaîne de rotation est auditable.
    CONSTRAINT ck_refresh_token_replacement
        CHECK (replaced_by_refresh_token_id IS NULL OR revoked_at IS NOT NULL)
);

-- La recherche se fait toujours par hash (jamais par le token brut).
CREATE UNIQUE INDEX ux_refresh_token_hash ON refresh_token (token_hash);

-- La révocation d'une famille (détection de vol) scanne les tokens du compte.
CREATE INDEX ix_refresh_token_account_id ON refresh_token (account_id);
