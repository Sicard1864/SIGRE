use sqlx::PgPool;

/// État applicatif unique passé à `Router::with_state` (STYLE_RUST.md § 13).
/// Le cloner par requête est le modèle attendu d'Axum : PgPool est
/// Arc-redondant en interne, et le poivre est une chaîne courte.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    /// Poivre applicatif (CDC § 2.5) : jamais en base, jamais dans le dépôt,
    /// uniquement dans l'environnement (PASSWORD_PEPPER). Concaténé au mot de
    /// passe avant hachage Argon2id.
    pub password_pepper: String,
}
