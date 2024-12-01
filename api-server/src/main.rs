mod entity;
mod error;
mod graphql;
mod models;

use crate::graphql::schema::graphiql;
use async_graphql_axum::GraphQL;
use axum::routing::get;
use axum::Router;
use dotenv::dotenv;
use redis::Client as RedisClient;
use sea_orm::Database;
use std::env;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // Initialize SeaORM
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url).await?;

    // Initialize Redis
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis = RedisClient::open(redis_url)?;

    let schema = graphql::schema::create_schema(db.clone(), redis.clone());

    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    println!(
        "GraphQL server running at http://localhost:{}/",
        env::var("PORT").unwrap_or_else(|_| "Err: No PORT SET".to_string())
    );

    axum::serve(
        TcpListener::bind(format!(
            "0.0.0.0:{}",
            env::var("PORT").expect("PORT must be set")
        ))
        .await
        .unwrap(),
        app,
    )
    .await?;

    Ok(())
}
