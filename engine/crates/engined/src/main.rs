use anyhow::Result;
use tracing_subscriber::EnvFilter;
use lancea_bus;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    lancea_bus::run_bus().await
}
