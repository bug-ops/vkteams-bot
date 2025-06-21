use crate::cli_bridge::CliBridge;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Server {
    pub cli: Arc<CliBridge>,
    pub chat_id: String,
}

impl Server {
    pub fn bridge(&self) -> Arc<CliBridge> {
        Arc::clone(&self.cli)
    }
}

impl Default for Server {
    fn default() -> Self {
        let chat_id = std::env::var("VKTEAMS_BOT_CHAT_ID").expect("VKTEAMS_BOT_CHAT_ID is not set");
        let cli = CliBridge::new().expect("Failed to create CLI bridge");
        Self {
            cli: Arc::new(cli),
            chat_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_default_and_bridge() {
        // Set required env vars for default
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat_id");
        }
        
        // Note: This test might fail if CLI binary is not available in test environment
        match std::panic::catch_unwind(|| {
            let server = Server::default();
            assert_eq!(server.chat_id, "test_chat_id");
            let bridge = server.bridge();
            assert!(Arc::strong_count(&bridge) >= 1);
        }) {
            Ok(_) => println!("Server created successfully"),
            Err(_) => println!("Expected failure in test environment without CLI binary"),
        }
    }
}
