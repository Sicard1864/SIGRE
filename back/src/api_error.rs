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
