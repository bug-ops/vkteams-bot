pub mod errors;
pub mod server;
pub mod types;

use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};
use types::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = Server::default();
    let transport = (stdin(), stdout());
    server.serve(transport).await?.waiting().await?;
    Ok(())
}
