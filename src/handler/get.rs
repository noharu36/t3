use crate::{db::MetadataStore, decode};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use bytes::BytesMut;
use std::pin::Pin;
use std::task::{Context as stdContext, Poll};
use tokio::io::{self, AsyncRead, ReadBuf};
use tokio_util::io::ReaderStream;
use tracing::{error, info, instrument};

struct MyBytesMut(pub BytesMut);

impl AsyncRead for MyBytesMut {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut stdContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let amt = std::cmp::min(self.0.len(), buf.remaining());
        if amt > 0 {
            let data = self.0.split_to(amt);
            buf.put_slice(&data);
        }
        Poll::Ready(Ok(()))
    }
}

#[instrument(skip(store))]
pub async fn get_object(
    Path((bucket_name, object_key)): Path<(String, String)>,
    State(store): State<MetadataStore>,
) -> impl IntoResponse {
    info!("Handling GET request for object.");
    let metadata = match store.get_metadata(&bucket_name, &object_key).await {
        Ok(Some(data)) => data,
        Ok(None) => {
            error!("data not found.");
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(e) => {
            error!("database error: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let file_name = metadata.file_name.unwrap();
    let file_path = std::path::PathBuf::from(&file_name);
    let content_type = mime_guess::from_path(&file_path).first_or_octet_stream();
    match decode::load_shards(&metadata.object_key).await {
        Ok(mut shards) => match decode::decode_shards(&mut shards).await {
            Ok(data) => {
                let reader = ReaderStream::new(MyBytesMut(data));
                let body = Body::from_stream(reader);
                let headers = [
                    (header::CONTENT_TYPE, content_type.to_string()),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", file_name),
                    ),
                ];

                info!("Load and decode success!");
                (StatusCode::OK, headers, body).into_response()
            }
            Err(e) => {
                error!("GET request failed: decode error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(e) => {
            info!("GET request failed: load error: {}", e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
