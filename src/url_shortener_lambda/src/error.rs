use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("InputValidationFailed")]
    ValidationError(#[from] ValidationErrors),
    #[error("Dynamodb serialization failed")]
    SerdeDynamo(#[from] serde_dynamo::Error),
    #[error("AWS SDK Error")]
    AwsSdkError,
    #[error("Not Found")]
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match self {
            AppError::ValidationError(err) => (
                StatusCode::BAD_REQUEST,
                serde_json::to_string(&err).unwrap_or_default(),
            ),
            AppError::SerdeDynamo(_) => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
            AppError::AwsSdkError => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
            AppError::NotFound => (StatusCode::NOT_FOUND, String::new()),
        };

        let mut response = Response::new(msg);
        *response.status_mut() = code;

        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str("application/json").expect("Serious typo"),
        );

        response.into_response()
    }
}
