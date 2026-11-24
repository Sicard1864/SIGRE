use sqlx::PgPool;

use crate::rate_limit::RateLimiter;

/// Configuration JWT, validée une seule fois au démarrage (STYLE_RUST.md § 13) :
/// le secret vit en variable d'environ, jamais en base ni dans le dépôt.
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_ttl_minutes: i64,
    pub refresh_ttl_days: i64,
}

impl JwtConfig {
    /// JWT_SECRET est obligatoire et doit faire au moins 32 octets :
    /// un secret HS256 court se brute-force, c'est la porte du back.
    /// Les durées ont des défauts raisonnables (15 min / 30 jours).
    pub fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET manquante (définis-la dans back/.env, cf. .env.example)");
        assert!(
            secret.len() >= 32,
            "JWT_SECRET trop courte : au moins 32 octets requis"
        );

        let access_ttl_minutes = std::env::var("JWT_ACCESS_TTL_MINUTES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(15);
        let refresh_ttl_days = std::env::var("JWT_REFRESH_TTL_DAYS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(30);

        Self {
            secret,
            access_ttl_minutes,
            refresh_ttl_days,
        }
    }
}

/// État applicatif unique passé à `Router::with_state` (STYLE_RUST.md § 13).
/// Le cloner par requête est le modèle attendu d'Axum : PgPool est
/// Arc-redimensionné en interne, le limiter partage son compteur via Arc.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    /// Poivre applicatif (CDC § 2.5) : jamais en base, jamais dans le dépôt,
    /// uniquement dans l'environnement (PASSWORD_PEPPER). Concaténé au mot de
    /// passe avant hachage Argon2id.
    pub password_pepper: String,
    pub jwt: JwtConfig,
    pub rate_limiter: RateLimiter,
}
