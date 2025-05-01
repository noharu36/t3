use crate::env::{DATA_SHARDS, NUM_OUTPUT_DIRS, OUTPUT_DIR_PREFIX, PARITY_SHARDS};
use anyhow::Result;
use bytes::BytesMut;
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use tokio::fs;
use tracing::{error, info, instrument};

#[instrument]
pub async fn load_shards(object_id: &str) -> Result<Vec<Option<BytesMut>>> {
    info!("Data loading...");
    let mut shards = vec![None; DATA_SHARDS + PARITY_SHARDS];

    for i in 0..(DATA_SHARDS + PARITY_SHARDS) {
        let mut hasher = DefaultHasher::new();
        format!("{}{}", object_id, i).hash(&mut hasher);
        let hash = hasher.finish();
        let dir_index = (hash % NUM_OUTPUT_DIRS as u64) as usize;
        let output_dir_name = format!("{}{}", OUTPUT_DIR_PREFIX, dir_index + 1);
        let output_path = Path::new(&output_dir_name);
        let filename = format!("{}_{:02}.bin", object_id, i);
        let filepath = output_path.join(filename);

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
