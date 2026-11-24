use utoipa::OpenApi;

/// Déclaration OpenAPI du back : le contrat est généré depuis le code
/// (STYLE_API.md § 12), servi sur /api/v1/openapi.json. L'UI Swagger n'est
/// montée qu'en debug (cf. main.rs : jamais en image de production).
#[derive(OpenApi)]
#[openapi(
    info(
        title = "SIGRE API",
        version = "0.1.0",
        description = "API du Système Informatique de Gestion de Révisions Espacées."
    ),
    paths(crate::accounts::register, crate::questions::list),
    components(
        schemas(
            crate::accounts::RegisterAccountRequest,
            crate::accounts::AccountResponse,
            crate::questions::QuestionSummary,
            crate::problem::Problem,
            crate::problem::FieldError,
        )
    ),
    tags(
        (name = "accounts", description = "Inscription et consultation de son compte"),
        (name = "questions", description = "Questions publiées")
    )
)]
pub struct ApiDoc;
