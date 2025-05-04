use crate::env::{DATA_SHARDS, PARITY_SHARDS};
use crate::get_filepath;
use anyhow::Result;
use bytes::BytesMut;
use reed_solomon_erasure::galois_8::ReedSolomon;
use tokio::fs;
use tracing::{error, info, instrument};

#[instrument]
pub async fn load_shards(object_id: &str) -> Result<Vec<Option<BytesMut>>> {
    info!("Data loading...");
    let mut shards = vec![None; DATA_SHARDS + PARITY_SHARDS];

    for i in 0..(DATA_SHARDS + PARITY_SHARDS) {
        let filepath = get_filepath(object_id, i).await;

        match fs::read(&filepath).await {
            Ok(content) => {
                shards[i] = Some(BytesMut::from(&content[..]));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                error!("Shard {} not found in {:?}.", i, filepath);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(shards)
}

#[instrument(skip(shards))]
pub async fn decode_shards(shards: &mut Vec<Option<BytesMut>>) -> Result<BytesMut> {
    info!("decoding...");
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
    r.reconstruct(shards)?;

    let mut output = BytesMut::new();
    for shard in shards.iter().take(DATA_SHARDS) {
        if let Some(s) = shard {
            output.extend_from_slice(s);
        }
    }

    Ok(output)
}
