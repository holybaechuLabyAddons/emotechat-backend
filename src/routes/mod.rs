use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};

pub mod v1;
pub mod error;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("Resource not found")]
    NotFound,
    #[error("Emote Provider Error: {0}")]
    EmoteProviderError(String),
    #[error("Emote ID is invalid")]
    InvalidId,
    #[error("Legacy API Error: {0}")]
    LegacyApiError(#[from] reqwest::Error),
}

impl ApiError {
    pub fn as_api_error(&self) -> error::ApiError {
        crate::routes::error::ApiError {
            error: match &self {
                ApiError::Database(..) => "database_error",
                ApiError::NotFound => "not_found",
                ApiError::EmoteProviderError(..) => "emote_provider_error",
                ApiError::InvalidId => "invalid_id",
                ApiError::LegacyApiError(..) => "legacy_api_error",
            },
            description: self.to_string()
        }
    }
}


impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            ApiError::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::EmoteProviderError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidId => StatusCode::BAD_REQUEST,
            ApiError::LegacyApiError(..) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error_message = self.as_api_error();

        (status_code, Json(error_message)).into_response()
    }
}