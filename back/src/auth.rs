use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use axum::{Json, extract::State};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use uuid::Uuid;

use crate::accounts::{AccountResponse, hash_password};
use crate::api_error::ApiError;
use crate::app_state::{AppState, JwtConfig};
use crate::json::JsonBody;
use crate::problem::Problem;
use crate::rate_limit;

// ============================================================
// DTO
// ============================================================

/// Corps de `POST /api/v1/auth/login`.
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Corps de `POST /api/v1/auth/refresh`.
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Paire de jetons. Le refresh est opaque (UUID v4) et rotatif : le jeton
/// présenté à /auth/refresh est révoqué à chaque usage, remplacé par un neuf.
/// V1 choisie « au plus simple » : les jetons voyagent dans le corps JSON,
/// pas encore en cookie HttpOnly (à revisiter avec le front réel).
#[derive(Serialize, utoipa::ToSchema)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

/// Réponse de login : les jetons + le compte (le front affiche le pseudo sans
/// requête supplémentaire).
#[derive(Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub account: AccountResponse,
}

/// Claims du JWT d'accès (HS256). `sub` porte le public_id UUID : le BIGINT
/// interne ne sort jamais du serveur, même dans un jeton signé.
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

/// Compte tel que stocké (avec le hash) : interne, jamais sérialisé tel quel.
#[derive(FromRow)]
struct StoredAccount {
    account_id: i64,
    public_id: Uuid,
    account_type: String,
    username: String,
    email: Option<String>,
    email_verified_at: Option<DateTime<Utc>>,
    bio: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    password_hash: String,
}

impl From<StoredAccount> for AccountResponse {
    fn from(account: StoredAccount) -> Self {
        Self {
            id: account.public_id,
            account_type: account.account_type,
            username: account.username,
            email: account.email,
            email_verified_at: account.email_verified_at,
            bio: account.bio,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

/// Ligne de refresh token résolue par hash.
#[derive(FromRow)]
struct RefreshRow {
    refresh_token_id: i64,
    account_id: i64,
    public_id: Uuid,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

// ============================================================
// Méthodes privées (jetons)
// ============================================================

/// 401 générique de login : identique que le compte existe ou non, quel champ
/// est faux. La réponse ne doit rien révéler (STYLE_API.md § 8).
fn invalid_credentials() -> ApiError {
    ApiError::Unauthorized {
        code: "auth.invalid_credentials",
        detail: "Identifiants invalides.",
    }
}

/// 401 générique de refresh (token inconnu, expiré ou déjà révoqué).
fn refresh_invalid() -> ApiError {
    ApiError::Unauthorized {
        code: "auth.refresh_invalid",
        detail: "Refresh token invalide ou expiré.",
    }
}

fn internal(context: &'static str, sqlx_error: sqlx::Error) -> ApiError {
    eprintln!("[{context}] échec SQL : {sqlx_error:?}");
    ApiError::Internal
}

/// SHA-256 hex : ce qui part (et ce qui est cherché) en base. Le token brut
/// ne quitte jamais le serveur/le front vers la base.
fn sha256_hex(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Vérifie `password || poivre` contre le hash PHC stocké, en thread dédié :
/// la vérification Argon2 coûte du CPU (c'est le but), elle ne doit pas
/// geler le runtime async (STYLE_RUST.md § 11).
async fn verify_password(
    password: String,
    password_hash: String,
    pepper: String,
) -> Result<bool, ApiError> {
    tokio::task::spawn_blocking(move || {
        // Un hash illisible en base est une erreur de données, pas un crash :
        // le compte ne peut pas se connecter, l'API reste debout.
        let Ok(parsed) = PasswordHash::new(&password_hash) else {
            return Ok(false);
        };
        let peppered = format!("{password}{pepper}");
        Ok(Argon2::default()
            .verify_password(peppered.as_bytes(), &parsed)
            .is_ok())
    })
    .await
    .map_err(|join_error| {
        eprintln!("[auth::login] tâche de vérification : {join_error}");
        ApiError::Internal
    })?
}

/// Signe le JWT d'accès du compte (public_id en sub, exp = now + TTL config).
fn issue_access_token(
    jwt: &JwtConfig,
    account_public_id: Uuid,
    now: DateTime<Utc>,
) -> Result<String, ApiError> {
    let claims = Claims {
        sub: account_public_id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::minutes(jwt.access_ttl_minutes)).timestamp() as usize,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(), // HS256
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt.secret.as_bytes()),
    )
    .map_err(|error| {
        eprintln!("[auth] signature JWT impossible : {error}");
        ApiError::Internal
    })
}

/// Login : crée la paire de jetons et persiste le refresh (hashé).
async fn issue_token_pair(
    state: &AppState,
    account_id: i64,
    account_public_id: Uuid,
) -> Result<TokenPair, ApiError> {
    let now = Utc::now();
    let access_token = issue_access_token(&state.jwt, account_public_id, now)?;

    // UUID v4 : 122 bits aléatoires, réputé infaisable à deviner. Suffisant
    // en V1 ; passer à 32 octets bruts en base64url si on veut de la marge.
    let refresh_token = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO app.refresh_token (token_hash, account_id, expires_at) \
         VALUES ($1, $2, $3)",
    )
    .bind(sha256_hex(&refresh_token))
    .bind(account_id)
    .bind(now + Duration::days(state.jwt.refresh_ttl_days))
    .execute(&state.pool)
    .await
    .map_err(|sqlx_error| internal("auth::login", sqlx_error))?;

    Ok(TokenPair {
        access_token,
        refresh_token,
    })
}

// ============================================================
// Handlers
// ============================================================

/// POST /api/v1/auth/login : échange identifiants → paire de jetons + compte.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    operation_id = "login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Identifiants valides", body = LoginResponse),
        (status = 400, description = "JSON illisible ou champ inconnu",
         body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Identifiants invalides (jamais lequel des deux)",
         body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "Trop de tentatives (rate limit)",
         body = Problem, content_type = "application/problem+json"),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    JsonBody(request): JsonBody<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    state
        .rate_limiter
        .check(rate_limit::LOGIN)
        .map_err(|retry_after_secs| ApiError::RateLimited { retry_after_secs })?;

    let account = sqlx::query_as::<_, StoredAccount>(
        "SELECT account_id, public_id, account_type, username, email, email_verified_at, \
                bio, created_at, updated_at, password_hash \
         FROM app.account \
         WHERE LOWER(email) = LOWER($1) AND archived_at IS NULL AND account_type = 'standard'",
    )
    .bind(&request.email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|sqlx_error| internal("auth::login", sqlx_error))?;

    let verified = match &account {
        Some(stored) => {
            verify_password(
                request.password.clone(),
                stored.password_hash.clone(),
                state.password_pepper.clone(),
            )
            .await?
        }
        None => {
            // Égalisation du temps de réponse : un email inconnu paie le même
            // coût Argon2 qu'un email existant, sinon la latence révèle quels
            // emails sont inscrits (énumération, STYLE_API.md § 8).
            let password = request.password.clone();
            let pepper = state.password_pepper.clone();
            let _ = tokio::task::spawn_blocking(move || hash_password(&password, &pepper)).await;
            false
        }
    };

    let Some(account) = account else {
        return Err(invalid_credentials());
    };
    if !verified {
        return Err(invalid_credentials());
    }

    sqlx::query("UPDATE app.account SET last_login_at = CURRENT_TIMESTAMP WHERE account_id = $1")
        .bind(account.account_id)
        .execute(&state.pool)
        .await
        .map_err(|sqlx_error| internal("auth::login", sqlx_error))?;

    let TokenPair {
        access_token,
        refresh_token,
    } = issue_token_pair(&state, account.account_id, account.public_id).await?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        account: account.into(),
    }))
}

/// POST /api/v1/auth/refresh : rotation. Le token présenté est révoqué
/// (replaced_by → le nouveau), un jeton révoqué réutilisé fait révoquer
/// toute la famille (détection de vol).
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    operation_id = "refresh_tokens",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Nouvelle paire de jetons (l'ancien refresh est révoqué)",
         body = TokenPair),
        (status = 401, description = "Refresh token inconnu, expiré ou réutilisé",
         body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "Trop de rafraîchissements (rate limit)",
         body = Problem, content_type = "application/problem+json"),
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    JsonBody(request): JsonBody<RefreshRequest>,
) -> Result<Json<TokenPair>, ApiError> {
    state
        .rate_limiter
        .check(rate_limit::REFRESH)
        .map_err(|retry_after_secs| ApiError::RateLimited { retry_after_secs })?;

    let row = sqlx::query_as::<_, RefreshRow>(
        "SELECT rt.refresh_token_id, rt.account_id, rt.expires_at, rt.revoked_at, \
                a.public_id \
         FROM app.refresh_token rt \
         JOIN app.account a ON a.account_id = rt.account_id \
         WHERE rt.token_hash = $1 AND a.archived_at IS NULL",
    )
    .bind(sha256_hex(&request.refresh_token))
    .fetch_optional(&state.pool)
    .await
    .map_err(|sqlx_error| internal("auth::refresh", sqlx_error))?;

    let Some(row) = row else {
        return Err(refresh_invalid());
    };

    if let Some(revoked_at) = row.revoked_at {
        // Réutilisation d'un token déjà révoqué : vol probable (le client
        // légitime a déjà tourné). On révoque la famille entière du compte :
        // le voleur ET le légitime doivent repasser par le login.
        eprintln!(
            "[auth::refresh] réutilisation suspectée (token {}, compte {}, révoqué {revoked_at}) \
             — révocation de la famille",
            row.refresh_token_id, row.account_id
        );
        sqlx::query(
            "UPDATE app.refresh_token \
             SET revoked_at = CURRENT_TIMESTAMP \
             WHERE account_id = $1 AND revoked_at IS NULL",
        )
        .bind(row.account_id)
        .execute(&state.pool)
        .await
        .map_err(|sqlx_error| internal("auth::refresh", sqlx_error))?;
        return Err(refresh_invalid());
    }

    if row.expires_at <= Utc::now() {
        return Err(refresh_invalid());
    }

    // Rotation transactionnelle : le nouveau token est inséré, l'ancien est
    // révoqué en pointant vers son remplaçant. La garde AND revoked_at IS NULL
    // fait échouer une rotation concurrente du même token.
    let mut transaction = state
        .pool
        .begin()
        .await
        .map_err(|sqlx_error| internal("auth::refresh", sqlx_error))?;

    let new_refresh_token = Uuid::new_v4().to_string();
    let new_id: i64 = sqlx::query_scalar(
        "INSERT INTO app.refresh_token (token_hash, account_id, expires_at) \
         VALUES ($1, $2, $3) \
         RETURNING refresh_token_id",
    )
    .bind(sha256_hex(&new_refresh_token))
    .bind(row.account_id)
    .bind(Utc::now() + Duration::days(state.jwt.refresh_ttl_days))
    .fetch_one(&mut *transaction)
    .await
    .map_err(|sqlx_error| internal("auth::refresh", sqlx_error))?;

    let revoked = sqlx::query(
        "UPDATE app.refresh_token \
         SET revoked_at = CURRENT_TIMESTAMP, replaced_by_refresh_token_id = $2 \
         WHERE refresh_token_id = $1 AND revoked_at IS NULL",
    )
    .bind(row.refresh_token_id)
    .bind(new_id)
    .execute(&mut *transaction)
    .await
    .map_err(|sqlx_error| internal("auth::refresh", sqlx_error))?;

    if revoked.rows_affected() != 1 {
        // Rotation concurrente du même token : on annule tout, l'autre a gagné.
        transaction
            .rollback()
            .await
            .map_err(|sqlx_error| internal("auth::refresh", sqlx_error))?;
        return Err(refresh_invalid());
    }

    transaction
        .commit()
        .await
        .map_err(|sqlx_error| internal("auth::refresh", sqlx_error))?;

    let access_token = issue_access_token(&state.jwt, row.public_id, Utc::now())?;

    Ok(Json(TokenPair {
        access_token,
        refresh_token: new_refresh_token,
    }))
}
