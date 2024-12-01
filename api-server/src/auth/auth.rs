use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_graphql::*;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

//TODO: implement env vars for secret key (also check if 2 secret keys are needed)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct Auth;

impl Auth {
    pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string())
    }

    pub fn verify_password(
        password: &str,
        hash: &str,
    ) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn create_token(user_id: i32, role: String) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = Claims {
            user_id: user_id.to_string(),
            role,
            exp: (now + Duration::days(30)).timestamp(), // 30 days might be unconventional, but we need it because refresh tokens implementation is limited due to OS limitations. also revoke token will prevent misuse (maybe, idk)
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("your_secret_key".as_ref()),
        )
    }

    pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret("your_secret_key".as_ref()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    pub fn verify_role(token: &str, role: &str) -> Result<bool, jsonwebtoken::errors::Error> {
        let claims = Auth::verify_token(token)?;
        Ok(claims.role == role)
    }

    pub fn refresh_token(token: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Auth::verify_token(token)?;
        let now = Utc::now();
        let new_claims = Claims {
            user_id: claims.user_id,
            role: claims.role,
            exp: (now + Duration::days(30)).timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &new_claims,
            &EncodingKey::from_secret("your_secret_key".as_ref()),
        )
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
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let claims = Auth::verify_token(token).map_err(|_| "Invalid token")?;

        if self.allowed_roles.contains(&claims.role) {
            Ok(())
        } else {
            Err("Insufficient permissions".into())
        }
    }
}
