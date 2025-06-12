//! This module contains the templates for the message text parser.
//! https://teams.vk.com/botapi/tutorial/#Format_HTML
use crate::api::types::*;
use crate::error::{BotError, Result};
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
        match self.tmpl.render(self.name.as_str(), &self.ctx) {
            Ok(text) => Ok(text),
            Err(e) => Err(BotError::Template(e)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use tera::{Context, Tera};

    #[derive(Serialize)]
    struct DummyCtx {
        name: String,
    }

    fn make_tera() -> Tera {
        let mut tera = Tera::default();
        tera.add_raw_template("hello", "Hello, {{ name }}!")
            .unwrap();
        tera
    }

    #[test]
    fn test_from_tmpl_sets_template_mode() {
        let tera = make_tera();
        let parser = MessageTextParser::from_tmpl(tera);
        assert_eq!(parser.parse_mode, ParseMode::Template);
    }

    #[test]
    fn test_set_ctx_sets_context_and_name() {
        let tera = make_tera();
        let mut parser = MessageTextParser::from_tmpl(tera);
        let ctx = DummyCtx {
            name: "VK".to_string(),
        };
        let parser2 = parser.set_ctx(ctx, "hello");
        assert_eq!(parser2.name, "hello");
        assert!(parser2.ctx.contains_key("name"));
    }

    #[test]
    fn test_parse_tmpl_success() {
        let tera = make_tera();
        let mut parser = MessageTextParser::from_tmpl(tera);
        let ctx = DummyCtx {
            name: "VK".to_string(),
        };
        let parser2 = parser.set_ctx(ctx, "hello");
        let rendered = parser2.parse_tmpl().unwrap();
        assert_eq!(rendered, "Hello, VK!");
    }

    #[test]
    fn test_parse_tmpl_invalid_template_name() {
        let tera = make_tera();
        let mut parser = MessageTextParser::from_tmpl(tera);
        let ctx = DummyCtx {
            name: "VK".to_string(),
        };
        let parser2 = parser.set_ctx(ctx, "not_found");
        let err = parser2.parse_tmpl().unwrap_err();
        match err {
            BotError::Template(_) => (),
            _ => panic!("Expected BotError::Template, got {:?}", err),
        }
    }
}
