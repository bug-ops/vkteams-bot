use crate::api::types::*;

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            buttons: vec![vec![]],
        }
    }
}
impl Default for MessageTextParser {
    /// Create new [`MessageText`] with required params
    fn default() -> Self {
        Self {
            text: vec![MessageTextFormat::None],
            parse_mode: ParseMode::HTML,
        }
    }
}
