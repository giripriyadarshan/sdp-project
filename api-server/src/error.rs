use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: sea_orm::DbErr,
        context: Option<String>,
    },

    #[error("Authentication error: {message}")]
    Auth {
        message: String,
        code: AuthErrorCode,
        user_id: Option<String>,
    },

    #[error("Validation error in {field}: {message}")]
    Validation {
        message: String,
        field: String,
        value: Option<String>,
    },

    #[error("Resource {resource} with id {id} not found")]
    NotFound { resource: String, id: String },

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone)]
pub enum AuthErrorCode {
    InvalidCredentials,
    TokenExpired,
    InsufficientPermissions,
    AccountLocked,
}

impl fmt::Display for AuthErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "INVALID_CREDENTIALS"),
            Self::TokenExpired => write!(f, "TOKEN_EXPIRED"),
            Self::InsufficientPermissions => write!(f, "INSUFFICIENT_PERMISSIONS"),
            Self::AccountLocked => write!(f, "ACCOUNT_LOCKED"),
        }
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::Database {
            message: err.to_string(),
            source: err,
            context: None,
        }
    }
}

impl AppError {
    pub fn with_context(self, context: impl Into<String>) -> Self {
        match self {
            Self::Database {
                message, source, ..
            } => Self::Database {
                message,
                source,
                context: Some(context.into()),
            },
            error => error,
        }
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: field.into(),
            value: None,
        }
    }

    pub fn not_found(resource: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
            id: id.into(),
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
            AppError::Validation {
                message,
                field,
                value,
                ..
            } => {
                e.set("code", "VALIDATION_ERROR");
                e.set("message", message);
                e.set("field", field);
                if let Some(v) = value {
                    e.set("invalidValue", v);
                }
            }
            AppError::NotFound { resource, id } => {
                e.set("code", "NOT_FOUND");
                e.set("resource", resource);
                e.set("id", id);
            }
            AppError::Internal(message) => {
                e.set("code", "INTERNAL_ERROR");
                e.set("message", message);
            }
        })
    }
}
