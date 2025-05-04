use super::api::ApiResult;
use crate::encode;
use axum::{extract::Multipart, http::StatusCode, response::IntoResponse};
use bytes::Bytes;
use serde::Serialize;
use tracing::{error, info, instrument};

#[derive(Serialize)]
struct PostResponse {
    object_id: String,
}

#[instrument(skip(multipart))]
pub async fn post_object(mut multipart: Multipart) -> impl IntoResponse {
    info!("Handling POST request for object.");
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name();
        let content_type = field.content_type();

        info!(
            "Field: name = {:?}, content_type = {:?}",
            name, content_type
        );

        if name.as_deref() == Some("file") {
            match field.bytes().await {
                Ok(bytes) => return store_data(bytes).await,
                Err(e) => {
                    error!("POST request failed: {}: {}", e.status(), e.body_text());
                    return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
                }
            }
        }
    }

    return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, "error".to_string());
}

async fn store_data(bytes: Bytes) -> ApiResult<PostResponse> {
    let object_id = format!("{}", uuid::Uuid::new_v4());
    // bytesからbytesmutへの変換.失敗しないことを祈ってunwrap.
    let data = bytes.try_into_mut().unwrap();
    match encode::encode_file(data).await {
        Ok(shards) => match encode::save_shards(&shards, &object_id).await {
            Ok(_) => {
                info!("Saved data successfully. object_id: {}", object_id);
                return ApiResult::Success(StatusCode::OK, PostResponse { object_id });
            }
            Err(e) => {
                error!("POST request failed: save error: {}", e);
                return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
            }
        },
        Err(e) => {
            error!("POST request failed: encode error: {}", e);
            return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    }
}
