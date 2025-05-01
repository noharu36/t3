use super::handler::{delete::delete_object, get::get_object, post::post_object};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::limit::RequestBodyLimitLayer;

pub async fn run_server() -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/object", post(post_object))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(3 * 1024 * 1024 * 1024))
        .route("/object", get(get_object))
        .route("/object", delete(delete_object));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on {}", addr);
    axum::serve(TcpListener::bind("127.0.0.1:8080").await.unwrap(), app).await?;

    Ok(())
}
