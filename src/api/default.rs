//! Default trait implementations
use crate::api::types::*;
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
