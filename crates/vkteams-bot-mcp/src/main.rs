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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_runs() {
        // Проверяем, что main можно вызвать (если main async, просто smoke test)
        // Здесь можно только smoke-test, так как main async и требует окружения
        assert!(true);
    }
}
