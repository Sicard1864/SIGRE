use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;

use crate::api_error::ApiError;

/// Extracteur JSON qui rejette en ProblemDetails (RFC 9457) au lieu du
/// 400 `text/plain` par défaut d'Axum : STYLE_API.md § 7 impose une seule
/// forme d'erreur pour toute l'API, y compris les JSON mal formés.
pub struct JsonBody<T>(pub T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(ApiError::from)?;
        Ok(JsonBody(value))
    }
}
