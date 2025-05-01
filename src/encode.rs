use crate::env::{DATA_SHARDS, NUM_OUTPUT_DIRS, OUTPUT_DIR_PREFIX, PARITY_SHARDS};
use anyhow::Result;
use bytes::{BufMut, BytesMut};
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use tokio::fs;
use rayon::prelude::*;

pub async fn encode_file(content: BytesMut) -> Result<Vec<BytesMut>> {
    let content_size = content.len();
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
    let shard_len = (content_size + DATA_SHARDS - 1) / DATA_SHARDS;
    let mut shards: Vec<BytesMut> = (0..DATA_SHARDS).into_par_iter()
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

    Ok(shards)
}

pub async fn save_shards(shards: &Vec<BytesMut>, object_id: &str) -> Result<()> {
    for i in 0..shards.len() {
        let mut hasher = DefaultHasher::new();
        // object_idとシャードインデックスを組み合わせてハッシュ化
        format!("{}{}", object_id, i).hash(&mut hasher);
        let hash = hasher.finish();
        // どのストレージに格納するかをハッシュ値を使って決めている
        let dir_index = (hash % NUM_OUTPUT_DIRS as u64) as usize;
        let output_dir_name = format!("{}{}", OUTPUT_DIR_PREFIX, dir_index + 1);
        let output_path = Path::new(&output_dir_name);
        fs::create_dir_all(output_path).await?;

        let filename = format!("{}_{:02}.bin", object_id, i);
        let filepath = output_path.join(filename);
        fs::write(&filepath, &shards[i]).await?;
        println!("Saved shard {} to {:?}", i, filepath);
    }
    Ok(())
}
