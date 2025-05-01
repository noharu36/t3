use crate::decode;
use axum::{
    Json,
    body::Body,
    http::{StatusCode, header},
    response::IntoResponse,
};
use bytes::BytesMut;
use serde::Deserialize;
use std::pin::Pin;
use std::task::{Context as stdContext, Poll};
use tokio::io::{self, AsyncRead, ReadBuf};
use tokio_util::io::ReaderStream;

#[derive(Deserialize)]
pub struct GetRequest {
    object_id: String,
    file_name: String,
}

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

pub async fn get_object(Json(req): Json<GetRequest>) -> impl IntoResponse {
    let object_id = &req.object_id;
    let file_name = &req.file_name;
    let file_path = std::path::PathBuf::from(&file_name);
    let content_type = mime_guess::from_path(&file_path).first_or_octet_stream();
    match decode::load_shards(object_id).await {
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

                (StatusCode::OK, headers, body).into_response()
            },
            Err(e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        Err(e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
