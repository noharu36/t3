use anyhow::Result;
use t3::server;

#[tokio::main]
async fn main() -> Result<()> {
    server::run_server().await?;

    Ok(())
}
