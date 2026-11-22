-- Migration descendante : démontage dans l'ordre inverse du .up.sql
SET search_path TO app, public;

-- 6. Avis & signalements
DROP TABLE IF EXISTS feedback;
DROP TABLE IF EXISTS rating;

-- 5. Apprentissage adaptatif (SRS & Elo)
DROP TABLE IF EXISTS elo_rating_history;
DROP TABLE IF EXISTS elo_rating;
DROP TABLE IF EXISTS srs_state;

-- 4. Devoirs & pratique
DROP TABLE IF EXISTS attempt;
DROP TABLE IF EXISTS practice_session;
DROP TABLE IF EXISTS assignment;

-- 3. Tags
DROP TABLE IF EXISTS study_group_tag;
DROP TABLE IF EXISTS exercise_tag;
DROP TABLE IF EXISTS question_tag;
DROP TABLE IF EXISTS account_tag;
DROP TABLE IF EXISTS tag;

-- 2. Contenu pédagogique
DROP TABLE IF EXISTS answer;
DROP TABLE IF EXISTS exercise_question;
DROP TABLE IF EXISTS question;
DROP TABLE IF EXISTS exercise;

-- 1. Comptes & social
DROP TABLE IF EXISTS study_group_member;
DROP TABLE IF EXISTS study_group;
DROP TABLE IF EXISTS account_block;
DROP TABLE IF EXISTS friendship;
DROP TABLE IF EXISTS verified_identity;
DROP TABLE IF EXISTS account;

-- 0. Référentiels
DROP TABLE IF EXISTS country;

-- Plomberie
DROP FUNCTION IF EXISTS app.set_updated_at;
DROP SCHEMA IF EXISTS app;