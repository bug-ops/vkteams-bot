use crate::api::types::*;
use std::fmt::{Display, Formatter, Result};
/// Display trait for [`ChatId`]
impl Display for ChatId {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}
/// Link basse path for API version
impl Display for APIVersionUrl {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            APIVersionUrl::V1 => write!(f, "bot/v1/"),
        }
    }
}
impl Display for MultipartName {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            MultipartName::File(..) => write!(f, "file"),
            MultipartName::Image(..) => write!(f, "image"),
            _ => write!(f, ""),
        }
    }
}
