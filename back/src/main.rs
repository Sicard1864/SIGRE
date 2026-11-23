use std::sync::Arc;

use axum::{Router, extract::State, routing::get};
use sqlx::{PgPool, postgres::PgPoolOptions};

async fn health(State(pool): State<Arc<PgPool>>) -> String {
    // Prouve que la connexion à PostgreSQL fonctionne, pas juste que le process vit
    let one: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&*pool)
        .await
        .expect("requête SELECT 1 impossible");
    format!("OK db={one}")
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL manquante (définis-la dans back/.env)");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("connexion à PostgreSQL impossible");

    let app = Router::new()
        .route("/health", get(health))
        .with_state(Arc::new(pool));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("port 3000 indisponible");
    println!("SIGRE back à l'écoute sur http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
