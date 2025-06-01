pub mod errors;
pub mod server;
pub mod types;

use rmcp::ServiceExt;
use std::io::Error;
use std::io::Result;
use tokio::io::{stdin, stdout};
use types::Server;
use vkteams_bot::otlp::init;

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = init().map_err(|e| Error::other(e.to_string()))?;
    let server = Server::default();
    let transport = (stdin(), stdout());
    server.serve(transport).await?.waiting().await?;
    Ok(())
}
