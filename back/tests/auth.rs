// Tests d'intégration de l'authentification et de /accounts/me.
// Chaque test reçoit sa base éphémère vierge (#[sqlx::test] crée une base
// dédiée, joue ./migrations, et la détruit) et son propre rate limiter :
// les tests tournent en parallèle sans interférence.

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use chrono::{Duration, Utc};
use http_body_util::BodyExt;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tower::ServiceExt;

use sigre_back::{
    app_state::{AppState, JwtConfig},
    auth::Claims,
    build_router,
    rate_limit::RateLimiter,
};

const TEST_SECRET: &str = "test-secret-0123456789abcdef/0123";
const VALID_PASSWORD: &str = "correct-horse-battery";

fn test_router(pool: PgPool) -> Router {
    let state = AppState {
        pool,
        password_pepper: "test-pepper".to_owned(),
        jwt: JwtConfig {
            secret: TEST_SECRET.to_owned(),
            access_ttl_minutes: 15,
            refresh_ttl_days: 30,
        },
        rate_limiter: RateLimiter::new(),
    };
    build_router(state)
}

async fn request(
    router: &Router,
    method: Method,
    uri: &str,
    bearer: Option<&str>,
    body: Option<String>,
) -> axum::http::Response<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let body = match body {
        Some(json) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(json)
        }
        None => Body::empty(),
    };
    let request = builder.body(body).unwrap();
    router.clone().oneshot(request).await.unwrap()
}

async fn post_json(router: &Router, uri: &str, json_body: &str) -> axum::http::Response<Body> {
    request(router, Method::POST, uri, None, Some(json_body.to_owned())).await
}

async fn register(router: &Router, username: &str) -> String {
    let response = post_json(
        router,
        "/api/v1/accounts",
        &format!(
            r#"{{"username": "{username}", "email": "{username}@example.org", "password": "{VALID_PASSWORD}"}}"#
        ),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = body_json(response).await;
    body["id"].as_str().expect("id doit être une chaîne").to_owned()
}

async fn login(router: &Router, email: &str, password: &str) -> axum::http::Response<Body> {
    post_json(
        router,
        "/api/v1/auth/login",
        &format!(r#"{{"email": "{email}", "password": "{password}"}}"#),
    )
    .await
}

async fn refresh(router: &Router, refresh_token: &str) -> axum::http::Response<Body> {
    post_json(
        router,
        "/api/v1/auth/refresh",
        &format!(r#"{{"refresh_token": "{refresh_token}"}}"#),
    )
    .await
}

async fn me(router: &Router, access_token: &str) -> axum::http::Response<Body> {
    request(
        router,
        Method::GET,
        "/api/v1/accounts/me",
        Some(access_token),
        None,
    )
    .await
}

async fn body_json(response: axum::http::Response<Body>) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn sha256_hex(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

// ============================================================
// Login
// ============================================================

#[sqlx::test]
async fn login_with_valid_credentials_returns_tokens_and_account(pool: PgPool) {
    let router = test_router(pool.clone());
    register(&router, "alice").await;

    let response = login(&router, "alice@example.org", VALID_PASSWORD).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert!(!body["access_token"].as_str().unwrap().is_empty());
    assert!(!body["refresh_token"].as_str().unwrap().is_empty());
    assert_eq!(body["account"]["username"], "alice");

    // En base : le hash du refresh renvoyé, jamais le token brut ; et
    // last_login_at est posé.
    let refresh_token = body["refresh_token"].as_str().unwrap();
    let stored: Option<i64> = sqlx::query_scalar(
        "SELECT refresh_token_id FROM app.refresh_token WHERE token_hash = $1",
    )
    .bind(sha256_hex(refresh_token))
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(stored.is_some());

    let last_login_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT last_login_at FROM app.account WHERE username = 'alice'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(last_login_at.is_some());
}

#[sqlx::test]
async fn login_with_wrong_credentials_returns_indistinguishable_401(pool: PgPool) {
    let router = test_router(pool);
    register(&router, "alice").await;

    let wrong_password = login(&router, "alice@example.org", "wrong-password-here").await;
    let unknown_email = login(&router, "nobody@example.org", "whatever-password").await;

    assert_eq!(wrong_password.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(unknown_email.status(), StatusCode::UNAUTHORIZED);

    // Corps strictement identiques : ni le code, ni le detail, ni le timing
    // observable via la réponse ne dit quel champ est faux (STYLE_API.md § 8).
    let body_wrong = body_json(wrong_password).await;
    let body_unknown = body_json(unknown_email).await;
    assert_eq!(body_wrong, body_unknown);
    assert_eq!(body_wrong["code"], "auth.invalid_credentials");
}

#[sqlx::test]
async fn login_rate_limited_returns_429_with_retry_after(pool: PgPool) {
    let router = test_router(pool.clone());
    register(&router, "alice").await;

    for _ in 0..10 {
        let response = login(&router, "alice@example.org", "wrong-password-here").await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    let response = login(&router, "alice@example.org", "wrong-password-here").await;

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(response.headers().contains_key(header::RETRY_AFTER));
    let body = body_json(response).await;
    assert_eq!(body["code"], "rate_limit.exceeded");

    // Compteur isolé par route : refresh n'a jamais été appelé, il passait
    // encore si on l'avait tenté. Le compte lui-même est intact.
    let accounts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app.account")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(accounts, 1);
}

async fn login_tokens(router: &Router, email: &str, password: &str) -> (String, String) {
    let response = login(router, email, password).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    (
        body["access_token"].as_str().unwrap().to_owned(),
        body["refresh_token"].as_str().unwrap().to_owned(),
    )
}

// ============================================================
// Compte courant (extracteur CurrentUser)
// ============================================================

#[sqlx::test]
async fn me_with_valid_token_returns_account(pool: PgPool) {
    let router = test_router(pool);
    register(&router, "alice").await;
    let (access_token, _) = login_tokens(&router, "alice@example.org", VALID_PASSWORD).await;

    let response = me(&router, &access_token).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["username"], "alice");
    assert_eq!(body["email"], "alice@example.org");
}

#[sqlx::test]
async fn me_without_token_returns_401_token_missing(pool: PgPool) {
    let router = test_router(pool);

    let response = request(&router, Method::GET, "/api/v1/accounts/me", None, None).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = body_json(response).await;
    assert_eq!(body["code"], "auth.token_missing");
}

#[sqlx::test]
async fn me_with_garbage_token_returns_401_token_invalid(pool: PgPool) {
    let router = test_router(pool);

    let response = me(&router, "ce-nest-pas-un-jwt").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = body_json(response).await;
    assert_eq!(body["code"], "auth.token_invalid");
}

#[sqlx::test]
async fn me_with_expired_token_returns_401_token_expired(pool: PgPool) {
    let router = test_router(pool);
    let public_id = register(&router, "alice").await;

    // Jeton signé avec le secret du test, mais expiré depuis longtemps.
    let now = Utc::now();
    let claims = Claims {
        sub: public_id,
        iat: (now - Duration::hours(2)).timestamp() as usize,
        exp: (now - Duration::hours(1)).timestamp() as usize,
    };
    let expired = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(TEST_SECRET.as_bytes()),
    )
    .unwrap();

    let response = me(&router, &expired).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = body_json(response).await;
    assert_eq!(body["code"], "auth.token_expired");
}

// ============================================================
// Rotation des refresh tokens
// ============================================================

#[sqlx::test]
async fn refresh_rotates_tokens_and_reuse_revokes_the_family(pool: PgPool) {
    let router = test_router(pool.clone());
    register(&router, "alice").await;
    let (_, first_refresh) = login_tokens(&router, "alice@example.org", VALID_PASSWORD).await;

    // Première rotation : nouvelles cartes valides.
    let response = refresh(&router, &first_refresh).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    let second_refresh = body["refresh_token"].as_str().unwrap().to_owned();
    assert_ne!(second_refresh, first_refresh);

    // L'ancien refresh ne sert plus.
    let replay = refresh(&router, &first_refresh).await;
    assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);

    // Réutilisation d'un token révoqué = vol présumé : TOUTE la famille tombe,
    // même le token fraîchement émis.
    let family = refresh(&router, &second_refresh).await;
    assert_eq!(family.status(), StatusCode::UNAUTHORIZED);

    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM app.refresh_token WHERE revoked_at IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active, 0);
}

#[sqlx::test]
async fn refresh_with_unknown_token_returns_401(pool: PgPool) {
    let router = test_router(pool);

    let response = refresh(&router, "83ef7f8e-1111-4222-8333-444455556666").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = body_json(response).await;
    assert_eq!(body["code"], "auth.refresh_invalid");
}

#[sqlx::test]
async fn refresh_with_expired_token_returns_401(pool: PgPool) {
    let router = test_router(pool.clone());
    register(&router, "alice").await;
    let (_, refresh_token) = login_tokens(&router, "alice@example.org", VALID_PASSWORD).await;

    sqlx::query("UPDATE app.refresh_token SET expires_at = CURRENT_TIMESTAMP - interval '1 hour'")
        .execute(&pool)
        .await
        .unwrap();

    let response = refresh(&router, &refresh_token).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = body_json(response).await;
    assert_eq!(body["code"], "auth.refresh_invalid");
}

// ============================================================
// PATCH /accounts/me
// ============================================================

#[sqlx::test]
async fn patch_me_updates_bio_and_ignores_absent_fields(pool: PgPool) {
    let router = test_router(pool);
    register(&router, "alice").await;
    let (access_token, _) = login_tokens(&router, "alice@example.org", VALID_PASSWORD).await;

    let response = request(
        &router,
        Method::PATCH,
        "/api/v1/accounts/me",
        Some(&access_token),
        Some(r#"{"bio": "Apprendre c'est réviser intelligemment."}"#.to_owned()),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["bio"], "Apprendre c'est réviser intelligemment.");
    assert_eq!(body["username"], "alice");

    // Corps vide : sans effet, le compte revient inchangé (PATCH, § 4).
    let response = request(
        &router,
        Method::PATCH,
        "/api/v1/accounts/me",
        Some(&access_token),
        Some("{}".to_owned()),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["bio"], "Apprendre c'est réviser intelligemment.");
}

#[sqlx::test]
async fn patch_me_username_taken_returns_conflict(pool: PgPool) {
    let router = test_router(pool);
    register(&router, "alice").await;
    register(&router, "bob").await;
    let (bob_token, _) = login_tokens(&router, "bob@example.org", VALID_PASSWORD).await;

    let response = request(
        &router,
        Method::PATCH,
        "/api/v1/accounts/me",
        Some(&bob_token),
        Some(r#"{"username": "Alice"}"#.to_owned()),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = body_json(response).await;
    assert_eq!(body["code"], "account.username_taken");
}

#[sqlx::test]
async fn patch_me_without_token_returns_401(pool: PgPool) {
    let router = test_router(pool);

    let response = request(
        &router,
        Method::PATCH,
        "/api/v1/accounts/me",
        None,
        Some(r#"{"bio": "nope"}"#.to_owned()),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================
// DELETE /accounts/me : anonymisation RGPD
// ============================================================

#[sqlx::test]
async fn delete_me_anonymizes_account_and_revokes_sessions(pool: PgPool) {
    let router = test_router(pool.clone());
    let public_id = register(&router, "alice").await;
    let (access_token, refresh_token) =
        login_tokens(&router, "alice@example.org", VALID_PASSWORD).await;

    let response = request(
        &router,
        Method::DELETE,
        "/api/v1/accounts/me",
        Some(&access_token),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // En base : identité effacée, compte archivé, sessions révoquées.
    let (username, email, password_hash, archived_at): (
        String,
        Option<String>,
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as(
        "SELECT username, email, password_hash, archived_at \
         FROM app.account WHERE public_id = $1",
    )
    .bind(public_id.parse::<uuid::Uuid>().unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(username.starts_with("anonymous_"));
    assert!(email.is_none());
    assert!(password_hash.is_none());
    assert!(archived_at.is_some());

    let active_sessions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM app.refresh_token WHERE revoked_at IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_sessions, 0);

    // Le jeton d'accès encore valide côté durée ne sert plus : le compte est
    // archivé, l'extracteur refuse. Et le login devient impossible.
    let me_response = me(&router, &access_token).await;
    assert_eq!(me_response.status(), StatusCode::UNAUTHORIZED);

    let login_response = login(&router, "alice@example.org", VALID_PASSWORD).await;
    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);

    let refresh_response = refresh(&router, &refresh_token).await;
    assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);
}
