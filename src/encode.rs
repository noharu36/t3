use crate::env::{DATA_SHARDS, PARITY_SHARDS};
use crate::get_filepath;
use anyhow::Result;
use bytes::{BufMut, BytesMut};
use rayon::prelude::*;
use reed_solomon_erasure::galois_8::ReedSolomon;
use tokio::fs;
use tracing::{info, instrument};

#[instrument(skip(content))]
pub async fn encode_file(content: BytesMut) -> Result<Vec<BytesMut>> {
    info!("encoding...");
    let content_size = content.len();
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
    let shard_len = (content_size + DATA_SHARDS - 1) / DATA_SHARDS;
    let mut shards: Vec<BytesMut> = (0..DATA_SHARDS)
        .into_par_iter()
        .map(|i| {
            let start = i * shard_len;
            let end = std::cmp::min((i + 1) * shard_len, content_size);
            let mut shard = BytesMut::from(&content[start..end]);
            while shard.len() < shard_len {
                // ライブラリの仕様上、shardの長さは全て等しくなければいけないのでゼロでパディングしている
                shard.put_u8(0);
            }
            shard
        })
        .collect();

    for _ in 0..PARITY_SHARDS {
        shards.push(BytesMut::zeroed(shard_len));
    }

    r.encode(&mut shards)?;
    info!("encoded!");

    Ok(shards)
}

#[instrument(skip(shards))]
pub async fn save_shards(shards: &Vec<BytesMut>, object_id: &str) -> Result<()> {
    info!("Starting save data...");
    for i in 0..shards.len() {
        let filepath = get_filepath(object_id, i).await;
        fs::write(&filepath, &shards[i]).await?;
        info!("Saved shard {} to {:?}", i, filepath);
    }
    Ok(())
}
