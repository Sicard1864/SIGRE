use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::problem::problem;

/// Représentation publique d'une question : l'UUID public remplace
/// le BIGINT interne, jamais exposé (cf. migration 0002).
#[derive(Serialize, FromRow)]
pub struct QuestionSummary {
    pub id: Uuid,
    pub question_type: String,
    pub publication_status: String,
    pub created_at: DateTime<Utc>,
}

/// GET /api/v1/questions : les questions les plus récentes.
/// Taille volontairement fixe tant que la pagination n'est pas spécifiée.
pub async fn list(State(pool): State<Arc<PgPool>>) -> Response {
    let rows = sqlx::query_as::<_, QuestionSummary>(
        "SELECT public_id AS id, question_type, publication_status, created_at \
         FROM app.question \
         ORDER BY created_at DESC \
         LIMIT 50",
    )
    .fetch_all(&*pool)
    .await;

    match rows {
        Ok(items) => Json(items).into_response(),
        Err(sqlx_error) => {
            // Le détail SQL reste côté serveur (log), le client reçoit un ProblemDetails générique.
            eprintln!("[questions::list] échec SQL : {sqlx_error:?}");
            problem(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        }
    }
}
