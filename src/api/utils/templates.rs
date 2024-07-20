//! This module contains the templates for the message text parser.
//! https://teams.vk.com/botapi/tutorial/#Format_HTML
use crate::api::types::*;
use anyhow::Result;
use serde::Serialize;
use tera::Context;
use tera::Tera;

use super::MessageTextParser;
impl MessageTextParser {
    pub fn from_tmpl(tmpl: Tera) -> Self {
        Self {
            tmpl,
            ctx: Context::new(),
            parse_mode: ParseMode::Template,
            ..Default::default()
        }
    }
    pub fn parse_tmpl(&self) -> Result<String> {
        match self.tmpl.render(&self.name.as_str(), &self.ctx) {
            Ok(text) => Ok(text),
            Err(e) => Err(e.into()),
        }
    }
    pub fn set_ctx<T>(&mut self, msg: T, name: &str) -> Self
    where
        T: Serialize,
    {
        self.ctx = Context::from_serialize(msg).unwrap();
        self.name = name.to_string();
        self.to_owned()
    }
}
