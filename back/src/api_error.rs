use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::problem::{FieldError, problem};

/// Traduction métier → HTTP, centralisée (STYLE_RUST.md § 14) : le handler
/// choisit l'erreur sémantique, jamais le code de statut ni le corps.
pub enum ApiError {
    /// JSON illisible, type inattendu, champ inconnu (STYLE_API.md § 6).
    BadBody(String),
    /// JSON valide mais règles de validation en échec : 422 + `errors`.
    Validation(Vec<FieldError>),
    /// Conflit d'état (unicité…), avec le code stable du contrat front.
    Conflict {
        code: &'static str,
        detail: &'static str,
    },
    /// Pas ou plus authentifié : le code distingue la cause pour le front
    /// (auth.token_missing / auth.token_invalid / auth.token_expired /
    /// auth.invalid_credentials), le corps ne révèle rien de plus.
    Unauthorized {
        code: &'static str,
        detail: &'static str,
    },
    /// Rate limiting atteint (STYLE_API.md § 8) : 429 + Retry-After.
    RateLimited { retry_after_secs: u32 },
    /// Bug ou panne côté serveur : rien ne fuit, tout va dans les logs.
    Internal,
}

impl From<axum::extract::rejection::JsonRejection> for ApiError {
    fn from(rejection: axum::extract::rejection::JsonRejection) -> Self {
        // Le message de rejet peut refléter l'input du client : il reste côté serveur.
        eprintln!("[json] corps de requête rejeté : {rejection}");
        ApiError::BadBody("Corps de requête JSON invalide.".to_owned())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadBody(detail) => problem(
                StatusCode::BAD_REQUEST,
                "Requête invalide",
                Some(detail),
                Some("request.body_invalid".to_owned()),
                None,
            ),
            ApiError::Validation(errors) => problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation échouée",
                Some("Certains champs sont invalides.".to_owned()),
                Some("validation.failed".to_owned()),
                Some(errors),
            ),
            ApiError::Conflict { code, detail } => problem(
                StatusCode::CONFLICT,
                "Conflit",
                Some(detail.to_owned()),
                Some(code.to_owned()),
                None,
            ),
            ApiError::Unauthorized { code, detail } => {
                // RFC 6750 : tout 401 porte l'en-tête WWW-Authenticate: Bearer.
                let mut response = problem(
                    StatusCode::UNAUTHORIZED,
                    "Non authentifié",
                    Some(detail.to_owned()),
                    Some(code.to_owned()),
                    None,
                );
                response.headers_mut().insert(
                    axum::http::header::WWW_AUTHENTICATE,
                    axum::http::HeaderValue::from_static("Bearer"),
                );
                response
            }
            ApiError::RateLimited { retry_after_secs } => {
                let mut response = problem(
                    StatusCode::TOO_MANY_REQUESTS,
                    "Trop de requêtes",
                    Some("Trop de requêtes, réessaie plus tard.".to_owned()),
                    Some("rate_limit.exceeded".to_owned()),
                    None,
                );
                if let Ok(value) = axum::http::HeaderValue::from_str(&retry_after_secs.to_string()) {
                    response
                        .headers_mut()
                        .insert(axum::http::header::RETRY_AFTER, value);
                }
                response
            }
            ApiError::Internal => problem(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erreur interne",
                None,
                None,
                None,
            ),
        }
    }
}
