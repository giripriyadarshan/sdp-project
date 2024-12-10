use crate::auth::auth::Auth;
use crate::graphql::{mutation_root::MutationRoot, query_root::QueryRoot};
use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::http::HeaderMap;
use axum::response::{self, IntoResponse};
use axum::{Extension, Json};
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

pub async fn graphql_handler(
    schema: Extension<AppSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> impl IntoResponse {
    let token = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(String::from);

    let mut request = req.into_inner();

    // Add the token to the request context
    if let Some(token) = token {
        request = request.data(token);
    }

    let response = schema.execute(request).await;
    Json(response)
}
