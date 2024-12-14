use crate::error::{AppError, AuthErrorCode};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use async_graphql::*;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_regex::regex;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct Auth;

impl Auth {
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let secret_key = env::var("PASSWORD_SECRET").unwrap().as_bytes().to_owned();
        let argon2 = Argon2::new_with_secret(
            &secret_key,
            Algorithm::Argon2id,
            Version::V0x13,
            Params::default(),
        )
        .unwrap();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
        let secret_key = env::var("PASSWORD_SECRET").unwrap().as_bytes().to_owned();
        Ok(Argon2::new_with_secret(
            &secret_key,
            Algorithm::Argon2id,
            Version::V0x13,
            Params::default(),
        )
        .unwrap()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
    }

    pub fn create_token(user_id: i32, role: String) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = Claims {
            user_id: user_id.to_string(),
            role,
            exp: (now + Duration::days(30)).timestamp(), // 30 days might be unconventional, but we need it because refresh tokens implementation is limited due to OS limitations. also revoke token will prevent misuse (maybe, idk)
            iat: now.timestamp(),
        };

        let secret = env::var("TOKEN_SECRET").map_err(|_| {
            AppError::Internal("TOKEN_SECRET environment variable not set".to_string())
        })?;

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| AppError::Internal(format!("Token creation failed: {}", e)))
    }

    pub fn verify_token(token: &str) -> Result<Claims, AppError> {
        let secret = env::var("TOKEN_SECRET").map_err(|_| {
            AppError::Internal("TOKEN_SECRET environment variable not set".to_string())
        })?;

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map(|token_data| token_data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::Auth {
                message: "Token has expired".to_string(),
                code: AuthErrorCode::TokenExpired,
                user_id: None,
            },
            _ => AppError::Auth {
                message: format!("Invalid token: {}", e),
                code: AuthErrorCode::InvalidCredentials,
                user_id: None,
            },
        })
    }

    pub fn refresh_token(token: &str) -> Result<String, AppError> {
        let claims = Auth::verify_token(token)?;
        let now = Utc::now();
        let new_claims = Claims {
            user_id: claims.user_id,
            role: claims.role,
            exp: (now + Duration::days(30)).timestamp(),
            iat: now.timestamp(),
        };

        let secret = env::var("TOKEN_SECRET").expect("TOKEN_SECRET must be set");
        encode(
            &Header::default(),
            &new_claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| AppError::Internal(format!("Token creation failed: {}", e)))
    }

    pub fn check_password_strength(password: &str) -> Result<(), &'static str> {
        if password.len() < 8
            || !regex!(r"[A-Z]").is_match(password)
            || !regex!(r"[0-9]").is_match(password)
            || !regex!(r"[a-z]").is_match(password)
            || !regex!(r"[!@#$%^&*]").is_match(password)
        {
            return Err("You're not the only person who knows about the developer tools in the browser. Nice try bro");
        }

        Ok(())
    }

    pub fn check_email(email: &str) -> Result<(), &'static str> {
        if !regex!(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").is_match(email) {
            return Err("Invalid email address");
        }

        Ok(())
    }

    // // TODO: implement token revocation in GraphQL models (mutation) or skip implementation if it takes too much time/resources.
    // pub fn revoke_token(token: &str) -> Result<(), jsonwebtoken::errors::Error> {
    //     let claims = Auth::verify_token(token)?;
    //     let now = Utc::now();
    //     let new_claims = Claims {
    //         user_id: claims.user_id,
    //         role: claims.role,
    //         exp: now.timestamp(),
    //         iat: now.timestamp(),
    //     };
    //
    //     encode(
    //         &Header::default(),
    //         &new_claims,
    //         &EncodingKey::from_secret("your_secret_key".as_ref()),
    //     )?;
    //     Ok(())
    // }
}

pub const ROLE_SUPPLIER: &str = "supplier";
pub const ROLE_CUSTOMER: &str = "customer";

pub struct RoleGuard {
    pub allowed_roles: Vec<String>,
}

impl RoleGuard {
    pub fn new(roles: Vec<&str>) -> Self {
        Self {
            allowed_roles: roles.into_iter().map(String::from).collect(),
        }
    }
}

impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let token = ctx.data_opt::<String>().ok_or(AppError::Auth {
            message: "No authorization token found".to_string(),
            code: AuthErrorCode::InvalidCredentials,
            user_id: None,
        })?;

        let claims = Auth::verify_token(token)?;

        if self.allowed_roles.contains(&claims.role) {
            Ok(())
        } else {
            Err(AppError::Auth {
                message: "Insufficient permissions".to_string(),
                code: AuthErrorCode::InsufficientPermissions,
                user_id: Some(claims.user_id),
            }
            .into())
        }
    }
}
