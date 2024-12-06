use crate::graphql::{mutation_root::MutationRoot, query_root::QueryRoot};
use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use axum::response::{self, IntoResponse};
use sea_orm::DatabaseConnection;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(db: DatabaseConnection, redis: redis::Client) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db)
        .data(redis)
        .finish()
}

pub async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}
