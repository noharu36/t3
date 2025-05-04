use super::api::ApiResult;
use crate::{env, get_filepath};
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};

#[derive(Deserialize)]
pub struct DeleteRequest {
    object_id: String,
}
#[derive(Serialize)]
struct DeleteResponse {
    success: bool,
}

#[instrument(skip(req))]
pub async fn delete_object(Json(req): Json<DeleteRequest>) -> impl IntoResponse {
    let object_id = &req.object_id;
    for i in 0..(env::DATA_SHARDS + env::PARITY_SHARDS) {
        let filepath = get_filepath(object_id, i).await;
        if let Some(e) = tokio::fs::remove_file(&filepath).await.err() {
            error!("GET request failed: decode error: {}", e);
            return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    }

    info!("Delete data successfully!");
    ApiResult::Success(StatusCode::OK, DeleteResponse { success: true })
}
