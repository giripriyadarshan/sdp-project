use crate::graphql::{
    addresses_objects::{AddressesMutation, AddressesQuery},
    carts_objects::{CartsMutation, CartsQuery},
    orders_objects::{OrdersMutation, OrdersQuery},
    payments_objects::{PaymentsMutation, PaymentsQuery},
    products_objects::{products_mutations::ProductsMutation, products_query::ProductsQuery},
    users_objects::{UsersMutation, UsersQuery},
};
use async_graphql::{http::GraphiQLSource, EmptySubscription, MergedObject, Schema};
use async_graphql_axum::GraphQLRequest;
use axum::{
    http::HeaderMap,
    response::{self, IntoResponse},
    Extension, Json,
};
use sea_orm::DatabaseConnection;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    AddressesQuery,
    CartsQuery,
    OrdersQuery,
    PaymentsQuery,
    ProductsQuery,
    UsersQuery,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    AddressesMutation,
    CartsMutation,
    OrdersMutation,
    PaymentsMutation,
    ProductsMutation,
    UsersMutation,
);

pub fn create_schema(db: DatabaseConnection) -> AppSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(db)
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
