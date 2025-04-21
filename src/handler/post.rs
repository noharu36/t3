use super::api::ApiResult;
use crate::encode;
use axum::{extract::Multipart, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
struct PostResponse {
    object_id: String,
}

pub async fn post_object(mut multipart: Multipart) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name();
        let content_type = field.content_type();

        println!(
            "Field: name = {:?}, content_type = {:?}",
            name, content_type
        );

        if name.as_deref() == Some("file") {
            println!("{:?}", field);
            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    println!("bytes() err");
                    println!("{}: {}", e.status(), e.body_text());
                    return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
                }
            };
            let object_id = format!("{}", uuid::Uuid::new_v4());
            // bytesからbytesmutへの変換.失敗しないことを祈ってunwrap.
            let data = data.try_into_mut().unwrap();
            let encoded_shards = match encode::encode_file(data).await {
                Ok(shards) => shards,
                Err(e) => {
                    println!("encode err");
                    return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
                }
            };
            if let Some(e) = encode::save_shards(&encoded_shards, &object_id).await.err() {
                println!("save err");
                return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
            };
            return ApiResult::Success(StatusCode::OK, PostResponse { object_id });
        }
    }
    return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, "error".to_string());
}
