use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::BoxError;
use sea_orm::DbErr;
use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: DbErr,
        context: Option<String>,
    },

    #[error("Authentication error: {message}")]
    Auth {
        message: String,
        code: AuthErrorCode,
        user_id: Option<String>,
    },

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone)]
pub enum AuthErrorCode {
    InvalidCredentials,
    TokenExpired,
    InsufficientPermissions,
}

impl fmt::Display for AuthErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "INVALID_CREDENTIALS"),
            Self::TokenExpired => write!(f, "TOKEN_EXPIRED"),
            Self::InsufficientPermissions => write!(f, "INSUFFICIENT_PERMISSIONS"),
        }
    }
}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        Self::Database {
            message: err.to_string(),
            source: err,
            context: None,
        }
    }
}

impl async_graphql::ErrorExtensions for AppError {
    fn extend(&self) -> async_graphql::Error {
        let error = async_graphql::Error::new(self.to_string());

        error.extend_with(|_, e| match self {
            AppError::Database {
                message, context, ..
            } => {
                e.set("code", "DATABASE_ERROR");
                e.set("message", message);
                if let Some(ctx) = context {
                    e.set("context", ctx);
                }
            }
            AppError::Auth {
                message,
                code,
                user_id,
                ..
            } => {
                e.set("code", code.to_string());
                e.set("message", message);
                if let Some(uid) = user_id {
                    e.set("userId", uid);
                }
            }
            AppError::Internal(message) => {
                e.set("code", "INTERNAL_ERROR");
                e.set("message", message);
            }
        })
    }
}

pub async fn handle_error(error: BoxError) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", error),
    )
}
