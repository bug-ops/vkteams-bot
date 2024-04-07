use crate::api::types::*;
use std::fmt::{Display, Formatter, Result};
/// Display trait for [`ChatId`]
impl Display for ChatId {
    /// Format [`ChatId`] to string
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}
/// Link basse path for API version
impl Display for APIVersionUrl {
    /// Format [`APIVersionUrl`] to string
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            APIVersionUrl::V1 => write!(f, "bot/v1/"),
        }
    }
}
/// Display trait for [`MultipartName`] enum
impl Display for MultipartName {
    /// Format [`MultipartName`] to string
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            MultipartName::File(..) => write!(f, "file"),
            MultipartName::Image(..) => write!(f, "image"),
            _ => write!(f, ""),
        }
    }
}
