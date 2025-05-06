use super::handler::{bucket, delete::delete_object, get::get_object, post::post_object};
use crate::db::MetadataStore;
use anyhow::Result;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, put},
};
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::{info, instrument};

#[instrument]
pub async fn run_server() -> Result<()> {
    info!("Starting the object storage server.");

    dotenvy::dotenv()?;
    let db_path = env::var("DATABASE_URL")?;
    let metadata_store = MetadataStore::new(&db_path).await?;

    let app = app(metadata_store);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("listening on {}", addr);
    axum::serve(TcpListener::bind(addr).await.unwrap(), app).await?;

    Ok(())
}

#[instrument(skip(state))]
pub fn app(state: MetadataStore) -> Router {
    Router::new()
        .merge(object_routes())
        .merge(bucket_routes())
        .with_state(state)
}

#[instrument]
fn object_routes() -> Router<MetadataStore> {
    Router::new()
        .route(
            "/bucket/{:bucket_name}/{:object_key}",
            put(post_object).get(get_object).delete(delete_object),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(3 * 1024 * 1024 * 1024))
}

#[instrument]
fn bucket_routes() -> Router<MetadataStore> {
    Router::new()
        .route(
            "/bucket/{:bucket_name}",
            put(bucket::create_bucket).delete(bucket::delete_bucket),
        )
        .route("/bucket", get(bucket::list_buckets))
}
