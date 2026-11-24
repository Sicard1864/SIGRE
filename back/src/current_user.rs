use axum::{extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::auth::Claims;

/// Compte courant déduit du jeton, jamais du corps (STYLE_API.md § 8).
/// Injecté dans les handlers de `/accounts/me` via `FromRequestParts` ;
/// porte l'account_id interne, les handlers refont leurs SQL dessus.
pub struct CurrentUser {
    pub account_id: i64,
}

fn token_missing() -> ApiError {
    ApiError::Unauthorized {
        code: "auth.token_missing",
        detail: "Authentification requise.",
    }
}

fn token_invalid() -> ApiError {
    ApiError::Unauthorized {
        code: "auth.token_invalid",
        detail: "Jeton invalide.",
    }
}

fn token_expired() -> ApiError {
    ApiError::Unauthorized {
        code: "auth.token_expired",
        detail: "Jeton expiré.",
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authorization = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(token_missing)?;

        let token = authorization
            .strip_prefix("Bearer ")
            .ok_or_else(token_invalid)?;

        let claims = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(state.jwt.secret.as_bytes()),
            &jsonwebtoken::Validation::default(), // HS256 + validation de exp
        )
        .map_err(|error| match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => token_expired(),
            _ => token_invalid(),
        })?;

        let public_id: Uuid = claims.claims.sub.parse().map_err(|_| token_invalid())?;

        let account_id: Option<i64> = sqlx::query_scalar(
            "SELECT account_id FROM app.account WHERE public_id = $1 AND archived_at IS NULL",
        )
        .bind(public_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|sqlx_error| {
            eprintln!("[current_user] échec SQL : {sqlx_error:?}");
            ApiError::Internal
        })?;

        // Compte anonymisé ou jeton référence un compte disparu : même 401
        // générique, on ne distingue rien.
        account_id
            .map(|account_id| Self { account_id })
            .ok_or_else(token_invalid)
    }
}
