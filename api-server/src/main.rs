mod auth;
mod entity;
mod error;
mod graphql;
mod models;

use crate::{
    error::AppError,
    graphql::schema::{graphiql, graphql_handler},
};
use axum::{routing::get, Extension, Router};
use dotenv::dotenv;
use redis::Client as RedisClient;
use sea_orm::Database;
use std::env;
use tokio::net::TcpListener;

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

    // Initialize Redis
    let redis_url = env::var("REDIS_URL")
        .map_err(|_| AppError::Internal("REDIS_URL must be set".to_string()))?;
    let redis = RedisClient::open(redis_url)
        .map_err(|e| AppError::Internal(format!("Failed to connect to Redis: {}", e)))?;

    let schema = graphql::schema::create_schema(db.clone(), redis.clone());

    let app = Router::new().route(
        "/",
        get(graphiql).post(graphql_handler).layer(Extension(schema)),
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
