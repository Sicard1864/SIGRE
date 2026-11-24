use sqlx::{PgPool, postgres::PgPoolOptions};

use sigre_back::{app_state::AppState, build_router};

/// Construction de l'état applicatif : la configuration est validée au
/// démarrage (expect avec message de correction), jamais à la première
/// requête d'un utilisateur.
fn build_state_from_env(pool: PgPool) -> AppState {
    let password_pepper = std::env::var("PASSWORD_PEPPER")
        .expect("PASSWORD_PEPPER manquante (définis-la dans back/.env, cf. .env.example)");
    AppState {
        pool,
        password_pepper,
    }
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

    // Applique les migrations au démarrage (idempotent : sans effet si déjà jouées)
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("application des migrations impossible");

    let state = build_state_from_env(pool);
    let app = build_router(state);

    // 0.0.0.0 : écoute sur toutes les interfaces. Obligatoire en conteneur :
    // 127.0.0.1 n'écouterait que la boucle locale DU conteneur, invisible de l'extérieur.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("port 3000 indisponible");
    println!("SIGRE back à l'écoute sur http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
