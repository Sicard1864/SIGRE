use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::current_user::CurrentUser;
use crate::{
    json::JsonBody,
    problem::{FieldError, Problem},
    rate_limit,
};

// ============================================================
// Champs (constantes)
// ============================================================

// ANSSI / CNIL : au moins 12 caractères, aucune règle de composition
// (majuscule/chiffre imposés = mots de passe plus faibles, pas plus forts).
const PASSWORD_MIN_CHARS: usize = 12;
const PASSWORD_MAX_CHARS: usize = 128;

const USERNAME_MIN_CHARS: usize = 3;
const USERNAME_MAX_CHARS: usize = 50;

// Longueur maximale d'une adresse email : RFC 5321.
const EMAIL_MAX_CHARS: usize = 254;

// Bio libre mais bornée : un TEXT illimité est un risque (bittotherium de
// sculptures UTF-8 de 10 Mo par requête).
const BIO_MAX_CHARS: usize = 5000;

// ============================================================
// DTO
// ============================================================

/// Corps de `POST /api/v1/accounts` (inscription d'un compte standard).
/// `deny_unknown_fields` : un champ non prévu est un 400 (STYLE_API.md § 6),
/// pas un champ silencieusement ignoré.
#[derive(serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RegisterAccountRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Corps de `PATCH /api/v1/accounts/me` : champ absent = inchangé
/// (STYLE_API.md § 4). V1 : pas de distinction absent/null, on ne peut pas
/// vider la bio en PATCH (documenté, à arbitrer plus tard).
#[derive(serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateAccountRequest {
    pub username: Option<String>,
    pub bio: Option<String>,
}

/// Vue publique d'un compte : le BIGINT interne n'existe pas dans le contrat,
/// seul l'UUID `public_id` (migration 0003, même logique que 0002) sort.
/// `email`/`bio`/`email_verified_at` sont nullables : un compte anonymé RGPD
/// n'a plus d'email ni de mot de passe.
#[derive(Serialize, FromRow, utoipa::ToSchema)]
pub struct AccountResponse {
    pub id: Uuid,
    pub account_type: String,
    pub username: String,
    pub email: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================
// Méthodes privées (validations)
// ============================================================

fn validate_username(username: &str) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let length = username.chars().count();

    if length < USERNAME_MIN_CHARS {
        errors.push(FieldError {
            field: "username",
            code: "too_short",
        });
    }
    if length > USERNAME_MAX_CHARS {
        errors.push(FieldError {
            field: "username",
            code: "too_long",
        });
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
    {
        errors.push(FieldError {
            field: "username",
            code: "invalid_format",
        });
    }
    errors
}

fn validate_email(email: &str) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let length = email.chars().count();

    if length > EMAIL_MAX_CHARS {
        errors.push(FieldError {
            field: "email",
            code: "too_long",
        });
    }

    // Simplification assumée : pas de « quoted local part » ni d'adresse exotique.
    // La base garantit l'unicité ; ici on rejette le bruit de saisie.
    let is_well_formed = match email.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && !domain.is_empty()
                && !email.chars().any(char::is_whitespace)
                && domain.contains('.')
                && !domain.split('.').any(str::is_empty)
        }
        None => false,
    };

    if !is_well_formed {
        errors.push(FieldError {
            field: "email",
            code: "invalid_format",
        });
    }
    errors
}

fn validate_password(password: &str) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let length = password.chars().count();

    if length < PASSWORD_MIN_CHARS {
        errors.push(FieldError {
            field: "password",
            code: "too_short",
        });
    }
    if length > PASSWORD_MAX_CHARS {
        errors.push(FieldError {
            field: "password",
            code: "too_long",
        });
    }
    if password.chars().any(char::is_control) {
        errors.push(FieldError {
            field: "password",
            code: "invalid_format",
        });
    }
    errors
}

fn validate_bio(bio: &str) -> Vec<FieldError> {
    let mut errors = Vec::new();

    if bio.chars().count() > BIO_MAX_CHARS {
        errors.push(FieldError {
            field: "bio",
            code: "too_long",
        });
    }
    errors
}

fn validate(request: &RegisterAccountRequest) -> Vec<FieldError> {
    // Tous les champs sont évalués : le front reçoit la liste complète pour
    // afficher toutes les erreurs du formulaire en une fois.
    let mut errors = validate_username(&request.username);
    errors.extend(validate_email(&request.email));
    errors.extend(validate_password(&request.password));
    errors
}

/// Traduit un échec SQL d'insertion ou de mise à jour de `account` : l'index
/// d'unicité violé devient un 409 du contrat front ; tout le reste est une
/// erreur interne sans détail. On tente l'écriture et on traduit l'erreur
/// plutôt que SELECT-then-INSERT (fenêtre de course entre les deux requêtes).
fn account_sql_error(sqlx_error: sqlx::Error) -> ApiError {
    match &sqlx_error {
        sqlx::Error::Database(db_error) => match db_error.constraint() {
            Some("ux_account_username") => {
                return ApiError::Conflict {
                    code: "account.username_taken",
                    detail: "Ce nom d'utilisateur est déjà pris.",
                };
            }
            Some("ux_account_email") => {
                return ApiError::Conflict {
                    code: "account.email_taken",
                    detail: "Cette adresse email est déjà utilisée.",
                };
            }
            other => {
                eprintln!("[accounts] échec SQL (contrainte {other:?})");
            }
        },
        other => {
            eprintln!("[accounts] échec SQL : {other:?}");
        }
    }

    ApiError::Internal
}

/// Hache le mot de passe + poivre en Argon2id (format PHC : algorithme,
/// paramètres et sel inclus). Appelé dans `spawn_blocking` : ce calcul coûte
/// volontairement du CPU (défense anti-force-brute déclarée) et ne doit pas
/// geler le runtime async (STYLE_RUST.md § 11).
pub(crate) fn hash_password(
    password: &str,
    pepper: &str,
) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Argon2id v19, paramètres par défaut de la crate
    // Le poivre n'est jamais stocké : il vit dans l'environnement et est
    // concaténé au mot de passe avant hachage (CDC § 2.5). Le hash stocké en
    // base ne permet donc pas de retenter hors application, même fuite de la
    // table incluse.
    let peppered = format!("{password}{pepper}");
    let hash = argon2.hash_password(peppered.as_bytes(), &salt)?;
    Ok(hash.to_string()) // "$argon2id$v=19$m=19456,t=2,p=1$…"
}

// ============================================================
// Handlers
// ============================================================

/// POST /api/v1/accounts : inscription d'un compte standard.
#[utoipa::path(
    post,
    path = "/api/v1/accounts",
    operation_id = "register_account",
    tag = "accounts",
    request_body = RegisterAccountRequest,
    responses(
        (status = 201, description = "Compte créé", body = AccountResponse),
        (status = 400, description = "JSON illisible, type inattendu ou champ inconnu",
         body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Pseudonyme ou email déjà pris",
         body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Règles de validation en échec (liste des champs)",
         body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "Trop d'inscriptions (rate limit)",
         body = Problem, content_type = "application/problem+json"),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    JsonBody(request): JsonBody<RegisterAccountRequest>,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, String); 1],
        Json<AccountResponse>,
    ),
    ApiError,
> {
    state
        .rate_limiter
        .check(rate_limit::REGISTER)
        .map_err(|retry_after_secs| ApiError::RateLimited { retry_after_secs })?;

    let errors = validate(&request);
    if !errors.is_empty() {
        return Err(ApiError::Validation(errors));
    }

    let password = request.password.clone();
    let pepper = state.password_pepper.clone();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password, &pepper))
        .await
        .map_err(|join_error| {
            eprintln!("[accounts::register] tâche de hachage : {join_error}");
            ApiError::Internal
        })?
        .map_err(|argon2_error| {
            eprintln!("[accounts::register] échec Argon2 : {argon2_error}");
            ApiError::Internal
        })?;

    // L'unicité est garantie par la base (index LOWER) : l'index violé est
    // traduit en 409 par account_sql_error, sans SELECT préalable.
    let row = sqlx::query_as::<_, AccountResponse>(
        "INSERT INTO app.account (account_type, username, email, password_hash) \
         VALUES ('standard', $1, $2, $3) \
         RETURNING public_id AS id, account_type, username, email, bio, \
                    email_verified_at, created_at, updated_at",
    )
    .bind(&request.username)
    .bind(&request.email)
    .bind(password_hash)
    .fetch_one(&state.pool)
    .await;

    let account = match row {
        Ok(account) => account,
        Err(sqlx_error) => return Err(account_sql_error(sqlx_error)),
    };

    // Le compte créé se lit ensuite via /accounts/me : aucune route ne rend un
    // compte consultable par son identifiant (anti-énumération, tâche 2).
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, "/api/v1/accounts/me".to_owned())],
        Json(account),
    ))
}

/// GET /api/v1/accounts/me : le compte courant, déduit du jeton.
#[utoipa::path(
    get,
    path = "/api/v1/accounts/me",
    operation_id = "get_me",
    tag = "accounts",
    responses(
        (status = 200, description = "Compte courant", body = AccountResponse),
        (status = 401, description = "Jeton absent, invalide, expiré ou compte archivé",
         body = Problem, content_type = "application/problem+json"),
    )
)]
pub async fn me(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<AccountResponse>, ApiError> {
    // L'extracteur a déjà prouvé que le compte existe et est actif ; ce
    // fetch_optional ne peut être vide que si l'anonymisation a eu lieu
    // entre la validation du jeton et la requête SQL.
    let row = sqlx::query_as::<_, AccountResponse>(
        "SELECT public_id AS id, account_type, username, email, email_verified_at, \
                bio, created_at, updated_at \
         FROM app.account \
         WHERE account_id = $1 AND archived_at IS NULL",
    )
    .bind(current_user.account_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|sqlx_error| {
        eprintln!("[accounts::me] échec SQL : {sqlx_error:?}");
        ApiError::Internal
    })?;

    // Le compte a été archivé entre la validation du jeton et cette lecture :
    // même 401 générique.
    row.map(Json).ok_or(ApiError::Unauthorized {
        code: "auth.token_invalid",
        detail: "Jeton invalide.",
    })
}

/// PATCH /api/v1/accounts/me : modification partielle (champs fournis).
#[utoipa::path(
    patch,
    path = "/api/v1/accounts/me",
    operation_id = "update_me",
    tag = "accounts",
    request_body = UpdateAccountRequest,
    responses(
        (status = 200, description = "Compte mis à jour", body = AccountResponse),
        (status = 400, description = "JSON illisible, type inattendu ou champ inconnu",
         body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Jeton absent, invalide, expiré ou compte archivé",
         body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Pseudonyme déjà pris",
         body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Règles de validation en échec",
         body = Problem, content_type = "application/problem+json"),
    )
)]
pub async fn update_me(
    State(state): State<AppState>,
    current_user: CurrentUser,
    JsonBody(request): JsonBody<UpdateAccountRequest>,
) -> Result<Json<AccountResponse>, ApiError> {
    // Champ absent = inchangé : on ne valide que ce qui est fourni, mais on
    // évalue tous les champs fournis en une passe (même règle que l'inscription).
    let mut errors = Vec::new();
    if let Some(username) = &request.username {
        errors.extend(validate_username(username));
    }
    if let Some(bio) = &request.bio {
        errors.extend(validate_bio(bio));
    }
    if !errors.is_empty() {
        return Err(ApiError::Validation(errors));
    }

    let row = sqlx::query_as::<_, AccountResponse>(
        "UPDATE app.account \
         SET username = COALESCE($1, username), \
             bio = COALESCE($2, bio) \
         WHERE account_id = $3 AND archived_at IS NULL \
         RETURNING public_id AS id, account_type, username, email, email_verified_at, \
                    bio, created_at, updated_at",
    )
    .bind(&request.username)
    .bind(&request.bio)
    .bind(current_user.account_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(account_sql_error)?;

    row.map(Json).ok_or(ApiError::Unauthorized {
        code: "auth.token_invalid",
        detail: "Jeton invalide.",
    })
}

/// DELETE /api/v1/accounts/me : suppression RGPD = anonymisation
/// (domain-model § décisions) : le contenu publié SURVIT, l'identité PART.
/// Pseudo remplacé (unique, dérivé de public_id), email et mot de passe
/// effacés, identité vérifiée supprimée, sessions révoquées, archived_at posé.
#[utoipa::path(
    delete,
    path = "/api/v1/accounts/me",
    operation_id = "delete_me",
    tag = "accounts",
    responses(
        (status = 204, description = "Compte anonymisé"),
        (status = 401, description = "Jeton absent, invalide, expiré ou compte archivé",
         body = Problem, content_type = "application/problem+json"),
    )
)]
pub async fn delete_me(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<StatusCode, ApiError> {
    // Une transaction : l'anonymisation est atomique. Un compte à moitié
    // anonymisé serait une violation RGPD en soi.
    let mut transaction = state.pool.begin().await.map_err(|sqlx_error| {
        eprintln!("[accounts::delete_me] ouverture de transaction : {sqlx_error:?}");
        ApiError::Internal
    })?;

    run_anonymization(&mut transaction, current_user.account_id)
        .await
        .map_err(|sqlx_error| {
            eprintln!("[accounts::delete_me] échec SQL : {sqlx_error:?}");
            ApiError::Internal
        })?;

    transaction.commit().await.map_err(|sqlx_error| {
        eprintln!("[accounts::delete_me] commit : {sqlx_error:?}");
        ApiError::Internal
    })?;

    Ok(StatusCode::NO_CONTENT)
}

async fn run_anonymization(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM app.verified_identity WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut **transaction)
        .await?;

    // 'anonymous_' || public_id : unique par construction (public_id l'est),
    // 46 caractères < VARCHAR(50), et surtout pas d'information personnelle.
    sqlx::query(
        "UPDATE app.account \
         SET username = 'anonymous_' || public_id::text, \
             email = NULL, \
             password_hash = NULL, \
             email_verified_at = NULL, \
             archived_at = CURRENT_TIMESTAMP \
         WHERE account_id = $1 AND archived_at IS NULL",
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;

    sqlx::query(
        "UPDATE app.refresh_token \
         SET revoked_at = CURRENT_TIMESTAMP \
         WHERE account_id = $1 AND revoked_at IS NULL",
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
