//! Default trait implementations
use crate::api::types::*;
#[cfg(feature = "storage")]
use crate::bot::storage;
/// Default values for [`Keyboard`]
impl Default for Keyboard {
    /// Create new [`Keyboard`] with required params
    fn default() -> Self {
        Self {
            // Empty vector of [`KeyboardButton`]
            buttons: vec![vec![]],
        }
    }
}
#[cfg(feature = "storage")]
impl Default for storage::Tnt {
    fn default() -> Self {
        Self::new()
    }
}

// /// Default values for [`MessageTextParser`]
// impl Default for MessageTextParser {
//     /// Create new [`MessageTextParser`] with required params
//     fn default() -> Self {
//         Self {
//             // Empty vector of [`MessageTextFormat`]
//             text: vec![],
//             parse_mode: ParseMode::HTML,

//         }
//     }
// }
