use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse
};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use tracing::{info, instrument};

use crate::{db::MetadataStore, handler::api::ApiResult};

#[derive(Deserialize, Serialize, Debug)]
struct Bucket {
    id: String,
    bucket_name: String,
    created_at: String,
}

#[derive(Deserialize, Serialize)]
pub struct BucketListResponse {
    buckets: Vec<Bucket>,
}

#[instrument(skip(store))]
pub async fn create_bucket(Path(bucket_name): Path<String>, State(store): State<MetadataStore>) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let bucket = Bucket { id, bucket_name, created_at: now };
    // すでにbucket_nameが存在している場合は作成しない
    let result = sqlx::query!(
        "
        INSERT OR IGNORE INTO bucket_metadata (id, bucket_name, created_at)
        VALUES (?, ?, ?)
        ",
        bucket.id,
        bucket.bucket_name,
        bucket.created_at
    )
    .execute(&store.pool)
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                info!("Bucket '{}' already exists", bucket.bucket_name);
                ApiResult::Error(StatusCode::CONFLICT, format!("Bucket '{}' already exists", bucket.bucket_name))
            } else {
                info!("Bucket '{:?}' created.", bucket.bucket_name);
                ApiResult::Success(StatusCode::CREATED, bucket)
            }
        },
        Err(e) => {
            info!("Create bucket failed: {}",e);
            ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

#[instrument(skip(store))]
pub async fn list_buckets(State(store): State<MetadataStore>) -> impl IntoResponse {
    let result = sqlx::query_as!(
        Bucket,
        "SELECT * FROM bucket_metadata"
    )
    .fetch_all(&store.pool)
    .await;

    match result {
        Ok(buckets) => ApiResult::Success(StatusCode::OK, BucketListResponse {buckets}),
        Err(e) => {
            info!("Get bucket list failed: {}",e);
            ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

#[instrument(skip(store))]
pub async fn delete_bucket(Path(bucket_name): Path<String>, State(store): State<MetadataStore>) -> impl IntoResponse {
    let result = sqlx::query!(
        "
        DELETE FROM bucket_metadata WHERE bucket_name = ?
        ",
        bucket_name,
    )
    .execute(&store.pool)
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                info!("Bucket '{}' NOT_FOUND", bucket_name);
                ApiResult::Error(StatusCode::NOT_FOUND, format!("Bucket '{}' not found.", bucket_name))
            } else {
                info!("Bucket '{}' deleted.", bucket_name);
                ApiResult::Success(StatusCode::OK, format!("Bucket '{}' deleted.", bucket_name))
            }
        },
        Err(e) => {
            info!("Failed to delete bucket '{}': {}", bucket_name, e);
            ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

#[instrument(skip(store))]
pub async fn exist_buckets(bucket_name: &str, store: &MetadataStore) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM bucket_metadata WHERE bucket_name = ?)",
        bucket_name
    )
    .fetch_one(&store.pool)
    .await.map(|i| if i == 1 { true } else { false })

}
