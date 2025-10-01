use anyhow::{Ok, Result};
use tracing_subscriber::EnvFilter;

mod reddit;
use crate::reddit::client::RedditClient;

#[tokio::main()]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let _client: RedditClient = RedditClient::new();

    Ok(())
}
