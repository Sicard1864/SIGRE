use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::problem::Problem;

/// Représentation publique d'une question : l'UUID public remplace
/// le BIGINT interne, jamais exposé (cf. migration 0002).
#[derive(Serialize, FromRow, utoipa::ToSchema)]
pub struct QuestionSummary {
    pub id: Uuid,
    pub question_type: String,
    pub publication_status: String,
    pub created_at: DateTime<Utc>,
}

/// GET /api/v1/questions : les questions les plus récentes.
/// Taille volontairement fixe tant que la pagination n'est pas spécifiée.
#[utoipa::path(
    get,
    path = "/api/v1/questions",
    operation_id = "list_questions",
    tag = "questions",
    responses(
        (status = 200, description = "Questions les plus récentes", body = [QuestionSummary]),
        (status = 500, description = "Erreur interne (détail en ProblemDetails)",
         body = Problem, content_type = "application/problem+json"),
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<QuestionSummary>>, ApiError> {
    let rows = sqlx::query_as::<_, QuestionSummary>(
        "SELECT public_id AS id, question_type, publication_status, created_at \
         FROM app.question \
         ORDER BY created_at DESC \
         LIMIT 50",
    )
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(items) => Ok(Json(items)),
        Err(sqlx_error) => {
            // Le détail SQL reste côté serveur (log), le client reçoit un
            // ProblemDetails générique.
            eprintln!("[questions::list] échec SQL : {sqlx_error:?}");
            Err(ApiError::Internal)
        }
    }
}
