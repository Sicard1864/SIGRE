// Tests d'intégration de POST /api/v1/accounts.
// Chaque test reçoit sa base éphémère vierge (#[sqlx::test] crée une base
// dédiée, joue ./migrations, et la détruit) : les tests tournent en
// parallèle sans interférence.

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

use sigre_back::{app_state::AppState, build_router};

fn test_router(pool: PgPool) -> Router {
    let state = AppState {
        pool,
        password_pepper: "test-pepper".to_owned(),
    };
    build_router(state)
}

async fn post_accounts(router: &Router, json_body: &str) -> axum::http::Response<Body> {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/accounts")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json_body.to_owned()))
        .unwrap();
    router.clone().oneshot(request).await.unwrap()
}

async fn body_json(response: axum::http::Response<Body>) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn standard_body(username: &str, email: &str, password: &str) -> String {
    format!(r#"{{"username": "{username}", "email": "{email}", "password": "{password}"}}"#)
}

fn has_field_error(body: &Value, field: &str, code: &str) -> bool {
    body["errors"]
        .as_array()
        .expect("errors doit être un tableau en 422")
        .iter()
        .any(|error| error["field"] == field && error["code"] == code)
}

const VALID_PASSWORD: &str = "correct-horse-battery";

#[sqlx::test]
async fn register_valid_input_creates_account(pool: PgPool) {
    let router = test_router(pool.clone());

    let response = post_accounts(
        &router,
        &standard_body("alice", "alice@example.org", VALID_PASSWORD),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        response
            .headers()
            .get(header::LOCATION)
            .expect("Location obligatoire sur un 201")
            .to_str()
            .unwrap(),
        "/api/v1/accounts/me"
    );

    let body = body_json(response).await;
    assert_eq!(body["account_type"], "standard");
    assert_eq!(body["username"], "alice");
    assert_eq!(body["email"], "alice@example.org");
    assert!(body["email_verified_at"].is_null());

    let id = body["id"].as_str().expect("id doit être une chaîne");
    let public_id: uuid::Uuid = id.parse().expect("id doit être un UUID valide");

    // Le compte persiste, avec un hash Argon2id au format PHC, jamais le mot de passe.
    let (saved_hash, saved_public_id): (String, uuid::Uuid) =
        sqlx::query_as("SELECT password_hash, public_id FROM app.account WHERE username = 'alice'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(saved_hash.starts_with("$argon2id$"));
    assert_eq!(saved_public_id, public_id);
}

#[sqlx::test]
async fn register_duplicate_username_wrong_case_returns_conflict(pool: PgPool) {
    let router = test_router(pool);

    let first = post_accounts(
        &router,
        &standard_body("alice", "alice@example.org", VALID_PASSWORD),
    )
    .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let response = post_accounts(
        &router,
        &standard_body("Alice", "other@example.org", VALID_PASSWORD),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = body_json(response).await;
    assert_eq!(body["code"], "account.username_taken");
}

#[sqlx::test]
async fn register_duplicate_email_wrong_case_returns_conflict(pool: PgPool) {
    let router = test_router(pool);

    let first = post_accounts(
        &router,
        &standard_body("alice", "alice@example.org", VALID_PASSWORD),
    )
    .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let response = post_accounts(
        &router,
        &standard_body("bob", "Alice@Example.org", VALID_PASSWORD),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = body_json(response).await;
    assert_eq!(body["code"], "account.email_taken");
}

#[sqlx::test]
async fn register_short_password_returns_validation_error(pool: PgPool) {
    let router = test_router(pool);

    let response = post_accounts(
        &router,
        &standard_body("alice", "alice@example.org", "short_pass"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_json(response).await;
    assert_eq!(body["code"], "validation.failed");
    assert!(has_field_error(&body, "password", "too_short"));
}

#[sqlx::test]
async fn register_invalid_username_returns_validation_error(pool: PgPool) {
    let router = test_router(pool);

    let response = post_accounts(
        &router,
        &standard_body("alice!", "alice@example.org", VALID_PASSWORD),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_json(response).await;
    assert_eq!(body["code"], "validation.failed");
    assert!(has_field_error(&body, "username", "invalid_format"));
}

#[sqlx::test]
async fn register_invalid_email_returns_validation_error(pool: PgPool) {
    let router = test_router(pool);

    let response = post_accounts(
        &router,
        &standard_body("alice", "not-an-email", VALID_PASSWORD),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_json(response).await;
    assert_eq!(body["code"], "validation.failed");
    assert!(has_field_error(&body, "email", "invalid_format"));
}

#[sqlx::test]
async fn register_collects_all_field_errors_at_once(pool: PgPool) {
    let router = test_router(pool);

    // Les trois champs sont invalides en même temps : le front doit pouvoir
    // afficher les trois messages du formulaire en une seule requête.
    let response = post_accounts(&router, &standard_body("ab", "not-an-email", "short")).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_json(response).await;
    assert!(has_field_error(&body, "username", "too_short"));
    assert!(has_field_error(&body, "email", "invalid_format"));
    assert!(has_field_error(&body, "password", "too_short"));
}

#[sqlx::test]
async fn register_unknown_json_field_returns_bad_request(pool: PgPool) {
    let router = test_router(pool);

    let response = post_accounts(
        &router,
        r#"{"username": "alice", "email": "alice@example.org", "password": "correct-horse-battery", "is_admin": true}"#,
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.starts_with("application/problem+json"));
    let body = body_json(response).await;
    assert_eq!(body["code"], "request.body_invalid");
}

#[sqlx::test]
async fn register_malformed_json_returns_bad_request(pool: PgPool) {
    let router = test_router(pool);

    let response = post_accounts(&router, r#"{"username": "alice""#).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert_eq!(body["code"], "request.body_invalid");
}
