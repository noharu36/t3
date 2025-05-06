use super::api::ApiResult;
use crate::{db::MetadataStore, encode, handler::bucket::exist_buckets};
use axum::{extract::{Path, State, Multipart}, http::StatusCode, response::IntoResponse};
use bytes::Bytes;
use serde::Serialize;
use tracing::{error, info, instrument};

#[derive(Serialize)]
struct PostResponse {
    object_id: String,
}

#[instrument(skip(store, multipart))]
pub async fn post_object(Path((bucket_name, object_key)): Path<(String, String)>, State(store): State<MetadataStore>, mut multipart: Multipart) -> impl IntoResponse {
    info!("Handling POST request for object.");

    // bucketが存在していない場合はエラー
    match exist_buckets(&bucket_name, &store).await {
        Ok(exists) => {
            if !exists {
                return ApiResult::Error(StatusCode::NOT_FOUND, "bucket not found. Please create the bucket first.".to_string())
            }
        },
        Err(e) => return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name();

        if name.as_deref() == Some("file") {
            let file_name = field.file_name().map(|name| name.to_string());
            let content_type = field.content_type().map(|ctype| ctype.to_string());
            match field.bytes().await {
                Ok(bytes) => {
                    let content_length = bytes.len() as i32;
                    info!("bucket_name: {}, object_key: {}, file_name: {:?}, content_type: {:?}, content_length: {}", bucket_name, object_key, file_name, content_type, content_length);
                    let _ = store.insert_metadata(&bucket_name, &object_key, file_name.as_deref(), content_type.as_deref(), content_length).await.unwrap();
                    return store_data(bytes, object_key).await;
                },
                Err(e) => {
                    error!("POST request failed: {}: {}", e.status(), e.body_text());
                    return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
                }
            }
        }
    }

    return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, "error".to_string());
}

#[instrument(skip(bytes))]
async fn store_data(bytes: Bytes, id: String) -> ApiResult<PostResponse> {
    // bytesからbytesmutへの変換.失敗しないことを祈ってunwrap.
    let data = bytes.try_into_mut().unwrap();
    match encode::encode_file(data).await {
        Ok(shards) => match encode::save_shards(&shards, &id).await {
            Ok(_) => {
                info!("Saved data successfully. object_id: {}", id);
                return ApiResult::Success(StatusCode::OK, PostResponse { object_id: id });
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
