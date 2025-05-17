use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{db::{MetadataStore, Bucket}, handler::api::ApiResult};


#[derive(Deserialize, Serialize)]
pub struct BucketListResponse {
    buckets: Vec<Bucket>,
}

#[instrument(skip(store))]
pub async fn create_bucket(
    Path(bucket_name): Path<String>,
    State(store): State<MetadataStore>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let bucket = Bucket {
        id,
        bucket_name,
        created_at: now,
    };
    // すでにbucket_nameが存在している場合は作成しない
    let result = store.create_bucket(&bucket.id, &bucket.bucket_name, &bucket.created_at).await;
    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                info!("Bucket '{}' already exists", bucket.bucket_name);
                ApiResult::Error(
                    StatusCode::CONFLICT,
                    format!("Bucket '{}' already exists", bucket.bucket_name),
                )
            } else {
                info!("Bucket '{:?}' created.", bucket.bucket_name);
                ApiResult::Success(StatusCode::CREATED, bucket)
            }
        }
        Err(e) => {
            info!("Create bucket failed: {}", e);
            ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

#[instrument(skip(store))]
pub async fn list_buckets(State(store): State<MetadataStore>) -> impl IntoResponse {
    let result = store.get_buckets().await;

    match result {
        Ok(buckets) => ApiResult::Success(StatusCode::OK, BucketListResponse { buckets }),
        Err(e) => {
            info!("Get bucket list failed: {}", e);
            ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

#[instrument(skip(store))]
pub async fn delete_bucket(
    Path(bucket_name): Path<String>,
    State(store): State<MetadataStore>,
) -> impl IntoResponse {
    let result = store.delete_bucket(&bucket_name).await;

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                info!("Bucket '{}' NOT_FOUND", bucket_name);
                ApiResult::Error(
                    StatusCode::NOT_FOUND,
                    format!("Bucket '{}' not found.", bucket_name),
                )
            } else {
                info!("Bucket '{}' deleted.", bucket_name);
                ApiResult::Success(StatusCode::OK, format!("Bucket '{}' deleted.", bucket_name))
            }
        }
        Err(e) => {
            info!("Failed to delete bucket '{}': {}", bucket_name, e);
            ApiResult::Error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

#[instrument(skip(store))]
pub async fn exist_buckets(bucket_name: &str, store: &MetadataStore) -> Result<bool, sqlx::Error> {
    store.exist_buckets(bucket_name)
    .await
    .map(|i| if i == 1 { true } else { false })
}
