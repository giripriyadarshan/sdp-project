mod download;
mod upload;

use crate::download::download_file;
use crate::upload::upload_file;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/upload", post(upload_file))
        .route("/download/:filename", get(download_file));

    let addr = SocketAddr::from(([0, 0, 0, 0], 1026));
    println!("Server running on http://{}", addr);
    axum::serve(
        TcpListener::bind("0.0.0.0:1026".to_string()).await.unwrap(),
        app,
    )
    .await
    .unwrap();
}
