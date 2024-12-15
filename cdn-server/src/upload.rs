use axum::{extract::Multipart, http::StatusCode};
use tokio::{fs::File, io::AsyncWriteExt};

pub async fn upload_file(mut multipart: Multipart) -> Result<StatusCode, StatusCode> {
    //creates a folder "./cdn_storage" in the root of the project if it doesn't exist
    let _ = (tokio::fs::create_dir("./cdn_storage").await).is_err();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        let path = format!("./cdn_storage/{}", filename);
        let mut file = File::create(&path).await.unwrap();
        file.write_all(&data).await.unwrap();
    }
    Ok(StatusCode::OK)
}
