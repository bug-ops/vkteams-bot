use pretty_env_logger::env_logger;
use vkteams_bot::prelude::Result;

pub mod cli;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    Cli::default().match_input().await?;
    Ok(())
}
