use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

/// Corps d'erreur au format ProblemDetails (RFC 9457, ex-7807),
/// imposé par doc/style-api.md pour toutes les erreurs de l'API.
#[derive(serde::Serialize)]
struct Problem {
    r#type: &'static str,
    title: &'static str,
    status: u16,
}

/// Construit une réponse d'erreur ProblemDetails.
/// Le détail technique (trace, SQL) ne quitte jamais le serveur : fuite d'information interdite.
pub fn problem(status: StatusCode, title: &'static str) -> Response {
    let body = Problem {
        r#type: "about:blank",
        title,
        status: status.as_u16(),
    };

    (
        status,
        [(header::CONTENT_TYPE, "application/problem+json")],
        serde_json::to_string(&body).expect("sérialisation de Problem impossible"),
    )
        .into_response()
}
