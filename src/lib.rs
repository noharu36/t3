use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::instrument;

pub mod decode;
pub mod encode;
pub mod handler;
pub mod server;

// データシャード数とパリティシャード数
pub mod env {
    pub const DATA_SHARDS: usize = 6;
    pub const PARITY_SHARDS: usize = 3;
    pub const NUM_OUTPUT_DIRS: usize = 9;
    pub const OUTPUT_DIR_PREFIX: &str = "outputs/output";
}

#[instrument(skip(object_id, i))]
pub async fn get_filepath(object_id: &str, i: usize) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    // object_idとシャードインデックスを組み合わせてハッシュ化
    format!("{}{}", object_id, i).hash(&mut hasher);
    let hash = hasher.finish();
    let dir_index = (hash % env::NUM_OUTPUT_DIRS as u64) as usize;
    let output_dir_name = format!("{}{}", env::OUTPUT_DIR_PREFIX, dir_index + 1);
    let output_path = Path::new(&output_dir_name);
    fs::create_dir_all(output_path)
        .await
        .expect("Directory creation failed.");

    let filename = format!("{}_{:02}.bin", object_id, i);
    let filepath = output_path.join(filename);

    filepath
}
