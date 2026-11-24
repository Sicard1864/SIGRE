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
use crate::json::JsonBody;
use crate::problem::{FieldError, Problem};

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

fn validate(request: &RegisterAccountRequest) -> Vec<FieldError> {
    // Tous les champs sont évalués : le front reçoit la liste complète pour
    // afficher toutes les erreurs du formulaire en une fois.
    let mut errors = validate_username(&request.username);
    errors.extend(validate_email(&request.email));
    errors.extend(validate_password(&request.password));
    errors
}

/// Traduit un échec SQL d'insertion : l'index d'unicité violé devient un 409
/// du contrat front ; tout le reste est une erreur interne sans détail.
/// On tente l'insertion et on traduit l'erreur plutôt que de
/// SELECT-then-INSERT (fenêtre de course entre les deux requêtes).
fn register_sql_error(sqlx_error: sqlx::Error) -> ApiError {
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
                eprintln!("[accounts::register] échec SQL (contrainte {other:?})");
            }
        },
        other => {
            eprintln!("[accounts::register] échec SQL : {other:?}");
        }
    }

    ApiError::Internal
}

/// Hache le mot de passe + poivre en Argon2id (format PHC : algorithme,
/// paramètres et sel inclus). Appelé dans `spawn_blocking` : ce calcul coûte
/// volontairement du CPU (défense anti-force-brute déclarée) et ne doit pas
/// geler le runtime async (STYLE_RUST.md § 11).
fn hash_password(password: &str, pepper: &str) -> Result<String, argon2::password_hash::Error> {
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
// Handler
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
    // traduit en 409 par register_sql_error, sans SELECT préalable.
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
        Err(sqlx_error) => return Err(register_sql_error(sqlx_error)),
    };

    // Le compte créé se lit ensuite via /accounts/me : aucune route ne rend un
    // compte consultable par son identifiant (anti-énumération, task 2).
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, "/api/v1/accounts/me".to_owned())],
        Json(account),
    ))
}
