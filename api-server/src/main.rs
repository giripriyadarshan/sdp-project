mod auth;
mod entity;
mod error;
mod graphql;
mod models;
mod verify_mail;

use crate::error::handle_error;
use crate::verify_mail::verify_mail;
use crate::{
    error::AppError,
    graphql::schema::{graphiql, graphql_handler},
};
use axum::{
    error_handling::HandleErrorLayer,
    http::{
        header::{
            ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE,
        },
        Method,
    },
    routing::get,
    BoxError, Extension, Router,
};
use dotenv::dotenv;
use sea_orm::Database;
use std::env;
use tokio::net::TcpListener;
use tower::{layer::util::Identity, ServiceBuilder};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();

    // Initialize SeaORM
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| AppError::Internal("DATABASE_URL must be set".to_string()))?;
    let db = Database::connect(&database_url)
        .await
        .map_err(|e| AppError::Database {
            message: "Failed to connect to database".to_string(),
            source: e,
            context: None,
        })?;

    let schema = graphql::schema::create_schema(db.clone());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_CREDENTIALS,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_ALLOW_METHODS,
        ]);

    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        .layer(cors);

    let app = Router::new()
        .route(
            "/",
            get(graphiql)
                .post(graphql_handler)
                .layer::<_, BoxError>(Extension(schema))
                .layer(Identity::new())
                .layer(middleware_stack.clone()),
        )
        .route(
            "/verify/:token",
            get(verify_mail)
                .layer::<_, BoxError>(Extension(db))
                .layer(Identity::new())
                .layer(middleware_stack),
        );

    let port = env::var("PORT").map_err(|_| AppError::Internal("PORT must be set".to_string()))?;
    println!("GraphQL server running at http://localhost:{}/", port);

    axum::serve(
        TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .map_err(|e| AppError::Internal(format!("Failed to bind to port: {}", e)))?,
        app,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
