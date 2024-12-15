use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::IntoResponse,
};
use tokio::fs;

pub async fn download_file(Path(filename): Path<String>) -> impl IntoResponse {
    let path = format!("./cdn_storage/{}", filename);
    match fs::read(&path).await {
        Ok(contents) => {
            let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime_type.as_ref())],
                contents,
            )
                .into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
