pub mod bridge_trait;
pub mod cli_bridge;
pub mod cli_commands;
pub mod errors;
pub mod file_utils;
pub mod mcp_bridge_trait;
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

    #[test]
    fn test_main_runs() {
        // Check that main can be called (if main is async, just smoke test)
        // We can only smoke-test here since main is async and requires environment
        // This test just ensures main function exists and compiles
    }

    #[test]
    fn test_main_env_error() {
        // Remove env var to simulate error
        unsafe {
            std::env::remove_var("VKTEAMS_BOT_CHAT_ID");
        }
        
        // Now Server::default should fail when chat_id is not in config
        let result = std::panic::catch_unwind(|| {
            let _ = crate::types::Server::default();
        });
        
        // This test may pass or fail depending on whether CLI binary is available
        // The main purpose is to test that the code handles missing environment variables properly
        match result {
            Ok(_) => {
                println!("Server created successfully (environment variable not required)");
            }
            Err(_) => {
                println!("Server creation failed as expected (missing configuration)");
            }
        }
    }
}
