use super::api::ApiResult;
use crate::db::MetadataStore;
use crate::{env, get_filepath};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use tracing::{error, info, instrument};

#[derive(Serialize)]
struct DeleteResponse {
    success: bool,
}

#[instrument(skip(store))]
pub async fn delete_object(
    Path((bucket_name, object_id)): Path<(String, String)>,
    State(store): State<MetadataStore>,
) -> impl IntoResponse {
    let metadata = match store.get_metadata(&bucket_name, &object_id).await {
        Ok(Some(data)) => data,
        Ok(None) => {
            error!("data not found.");
            return ApiResult::Error(
                StatusCode::NOT_FOUND,
                "Data not found. Either bucket_name or object_id may be wrong".to_string(),
            );
        }
        Err(e) => {
            error!("database error: {}", e);
            return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    };
    for i in 0..(env::DATA_SHARDS + env::PARITY_SHARDS) {
        let filepath = get_filepath(&metadata.object_id, i).await;
        if let Some(e) = tokio::fs::remove_file(&filepath).await.err() {
            // ここちょっと怪しい
            error!("DELETE request failed: {}", e);
            return ApiResult::Error(StatusCode::NOT_FOUND, e.to_string());
        }
    }

    match store
        .delete_metadata(&metadata.bucket_name, &metadata.object_id)
        .await
    {
        Ok(_) => {
            info!("metadata '{}' deleted.", metadata.object_id);
            info!("Delete data successfully!");
            ApiResult::Success(StatusCode::OK, DeleteResponse { success: true })
        }
        Err(e) => {
            info!("Failed to delete metadata '{}': {}", metadata.object_id, e);
            ApiResult::Error(StatusCode::NOT_FOUND, e.to_string())
        }
    }
}
