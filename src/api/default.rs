//! Default trait implementations
use crate::api::types::*;
use crate::error::ApiError;
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
impl<T> Default for ApiResult<T> {
    fn default() -> Self {
        ApiResult::Error(ApiError {
            ok: false,
            description: String::new(),
        })
    }
}
