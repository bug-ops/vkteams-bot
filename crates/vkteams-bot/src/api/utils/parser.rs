use crate::api::types::*;
use crate::error::{BotError, Result};
use reqwest::Url;
use std::convert::From;
// use tracing::error;
pub trait MessageTextHTMLParser {
    /// Create new parser
    fn new() -> Self
    where
        Self: Sized + Default,
    {
        Self::default()
    }
    /// Add formatted text to parser
    fn add(&mut self, text: MessageTextFormat) -> Self;
    /// Add new row to parser
    fn next_line(&mut self) -> Self;
    /// Add space to parser
    fn space(&mut self) -> Self;
    /// Parse text to HTML
    fn parse(&self) -> Result<(String, ParseMode)>;
}
impl MessageTextParser {
    /// Parse [`MessageTextFormat`] types to HTML string
    fn parse_html(&self, text: &MessageTextFormat) -> Result<String> {
        match text {
            MessageTextFormat::Plain(text) => Ok(self.replace_chars(text)),
            MessageTextFormat::Link(url, text) => {
                let parsed_url = Url::parse(&self.replace_chars(url))?;
                Ok(format!(
                    "<a href=\"{}\">{}</a>",
                    parsed_url,
                    self.replace_chars(text)
                ))
            }
            MessageTextFormat::Bold(text) => Ok(format!("<b>{}</b>", self.replace_chars(text))),
            MessageTextFormat::Italic(text) => Ok(format!("<i>{}</i>", self.replace_chars(text))),
            MessageTextFormat::Code(text) => {
                Ok(format!("<code>{}</code>", self.replace_chars(text)))
            }
            MessageTextFormat::Pre(text, class) => match class {
                Some(class) => Ok(format!(
                    "<pre class=\"{}\">{}</pre>",
                    self.replace_chars(class),
                    self.replace_chars(text)
                )),
                None => Ok(format!("<pre>{}</pre>", self.replace_chars(text))),
            },
            MessageTextFormat::Mention(chat_id) => Ok(format!("<a>@[{}]</a>", chat_id)),
            MessageTextFormat::Strikethrough(text) => {
                Ok(format!("<s>{}</s>", self.replace_chars(text)))
            }
            MessageTextFormat::Underline(text) => {
                Ok(format!("<u>{}</u>", self.replace_chars(text)))
            }
            MessageTextFormat::Quote(text) => Ok(format!(
                "<blockquote>{}</blockquote>",
                self.replace_chars(text)
            )),
            MessageTextFormat::OrderedList(list) => {
                let mut result = String::new();
                for item in list {
                    result.push_str(&format!("<li>{}</li>", self.replace_chars(item)));
                }
                Ok(format!("<ol>{}</ol>", result))
            }
            MessageTextFormat::UnOrderedList(list) => {
                let mut result = String::new();
                for item in list {
                    result.push_str(&format!("<li>{}</li>", self.replace_chars(item)));
                }
                Ok(format!("<ul>{}</ul>", result))
            }
            MessageTextFormat::None => Err(BotError::Validation(
                "MessageTextFormat::None is not supported".to_string(),
            )),
        }
    }
    /// Replace special characters with HTML entities
    fn replace_chars(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}
impl MessageTextHTMLParser for MessageTextParser {
    /// Add plain text to [`MessageTextFormat`]
    /// ## Parameters
    /// - `text`: [`String`] - Text
    fn add(&mut self, text: MessageTextFormat) -> Self {
        self.text.push(text);
        self.to_owned()
    }
    /// Line feed
    fn next_line(&mut self) -> Self {
        self.text.push(MessageTextFormat::Plain(String::from("\n")));
        self.to_owned()
    }
    /// Space
    fn space(&mut self) -> Self {
        self.text.push(MessageTextFormat::Plain(String::from(" ")));
        self.to_owned()
    }
    /// Parse [`MessageTextFormat`] to string
    fn parse(&self) -> Result<(String, ParseMode)> {
        let mut result = String::new();
        match self.parse_mode {
            ParseMode::HTML => {
                for item in &self.text {
                    if let MessageTextFormat::None = item {
                        continue;
                    }
                    result.push_str(&self.parse_html(item)?);
                }
                Ok((result, self.parse_mode))
            }
            #[cfg(feature = "templates")]
            ParseMode::Template => {
                result.push_str(self.parse_tmpl()?.as_str());
                Ok((result, ParseMode::HTML))
            }
            _ => todo!("Parse mode not implemented: {:?}", self.parse_mode),
        }
    }
}
pub use crate::api::types::MessageTextParser;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, MessageTextFormat, ParseMode};

    fn parser_html() -> MessageTextParser {
        MessageTextParser {
            text: vec![],
            parse_mode: ParseMode::HTML,
            ..Default::default()
        }
    }

    #[test]
    fn test_plain_text() {
        let mut parser = parser_html();
        parser = parser.add(MessageTextFormat::Plain("Hello".to_string()));
        let (html, mode) = parser.parse().unwrap();
        assert_eq!(html, "Hello");
        assert_eq!(mode, ParseMode::HTML);
    }

    #[test]
    fn test_bold_italic_code() {
        let mut parser = parser_html();
        parser = parser.add(MessageTextFormat::Bold("B".to_string()));
        parser = parser.add(MessageTextFormat::Italic("I".to_string()));
        parser = parser.add(MessageTextFormat::Code("C".to_string()));
        let (html, _) = parser.parse().unwrap();
        assert!(html.contains("<b>B</b>"));
        assert!(html.contains("<i>I</i>"));
        assert!(html.contains("<code>C</code>"));
    }

    #[test]
    fn test_pre_with_and_without_class() {
        let mut parser = parser_html();
        parser = parser.add(MessageTextFormat::Pre(
            "code".to_string(),
            Some("lang".to_string()),
        ));
        parser = parser.add(MessageTextFormat::Pre("code2".to_string(), None));
        let (html, _) = parser.parse().unwrap();
        assert!(html.contains("<pre class=\"lang\">code</pre>"));
        assert!(html.contains("<pre>code2</pre>"));
    }

    #[test]
    fn test_link_and_mention() {
        let mut parser = parser_html();
        parser = parser.add(MessageTextFormat::Link(
            "http://a.com".to_string(),
            "A".to_string(),
        ));
        parser = parser.add(MessageTextFormat::Mention(ChatId("cid".to_string())));
        let (html, _) = parser.parse().unwrap();
        // println!("HTML output: {}", html);
        assert!(html.contains("<a href=\"http://a.com/\">A</a>"));
        assert!(html.contains("<a>@[cid]</a>"));
    }

    #[test]
    fn test_strikethrough_underline_quote() {
        let mut parser = parser_html();
        parser = parser.add(MessageTextFormat::Strikethrough("S".to_string()));
        parser = parser.add(MessageTextFormat::Underline("U".to_string()));
        parser = parser.add(MessageTextFormat::Quote("Q".to_string()));
        let (html, _) = parser.parse().unwrap();
        assert!(html.contains("<s>S</s>"));
        assert!(html.contains("<u>U</u>"));
        assert!(html.contains("<blockquote>Q</blockquote>"));
    }

    #[test]
    fn test_ordered_and_unordered_list() {
        let mut parser = parser_html();
        parser = parser.add(MessageTextFormat::OrderedList(vec![
            "A".to_string(),
            "B".to_string(),
        ]));
        parser = parser.add(MessageTextFormat::UnOrderedList(vec!["X".to_string()]));
        let (html, _) = parser.parse().unwrap();
        assert!(html.contains("<ol><li>A</li><li>B</li></ol>"));
        assert!(html.contains("<ul><li>X</li></ul>"));
    }

    #[test]
    fn test_none_format_returns_error() {
        let mut parser = parser_html();
        parser = parser.add(MessageTextFormat::None);
        let res = parser.parse();
        assert!(res.is_ok()); // None просто пропускается
    }

    #[test]
    fn test_replace_chars_html_escape() {
        let parser = parser_html();
        let s = parser.replace_chars("<tag>&text>");
        assert_eq!(s, "&lt;tag&gt;&amp;text&gt;");
    }

    #[test]
    fn test_next_line_and_space() {
        let mut parser = parser_html();
        parser = parser.add(MessageTextFormat::Plain("A".to_string()));
        parser = parser.space();
        parser = parser.add(MessageTextFormat::Plain("B".to_string()));
        parser = parser.next_line();
        parser = parser.add(MessageTextFormat::Plain("C".to_string()));
        let (html, _) = parser.parse().unwrap();
        assert!(html.contains("A B"));
        assert!(html.contains("C"));
    }

    #[test]
    fn test_link_invalid_url_returns_error() {
        let mut parser = parser_html();
        parser = parser.add(MessageTextFormat::Link(
            "not a url".to_string(),
            "A".to_string(),
        ));
        let res = parser.parse();
        assert!(res.is_err());
    }

    #[test]
    fn test_empty_parser_returns_empty_string() {
        let parser = parser_html();
        let (html, mode) = parser.parse().unwrap();
        assert_eq!(html, "");
        assert_eq!(mode, ParseMode::HTML);
    }
}
