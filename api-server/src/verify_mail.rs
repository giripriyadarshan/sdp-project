use crate::auth::Auth;
use crate::entity::prelude::Users;
use crate::entity::users::ActiveModel;
use crate::error::{AppError, AuthErrorCode};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Extension;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

pub async fn verify_mail(
    Path(token): Path<String>,
    Extension(postgres): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    let claims = Auth::verify_token(&token.clone()).map_err(|_| AppError::Auth {
        message: "Invalid token".to_string(),
        code: AuthErrorCode::InvalidCredentials,
        user_id: None,
    });

    let claims = match claims {
        Ok(c) => c,
        Err(e) => return e.to_string(),
    };

    let user_id = claims.user_id.parse::<i32>().unwrap();
    let user = Users::find_by_id(user_id)
        .one(&postgres)
        .await
        .unwrap()
        .unwrap();

    let mut user: ActiveModel = user.into();
    user.email_verified = Set(Some(true));
    user.update(&postgres).await.unwrap();

    "Email verified successfully".to_string()
}
