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
