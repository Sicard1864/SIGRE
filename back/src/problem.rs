use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

/// Corps d'erreur au format ProblemDetails (RFC 9457, ex-7807),
/// imposé par doc/style-api.md pour toutes les erreurs de l'API.
/// Les champs optionnels ne partent que s'ils sont renseignés :
/// `detail`/`code` portent le contrat, `errors` n'existe que pour le 422.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct Problem {
    pub(crate) r#type: &'static str,
    pub(crate) title: &'static str,
    pub(crate) status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) errors: Option<Vec<FieldError>>,
}

/// Un champ en échec de validation, adresse au formulaire du front
/// (STYLE_API.md § 7 : `errors` uniquement pour le 422).
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct FieldError {
    pub field: &'static str,
    pub code: &'static str,
}

/// Construit une réponse d'erreur ProblemDetails.
/// Le détail technique (trace, SQL) ne quitte jamais le serveur : fuite
/// d'information interdite.
pub fn problem(
    status: StatusCode,
    title: &'static str,
    detail: Option<String>,
    code: Option<String>,
    errors: Option<Vec<FieldError>>,
) -> Response {
    let body = Problem {
        r#type: "about:blank",
        title,
        status: status.as_u16(),
        detail,
        code,
        errors,
    };

    (
        status,
        [(header::CONTENT_TYPE, "application/problem+json")],
        serde_json::to_string(&body).expect("sérialisation de Problem impossible"),
    )
        .into_response()
}
