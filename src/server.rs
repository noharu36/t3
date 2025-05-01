use super::handler::{delete::delete_object, get::get_object, post::post_object};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::{info, instrument};

#[instrument]
pub async fn run_server() -> Result<(), std::io::Error> {
    info!("Starting the object storage server.");
    let app = Router::new()
        .route("/object", post(post_object))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(3 * 1024 * 1024 * 1024))
        .route("/object", get(get_object))
        .route("/object", delete(delete_object));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::serve(TcpListener::bind(addr).await.unwrap(), app).await?;
    info!("listening on {}", addr);

    Ok(())
}
