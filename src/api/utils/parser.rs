use crate::api::types::*;
use anyhow::{Result, anyhow};
use reqwest::Url;
use std::convert::From;
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
    fn parse(&self) -> (String, ParseMode);
}
impl MessageTextParser {
    /// Parse [`MessageTextFormat`] types to HTML string
    fn parse_html(&self, text: &MessageTextFormat) -> Result<String> {
        match text {
            MessageTextFormat::Plain(text) => Ok(self.replace_chars(text)),
            MessageTextFormat::Link(url, text) => {
                let parsed_url = match Url::parse(&self.replace_chars(url)) {
                    Ok(url) => url,
                    Err(e) => return Err(e.into()),
                };
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
            MessageTextFormat::None => Err(anyhow!("MessageTextFormat::None is not supported")),
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
    fn parse(&self) -> (String, ParseMode) {
        let mut result = String::new();
        match self.parse_mode {
            ParseMode::HTML => {
                for item in &self.text {
                    if let MessageTextFormat::None = item {
                        continue;
                    }
                    result.push_str(&self.parse_html(item).unwrap());
                }
                (result, self.parse_mode)
            }
            #[cfg(feature = "templates")]
            ParseMode::Template => {
                let str = match &self.parse_tmpl() {
                    Ok(text) => text.to_owned(),
                    Err(e) => {
                        error!("Error: {}", e);
                        return (String::new(), ParseMode::HTML);
                    }
                };
                result.push_str(str.as_str());
                (result, ParseMode::HTML)
            }
            _ => todo!("Not implemented parse mode: {:?}", self.parse_mode),
        }
    }
}
