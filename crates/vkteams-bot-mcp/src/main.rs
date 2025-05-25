pub mod errors;
pub mod server;
pub mod types;

use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};
use types::Server;
use vkteams_bot::otlp::init;

#[tokio::main]
async fn main() -> anyhow::Result<(), Box<dyn std::error::Error>> {
    let _guard = init()?;
    let server = Server::default();
    let transport = (stdin(), stdout());
    server.serve(transport).await?.waiting().await?;
    Ok(())
}
