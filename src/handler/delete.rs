use super::api::ApiResult;
use crate::env;
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct DeleteRequest {
    object_id: String,
}
#[derive(Serialize)]
struct DeleteResponse {
    success: bool,
}

pub async fn delete_object(Json(req): Json<DeleteRequest>) -> impl IntoResponse {
    let object_id = &req.object_id;
    for i in 0..(env::DATA_SHARDS + env::PARITY_SHARDS) {
        let mut hasher = DefaultHasher::new();
        format!("{}{}", object_id, i).hash(&mut hasher);
        let hash = hasher.finish();
        let dir_index = (hash % env::NUM_OUTPUT_DIRS as u64) as usize;
        let output_dir_name = format!("{}{}", env::OUTPUT_DIR_PREFIX, dir_index + 1);
        let filepath = PathBuf::from(&output_dir_name).join(format!("{}_{:02}.bin", object_id, i));
        if tokio::fs::remove_file(&filepath).await.is_err() {
            return ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, "err".to_string());
        }
    }
    ApiResult::Success(StatusCode::OK, DeleteResponse { success: true })
}
