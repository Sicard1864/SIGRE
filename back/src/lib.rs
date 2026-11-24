pub mod accounts;
pub mod api_error;
pub mod app_state;
pub mod json;
pub mod openapi;
pub mod problem;
pub mod questions;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;

use crate::app_state::AppState;

async fn health(State(state): State<AppState>) -> String {
    // Prouve que la connexion à PostgreSQL fonctionne, pas juste que le process vit
    let one: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .expect("requête SELECT 1 impossible");
    format!("OK db={one}")
}

/// Router complet, réutilisé par les tests d'intégration : une seule
/// définition des routes évite que le contrat diverge entre prod et tests.
pub fn build_router(state: AppState) -> Router {
    // CORS : seules les origines du front sont autorisées (liste explicite, pas de wildcard).
    let allowed_origins = ["http://localhost:5173", "http://localhost:8080"];
    let cors = CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .map(|origin| origin.parse().expect("origine CORS invalide"))
                .collect::<Vec<axum::http::HeaderValue>>(),
        )
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/questions", get(questions::list))
        .route("/api/v1/accounts", post(accounts::register))
        .route(
            "/api/v1/openapi.json",
            get(|| async { Json(openapi::ApiDoc::openapi()) }),
        )
        .layer(cors)
        .with_state(state);

    // UI Swagger : environnement de dev uniquement (STYLE_API.md § 12).
    // Chemin sans joker : la crate enregistre elle-même le redirect, la page
    // UI et son sous-chemin {*rest}. Config::from pointe l'UI vers la route
    // /api/v1/openapi.json déclarée ci-dessus, sans la redéclarer (deux
    // déclarations de la même route = panic au merge des routers).
    #[cfg(debug_assertions)]
    let app = app.merge(
        utoipa_swagger_ui::SwaggerUi::new("/api/v1/swagger-ui")
            .config(utoipa_swagger_ui::Config::from("/api/v1/openapi.json")),
    );

    app
}
