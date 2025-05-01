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
