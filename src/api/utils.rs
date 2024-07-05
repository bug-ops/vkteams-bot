//! Module with traits for [`Keyboard`], [`MessageTextParser`], etc.
use crate::api::types::*;
use anyhow::{anyhow, Result};
use reqwest::Url;
use std::convert::From;

impl From<Keyboard> for String {
    /// # Convert [`Keyboard`] to JSON string
    fn from(val: Keyboard) -> Self {
        val.get_keyboard()
    }
}
impl Keyboard {
    /// # Create new [`Keyboard`]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    /// # Append row with buttons to [`Keyboard`]
    pub fn add_row(&mut self) -> Self {
        self.buttons.push(vec![]);
        self.to_owned()
    }
    /// # Get index of last row of [`Keyboard`]
    pub fn get_row_index(&self) -> usize {
        self.buttons.len() - 1
    }
    /// # Append button to last row of [`Keyboard`]
    /// Maximum buttons in row is 8. If row is full, add new row
    pub fn add_button(&mut self, button: &ButtonKeyboard) -> Self {
        // IF row is full, add new row
        if self.buttons[self.get_row_index()].len() >= 8 {
            self.add_row();
        }
        let row_index = self.get_row_index();
        self.buttons[row_index].push(button.clone());
        self.to_owned()
    }
    /// # Get keyboard as JSON string
    fn get_keyboard(&self) -> String {
        serde_json::to_string(&self.buttons).unwrap()
    }
}
impl ButtonKeyboard {
    /// Create new [`ButtonKeyboard`] with URL
    /// ## Parameters
    /// - `text`: [`String`] - Button text
    /// - `url`: [`String`] - URL
    /// - `style`: [`ButtonStyle`] - Button style
    pub fn url(text: String, url: String, style: ButtonStyle) -> Self {
        ButtonKeyboard {
            text,
            style: Some(style),
            url: Some(url),
            callback_data: None,
        }
    }
    /// Create new [`ButtonKeyboard`] with callback data
    /// ## Parameters
    /// - `text`: [`String`] - Button text
    /// - `cb`: [`String`] - Callback data
    /// - `style`: [`ButtonStyle`] - Button style
    pub fn cb(text: String, cb: String, style: ButtonStyle) -> Self {
        ButtonKeyboard {
            text,
            style: Some(style),
            url: None,
            callback_data: Some(cb),
        }
    }
}
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
        for item in &self.text {
            if let MessageTextFormat::None = item {
                continue;
            }
            match self.parse_mode {
                ParseMode::HTML => {
                    result.push_str(&self.parse_html(item).unwrap());
                }
                _ => todo!("Not implemented parse mode: {:?}", self.parse_mode),
            }
        }
        (result, self.parse_mode)
    }
}
#[allow(unused_variables)]
pub trait MessageTextSetters {
    /// Set text
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`] - Text parser
    fn set_text(&mut self, parser: MessageTextParser) -> Self
    where
        Self: Sized + Clone,
    {
        warn!("Method not implemented");
        self.to_owned()
    }
    /// Set reply message ID
    /// ## Parameters
    /// - `msg_id`: [`MsgId`] - Message ID
    fn set_reply_msg_id(&mut self, msg_id: MsgId) -> Self
    where
        Self: Sized + Clone,
    {
        warn!("Method not implemented");
        self.to_owned()
    }
    /// Set forward message ID
    /// ## Parameters
    /// - `chat_id`: [`ChatId`] - Chat ID
    /// - `msg_id`: [`MsgId`] - Message ID
    fn set_forward_msg_id(&mut self, chat_id: ChatId, msg_id: MsgId) -> Self
    where
        Self: Sized + Clone,
    {
        warn!("Method not implemented");
        self.to_owned()
    }
    /// Set keyboard
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`] - Keyboard
    fn set_keyboard(&mut self, keyboard: Keyboard) -> Self
    where
        Self: Sized + Clone,
    {
        warn!("Method not implemented");
        self.to_owned()
    }
}
