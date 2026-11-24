-- Retire les refresh tokens (la table part entière).
SET search_path TO app, public;

DROP TABLE IF EXISTS refresh_token;
