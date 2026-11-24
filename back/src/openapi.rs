use utoipa::OpenApi;

/// Déclaration OpenAPI du back : le contrat est généré depuis le code
/// (STYLE_API.md § 12), servi sur /api/v1/openapi.json. L'UI Swagger n'est
/// montée qu'en debug (cf. lib.rs : jamais en image de production).
#[derive(OpenApi)]
#[openapi(
    info(
        title = "SIGRE API",
        version = "0.1.0",
        description = "API du Système Informatique de Gestion de Révisions Espacées."
    ),
    paths(
        crate::accounts::register,
        crate::accounts::me,
        crate::accounts::update_me,
        crate::accounts::delete_me,
        crate::auth::login,
        crate::auth::refresh,
        crate::questions::list,
    ),
    components(
        schemas(
            crate::accounts::RegisterAccountRequest,
            crate::accounts::UpdateAccountRequest,
            crate::accounts::AccountResponse,
            crate::auth::LoginRequest,
            crate::auth::RefreshRequest,
            crate::auth::TokenPair,
            crate::auth::LoginResponse,
            crate::questions::QuestionSummary,
            crate::problem::Problem,
            crate::problem::FieldError,
        )
    ),
    tags(
        (name = "accounts", description = "Inscription et gestion de son compte"),
        (name = "auth", description = "Échange d'identifiants et de jetons"),
        (name = "questions", description = "Questions publiées")
    )
)]
pub struct ApiDoc;
