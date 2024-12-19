// This file contains few comments which may feel out of place, but they are here only to explain the concepts of OOP in Rust.

use crate::error::{AppError, AuthErrorCode};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use async_graphql::*;
use chrono::{Duration, TimeDelta, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_regex::regex;
use mail_send::mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
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
    // Abstraction
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

    pub fn create_token(
        user_id: i32,
        role: String,
        duration: TimeDelta,
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = Claims {
            user_id: user_id.to_string(),
            role,
            exp: (now + duration).timestamp(), // 30 days might be unconventional, but we need it because refresh tokens implementation is limited due to OS limitations. also revoke token will prevent misuse (maybe, idk)
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

    pub async fn send_email_verification(
        email: String,
        id: i32,
        role: String,
    ) -> Result<String, &'static str> {
        let token = Self::create_token(id, role, Duration::minutes(15))
            .map_err(|_| "Failed to create token")?;

        let port = env::var("PORT").map_err(|_| "PORT must be set")?;

        // Build a simple multipart message
        let message = MessageBuilder::new()
            .from((
                "Verify Mail Id Bitte",
                "postmaster@testing.giripriyadarshan.com",
            ))
            .to(email)
            .subject("Nine11 email verification")
            .html_body(
                "<a href=\"http://localhost:".to_string()
                    + port.as_str()
                    + "/verify/"
                    + token.as_str()
                    + "\">Click here to verify your email</a>",
            );

        // Connect to the SMTP submissions port, upgrade to TLS and
        // authenticate using the provided credentials.
        let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
        SmtpClientBuilder::new("smtp.mailgun.org", 587)
            .implicit_tls(false)
            .credentials((smtp_username.as_str(), smtp_password.as_str()))
            .connect()
            .await
            .expect("Failed to connect to SMTP server")
            .send(message)
            .await
            .expect("Failed to send email");
        Ok("Email verification sent".to_string())
    }
}

pub const ROLE_SUPPLIER: &str = "supplier";
pub const ROLE_CUSTOMER: &str = "customer";

// struct name is equivalent to a class name in OOP
// it consists of data members
pub struct RoleGuard {
    pub allowed_roles: Vec<String>,
}

// data functions are implemented using impl keyword
impl RoleGuard {
    // Encapsulation using a constructor like function to bind the member data to its member functions
    pub fn new(roles: Vec<&str>) -> Self {
        Self {
            allowed_roles: roles.into_iter().map(String::from).collect(),
        }
    }
}

// Inheritance
impl Guard for RoleGuard {
    // Polymorphism
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let token = ctx.data_opt::<String>().ok_or(AppError::Auth {
            message: "No authorization token found".to_string(),
            code: AuthErrorCode::InvalidCredentials,
            user_id: None,
        })?;

        let claims = Auth::verify_token(token)?;

        // Open recursion using 'self' keyword
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
