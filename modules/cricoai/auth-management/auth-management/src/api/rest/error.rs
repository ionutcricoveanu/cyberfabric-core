use http::StatusCode;
use modkit::api::problem::Problem;
use crate::domain::error::DomainError;

impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        let trace_id = tracing::Span::current()
            .id()
            .map(|id| id.into_u64().to_string());

        let (status, code, title, detail) = match &e {
            DomainError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "AUTH_NOT_FOUND",
                "Not found",
                msg.clone(),
            ),
            DomainError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                "AUTH_FORBIDDEN",
                "Forbidden",
                msg.clone(),
            ),
            DomainError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "AUTH_BAD_REQUEST",
                "Bad request",
                msg.clone(),
            ),
            DomainError::Database(msg) => {
                tracing::error!(error = ?e, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "AUTH_DB_ERROR",
                    "Database error",
                    msg.clone(),
                )
            }
        };

        let mut problem = Problem::new(status, title, detail)
            .with_type(format!("https://errors.hyperspot.com/{}", code))
            .with_code(code);

        if let Some(id) = trace_id {
            problem = problem.with_trace_id(id);
        }

        problem
    }
}
